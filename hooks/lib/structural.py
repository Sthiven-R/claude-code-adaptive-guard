"""
structural.py - Language-agnostic scoring based on form, not words.

Used when the semantic embedder is unavailable (and as the blended
baseline when it is). All signals measure structure, statistics, or
syntax - never specific vocabulary. Works equally on English, Spanish,
Portuguese, French, etc.

Every scorer returns a (total_score, breakdown) tuple. The breakdown
makes the decision inspectable: users can see exactly which axis
contributed to a score and by how much. This is the ONLY way to
calibrate the tool to your own workflow.
"""
from __future__ import annotations

import re

# ---------------------------------------------------------------------------
# REGEXES (all language-agnostic by construction)
# ---------------------------------------------------------------------------

_HEADING_RE = re.compile(r"(?m)^#{2,6}\s")
_CODE_BLOCK_RE = re.compile(r"```[\s\S]*?```")
_INLINE_CODE_RE = re.compile(r"`[^`\n]+`")
_LIST_RE = re.compile(r"(?m)^\s*(?:[-*+]|\d+[.)])\s+")
_TABLE_RE = re.compile(r"(?m)^\s*\|[\s\-:|]+\|\s*$")

_CAMEL_CASE_RE = re.compile(r"\b[a-z][a-zA-Z0-9]*[A-Z][a-zA-Z0-9]*\b")
_SNAKE_CASE_RE = re.compile(r"\b[a-z][a-z0-9]+(?:_[a-z0-9]+)+\b")
_DOTTED_IDENT_RE = re.compile(r"\b[A-Za-z][\w-]*(?:\.[A-Za-z0-9][\w-]*)+\b")
_VERSION_RE = re.compile(r"\bv?\d+\.\d+(?:\.\d+)?\b")
_ACRONYM_RE = re.compile(r"\b[A-Z]{2,6}\b")
_MIXED_CAPS_RE = re.compile(r"\b[A-Z][a-z]+[A-Z][a-zA-Z0-9]*\b")
_PROPER_NOUN_MID_RE = re.compile(r"(?<=[,;:\-()\s])[A-Z][a-zA-Z]{2,}\b")
_NUMBER_UNIT_RE = re.compile(r"\b\d+(?:\.\d+)?[kKmMgGtTpP]?[bBs]?\b")

_PATH_RE = re.compile(r"(?:(?:/|[A-Za-z]:[\\/])[\w.\-]+(?:[\\/][\w.\-]+)+)")
_URL_RE = re.compile(r"https?://[^\s)]+")

_SENTENCE_SPLIT_RE = re.compile(r"[.!?]+(?:\s|$)")


def _count_matches(pattern: re.Pattern, text: str) -> int:
    return len(pattern.findall(text))


# ---------------------------------------------------------------------------
# Public structural predicates.
#
# Other modules (e.g. depth.py) need to know "does this text have a
# heading / list / table / code block?" without importing private
# compiled regexes. We expose small typed helpers so that depth.py does
# not reach into this module's _ABC_RE internals (audit finding M-10).
# ---------------------------------------------------------------------------


def has_heading(text: str) -> bool:
    return bool(_HEADING_RE.search(text))


def has_list(text: str) -> bool:
    return bool(_LIST_RE.search(text))


def has_table(text: str) -> bool:
    return bool(_TABLE_RE.search(text))


def has_code_block(text: str) -> bool:
    return bool(_CODE_BLOCK_RE.search(text))


def _unique_word_ratio(text: str) -> float:
    tokens = [t for t in re.findall(r"\w+", text.lower()) if t]
    if not tokens:
        return 0.0
    return len(set(tokens)) / len(tokens)


def _sentence_length_variance(text: str) -> float:
    parts = [p.strip() for p in _SENTENCE_SPLIT_RE.split(text) if p.strip()]
    if len(parts) < 2:
        return 0.0
    lengths = [len(p.split()) for p in parts]
    mean = sum(lengths) / len(lengths)
    var = sum((ln - mean) ** 2 for ln in lengths) / len(lengths)
    return var ** 0.5


# ---------------------------------------------------------------------------
# COMPLEXITY (prompt side)
# ---------------------------------------------------------------------------

