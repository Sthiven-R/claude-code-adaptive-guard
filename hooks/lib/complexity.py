"""
complexity.py - Score user prompt complexity with full explainability.

Returns a tuple `(total_score, breakdown)` where `breakdown` details
which axis contributed to that score.

Dual-path:
  - Structural: always active, language-agnostic, zero deps.
  - Semantic:   opt-in via fastembed; blended with structural.

The embedder module is imported LAZILY inside score_complexity_explained
so that hook invocations that only need the structural path do not pay
the import cost. fastembed pulls in onnxruntime + tokenizers (~50-200 ms
of import time even when cached).
"""
from __future__ import annotations

from anchors import COMPLEX_PROMPT_ANCHORS, SIMPLE_PROMPT_ANCHORS
from structural import score_complexity_structural


def score_complexity_explained(prompt: str) -> tuple[int, dict]:
    """Return (total_score, breakdown) for a prompt.

    Shape of the breakdown:
      {
        "total": int,
        "structural": int,
        "semantic": int | None,
        "blend_weights": {"semantic": float, "structural": float} | None,
        "axes": {axis_name: points},
        "signals": {signal_name: raw_value},
      }
    """
    if not prompt:
        return 0, {
            "total": 0,
            "structural": 0,
            "semantic": None,
            "blend_weights": None,
            "axes": {},
            "signals": {},
        }

    structural_score, breakdown = score_complexity_structural(prompt)
    breakdown["structural"] = structural_score
    breakdown["semantic"] = None
    breakdown["blend_weights"] = None

    # Lazy import: the embedder module itself imports fastembed which is
    # heavy. If fastembed isn't installed, is_available() returns False
    # without any import cost.
    from embedder import is_available, score_affinity

    if not is_available():
        breakdown["total"] = structural_score
        return structural_score, breakdown

    try:
        semantic = score_affinity(
            prompt,
            positive_anchors=COMPLEX_PROMPT_ANCHORS,
            negative_anchors=SIMPLE_PROMPT_ANCHORS,
        )
    except Exception:
        semantic = None

    if semantic is None:
        breakdown["total"] = structural_score
        return structural_score, breakdown

    w_sem, w_str = 0.65, 0.35
    blended = int(round(semantic * w_sem + structural_score * w_str))
    blended = max(0, min(100, blended))

    breakdown["semantic"] = semantic
    breakdown["blend_weights"] = {"semantic": w_sem, "structural": w_str}
    breakdown["total"] = blended
    return blended, breakdown
