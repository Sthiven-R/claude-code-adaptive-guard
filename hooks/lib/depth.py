"""
depth.py - Score response depth with full explainability.

Mirror of complexity.py: returns `(total_score, breakdown)` plus a
structural-gap detector `detect_missing_aspects` that compares the
prompt's implicit demands against the response's shape.

The embedder module is imported LAZILY inside score_depth_explained
(same pattern as complexity.py) so the heavy fastembed dependency
isn't loaded for hook invocations that only need structural scoring.
"""
from __future__ import annotations

import re
from typing import Any

from anchors import DEEP_RESPONSE_ANCHORS, SHALLOW_RESPONSE_ANCHORS
from structural import (
    has_code_block,
    has_heading,
    has_list,
    has_table,
    score_depth_structural,
)


def score_depth_explained(response: str) -> tuple[int, dict]:
    if not response:
        return 0, {
            "total": 0,
            "structural": 0,
            "semantic": None,
            "blend_weights": None,
            "axes": {},
            "signals": {},
        }

    structural_score, breakdown = score_depth_structural(response)
    breakdown["structural"] = structural_score
    breakdown["semantic"] = None
    breakdown["blend_weights"] = None

    from embedder import is_available, score_affinity

    if not is_available():
        breakdown["total"] = structural_score
        return structural_score, breakdown

    try:
        semantic = score_affinity(
            response,
            positive_anchors=DEEP_RESPONSE_ANCHORS,
            negative_anchors=SHALLOW_RESPONSE_ANCHORS,
        )
    except Exception:
        semantic = None

    if semantic is None:
        breakdown["total"] = structural_score
        return structural_score, breakdown

    w_sem, w_str = 0.55, 0.45
    blended = int(round(semantic * w_sem + structural_score * w_str))
    blended = max(0, min(100, blended))

    breakdown["semantic"] = semantic
    breakdown["blend_weights"] = {"semantic": w_sem, "structural": w_str}
    breakdown["total"] = blended
    return blended, breakdown


def detect_missing_aspects(
    prompt: str,
    response: str,
    cfg: dict[str, Any] | None = None,
) -> list[str]:
    """Return language-agnostic structural gaps in the response."""
    missing: list[str] = []
    if not prompt or not response:
        return missing

    prompt_words = len(prompt.split())
    response_words = len(response.split())

    has_headings_r = has_heading(response)
    has_list_r = has_list(response)
    has_tables_r = has_table(response)
    has_code_r = has_code_block(response)
    has_any_structure = has_headings_r or has_list_r or has_tables_r or has_code_r

    q_count = prompt.count("?")
    comma_count = prompt.count(",")
    is_compound_prompt = (q_count >= 2) or (prompt_words >= 30 and comma_count >= 2)

    if is_compound_prompt and not has_any_structure:
        missing.append(
            "a structured breakdown (sections, lists, or step-by-step form) "
            "for the compound request"
        )

    tech_like = (
        bool(re.search(r"[A-Za-z][\w-]*\.[A-Za-z]", prompt))
        or "`" in prompt
        or bool(re.search(r"\b[A-Z]{2,6}\b", prompt))
        or bool(re.search(r"\b[A-Z][a-z]+[A-Z][a-zA-Z0-9]*\b", prompt))
    )
    if prompt_words >= 15 and tech_like and response_words < prompt_words * 3:
        missing.append(
            "sufficient expansion for the technical scope of the question"
        )

    if q_count >= 2 and not has_list_r and not has_headings_r:
        missing.append("explicit treatment of each distinct sub-question")

    tokens = re.findall(r"\w+", response.lower())
    if len(tokens) >= 40:
        ttr = len(set(tokens)) / len(tokens)
        if ttr < 0.3:
            missing.append(
                "lexical and conceptual variety (the response repeats itself)"
            )

    if tech_like and prompt_words >= 15 and not has_code_r and response_words < 150:
        missing.append(
            "a concrete example, command, or code snippet to anchor the answer"
        )

    return missing