def score_complexity_structural(prompt: str) -> tuple[int, dict]:
    """Return (score 0-100, breakdown dict).

    The breakdown is the SOURCE OF TRUTH for why a prompt scored what it
    scored. Each axis reports both the points contributed and the raw
    observed signals (token lists, counts, etc.) that justify those points.
    """
    breakdown = {
        "axes": {},
        "signals": {},
    }

    if not prompt:
        return 0, breakdown

    # Axis 1: length
    chars = len(prompt)
    length_pts = min(20, int(20 * chars / 400)) if chars >= 60 else 0
    breakdown["axes"]["length"] = length_pts
    breakdown["signals"]["char_count"] = chars
    breakdown["signals"]["word_count"] = len(prompt.split())

    # Axis 2: questions
    q_count = prompt.count("?")
    if q_count >= 1:
        questions_pts = min(15, q_count * 5 + max(0, (chars // 150) - 1) * 3)
    else:
        questions_pts = 0
    breakdown["axes"]["questions"] = questions_pts
    breakdown["signals"]["question_count"] = q_count

    # Axis 3: compound request structure
    sentence_count = max(
        1, len([s for s in _SENTENCE_SPLIT_RE.split(prompt) if s.strip()])
    )
    comma_count = prompt.count(",")
    semicolon_count = prompt.count(";")
    compound_pts = min(
        25, sentence_count * 5 + comma_count * 2 + semicolon_count * 3
    )
    breakdown["axes"]["compound"] = compound_pts
    breakdown["signals"]["sentence_count"] = sentence_count
    breakdown["signals"]["comma_count"] = comma_count
    breakdown["signals"]["semicolon_count"] = semicolon_count

    # Axis 4: technical token density.
    # PRIVACY: we only store COUNTS, never the matched strings. Paths,
    # URLs, inline code and (occasionally) dotted identifiers or acronyms
    # in prompts routinely contain user secrets: API tokens, file paths
    # under $HOME, credential snippets. Storing them verbatim in
    # telemetry would violate the promise that no prompt text leaves the
    # machine. Counts are enough to explain the score.
    tech_counts = {
        "camelCase": _count_matches(_CAMEL_CASE_RE, prompt),
        "snake_case": _count_matches(_SNAKE_CASE_RE, prompt),
        "dotted_ident": _count_matches(_DOTTED_IDENT_RE, prompt),
        "version": _count_matches(_VERSION_RE, prompt),
        "inline_code": _count_matches(_INLINE_CODE_RE, prompt),
        "acronyms": _count_matches(_ACRONYM_RE, prompt),
        "mixed_caps": _count_matches(_MIXED_CAPS_RE, prompt),
        "proper_nouns": _count_matches(_PROPER_NOUN_MID_RE, prompt),
        "numbers_with_units": min(3, _count_matches(_NUMBER_UNIT_RE, prompt)),
    }
    tech_hits = sum(tech_counts.values())
    tech_pts = min(30, tech_hits * 3)
    breakdown["axes"]["technical"] = tech_pts
    breakdown["signals"]["tech_token_counts"] = tech_counts
    breakdown["signals"]["tech_hit_total"] = tech_hits

    # Axis 5: external references. Counts only (paths and URLs often
    # contain secrets — never persisted).
    path_count = _count_matches(_PATH_RE, prompt)
    url_count = _count_matches(_URL_RE, prompt)
    code_blocks = _count_matches(_CODE_BLOCK_RE, prompt)
    ref_hits = path_count + url_count + code_blocks
    ref_pts = min(10, ref_hits * 5)
    breakdown["axes"]["external_refs"] = ref_pts
    breakdown["signals"]["path_count"] = path_count
    breakdown["signals"]["url_count"] = url_count
    breakdown["signals"]["code_block_count"] = code_blocks

    total = min(100, length_pts + questions_pts + compound_pts + tech_pts + ref_pts)
    breakdown["total"] = total
    return total, breakdown


# ---------------------------------------------------------------------------
# DEPTH (response side)
# ---------------------------------------------------------------------------

def score_depth_structural(response: str) -> tuple[int, dict]:
    """Return (score 0-100, breakdown dict) for the response."""
    breakdown = {
        "axes": {},
        "signals": {},
    }

    if not response:
        return 0, breakdown

    word_count = len(response.split())

    # Axis 1: length
    if word_count >= 60:
        length_pts = min(25, int(25 * word_count / 600))
    elif word_count >= 30:
        length_pts = 8
    else:
        length_pts = 0
    breakdown["axes"]["length"] = length_pts
    breakdown["signals"]["word_count"] = word_count
    breakdown["signals"]["char_count"] = len(response)

    # Axis 2: headings
    heading_count = _count_matches(_HEADING_RE, response)
    heading_pts = min(20, heading_count * 4)
    breakdown["axes"]["headings"] = heading_pts
    breakdown["signals"]["heading_count"] = heading_count

    # Axis 3: list/table structure
    list_lines = _count_matches(_LIST_RE, response)
    table_sep = _count_matches(_TABLE_RE, response)
    structure_pts = min(15, list_lines + table_sep * 5)
    breakdown["axes"]["lists_tables"] = structure_pts
    breakdown["signals"]["list_line_count"] = list_lines
    breakdown["signals"]["table_row_count"] = table_sep

    # Axis 4: code blocks
    code_blocks = _count_matches(_CODE_BLOCK_RE, response)
    inline_code = _count_matches(_INLINE_CODE_RE, response)
    code_pts = min(10, code_blocks * 5 + inline_code)
    breakdown["axes"]["code"] = code_pts
    breakdown["signals"]["code_block_count"] = code_blocks
    breakdown["signals"]["inline_code_count"] = inline_code

    # Axis 5: lexical diversity
    ttr = _unique_word_ratio(response)
    if ttr >= 0.45:
        ttr_pts = 15
    elif ttr >= 0.35:
        ttr_pts = 10
    elif ttr >= 0.25:
        ttr_pts = 5
    else:
        ttr_pts = 0
    breakdown["axes"]["lexical_diversity"] = ttr_pts
    breakdown["signals"]["type_token_ratio"] = round(ttr, 3)

    # Axis 6: sentence-length variance
    variance = _sentence_length_variance(response)
    if variance >= 10:
        var_pts = 15
    elif variance >= 5:
        var_pts = 8
    else:
        var_pts = 0
    breakdown["axes"]["sentence_variance"] = var_pts
    breakdown["signals"]["sentence_length_stddev"] = round(variance, 2)

    total = min(
        100,
        length_pts + heading_pts + structure_pts + code_pts + ttr_pts + var_pts,
    )
    breakdown["total"] = total
    return total, breakdown
