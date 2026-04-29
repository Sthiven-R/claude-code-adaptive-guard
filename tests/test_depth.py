"""
Tests for adaptive-guard.

Covers:
  - Unit: complexity and depth scoring (structural and blended)
  - Unit: transcript parsing and block extraction
  - Unit: detect_missing_aspects behaviour on generic structural gaps
  - Integration: parsing against real JSONL fixtures
  - Integration: end-to-end analyze() via subprocess
  - Regression: all findings from the hostile audit

Run:
  python tests/test_depth.py
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

# Make hooks/lib importable
REPO_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(REPO_ROOT / "hooks" / "lib"))

from config import load_config  # noqa: E402
from transcript import (  # noqa: E402
    extract_last_assistant_text,
    extract_last_user_prompt,
    extract_text_from_blocks,
)
from complexity import score_complexity_explained  # noqa: E402
from depth import detect_missing_aspects, score_depth_explained  # noqa: E402


# Thin helpers so tests can use the single-int form without the breakdown.
# The production code uses the _explained form directly.
def score_complexity(prompt: str, _cfg=None) -> int:
    return score_complexity_explained(prompt)[0]


def score_depth(response: str, _cfg=None) -> int:
    return score_depth_explained(response)[0]
from structural import (  # noqa: E402
    score_complexity_structural,
    score_depth_structural,
)

FIXTURES_DIR = REPO_ROOT / "tests" / "fixtures"
CONFIG_PATH = REPO_ROOT / "config" / "default.json"
CFG = load_config(str(CONFIG_PATH))


# ---------------------------------------------------------------------------
# UNIT: structural scoring (always active, no embeddings required)
# ---------------------------------------------------------------------------

def test_structural_trivial_prompt_low_complexity():
    s1, _ = score_complexity_structural("hi")
    s2, _ = score_complexity_structural("What time is it?")
    assert s1 < 20
    assert s2 < 25


def test_structural_technical_prompt_high_complexity():
    prompt = (
        "I have a production backend running Node.js with PostgreSQL that "
        "needs to scale. Compare vertical and horizontal sharding strategies, "
        "evaluate the operational risks of each, and recommend the right "
        "path given a workload of ~50k QPS."
    )
    s, _ = score_complexity_structural(prompt)
    assert s >= 40, f"expected >= 40, got {s}"


def test_structural_design_prompt_high_complexity_spanish():
    prompt = (
        "Diseña una arquitectura modular para un sistema de procesamiento "
        "de eventos en tiempo real. Analiza trade-offs entre Kafka, Redis "
        "Streams y NATS. Considera los riesgos y propón una implementación "
        "con edge cases, v2.1 compatibility y Node.js interop."
    )
    s, _ = score_complexity_structural(prompt)
    assert s >= 40, f"expected >= 40, got {s}"


def test_structural_short_response_low_depth():
    s, _ = score_depth_structural("Use Kafka. It's best.")
    assert s < 30


def test_structural_structured_response_high_depth():
    response = """## Analysis

### Option A
- **Pros**: throughput
- **Cons**: complexity

### Option B
- **Pros**: simplicity
- **Cons**: durability

## Recommendation

We choose A because throughput dominates. Still, if load drops the
calculus changes, and B becomes preferable. The failure modes we must
plan for are back-pressure, poison messages, and schema drift.

## Risks identified

1. Back-pressure - mitigation: autoscale
2. Poison messages - mitigation: DLQ
3. Schema drift - mitigation: registry
"""
    s, _ = score_depth_structural(response)
    assert s >= 40, f"expected >= 40, got {s}"


def test_structural_returns_breakdown():
    """Regression: every structural call must return (score, breakdown)."""
    score, breakdown = score_complexity_structural("Design X and Y with Node.js.")
    assert isinstance(breakdown, dict)
    assert "axes" in breakdown
    assert "signals" in breakdown
    assert breakdown.get("total") == score

    score, breakdown = score_depth_structural("## Section\n- bullet\n- bullet\nMore prose.")
    assert isinstance(breakdown, dict)
    assert "axes" in breakdown


def test_explained_scoring_returns_breakdown_with_total():
    """The _explained form always includes `total` matching the score."""
    prompt = "Analyze trade-offs and evaluate risks of option A vs B."
    score, breakdown = score_complexity_explained(prompt)
    assert breakdown.get("total") == score

    response = "## A\n- pro\n- con\n## B\n- pro"
    dscore, dbreakdown = score_depth_explained(response)
    assert dbreakdown.get("total") == dscore


def test_anchors_are_non_empty_and_substantive():
    """Audit finding LOW-7: an empty or near-empty anchor silently
    skews every semantic score toward 0. Validate at import time."""
    import anchors as A
    all_anchors = (
        A.SIMPLE_PROMPT_ANCHORS
        + A.COMPLEX_PROMPT_ANCHORS
        + A.SHALLOW_RESPONSE_ANCHORS
        + A.DEEP_RESPONSE_ANCHORS
    )
    for a in all_anchors:
        assert isinstance(a, str) and a.strip(), f"empty anchor: {a!r}"
        assert len(a.strip()) >= 3, f"anchor too short: {a!r}"
    # Ensure anchor sets are meaningfully populated (not a one-item accident).
    assert len(A.SIMPLE_PROMPT_ANCHORS) >= 5
    assert len(A.COMPLEX_PROMPT_ANCHORS) >= 5
    assert len(A.SHALLOW_RESPONSE_ANCHORS) >= 5
    assert len(A.DEEP_RESPONSE_ANCHORS) >= 3


def test_privacy_no_prompt_text_in_breakdown_signals():
    """Audit finding HIGH-security: prompt substrings (paths, URLs,
    inline code) must never end up in telemetry. This test sends a
    prompt with very distinctive content and asserts NONE of it is
    present in the breakdown signals object."""
    secret_path = "/home/user/.aws/super-secret-creds-abc123"
    secret_url = "https://api.corp.example/?token=sk-live-XYZZZZZ"
    secret_inline = "`API_KEY=sk-ant-secret-DONTLEAK`"
    prompt = (
        f"Debug my script at {secret_path} using {secret_url} "
        f"and call {secret_inline} please."
    )
    _, breakdown = score_complexity_explained(prompt)
    # Recursively serialize and assert no secret substring appears.
    import json as _json
    dumped = _json.dumps(breakdown)
    assert secret_path not in dumped, "path leaked into breakdown"
    assert secret_url not in dumped, "URL leaked into breakdown"
    assert "sk-ant-secret-DONTLEAK" not in dumped, "inline code leaked"
    assert "sk-live-XYZZZZZ" not in dumped, "URL token leaked"


# ---------------------------------------------------------------------------
# UNIT: wrapper scoring (may use semantic path if available)
# ---------------------------------------------------------------------------

def test_trivial_prompt_low_complexity():
    assert score_complexity("hi", CFG) < 30


def test_complex_prompt_high_complexity():
    prompt = (
        "Design a modular architecture for a real-time event processing "
        "system. Analyze trade-offs between Kafka, Redis Streams, and NATS. "
        "Evaluate the main risks and propose an implementation."
    )
    assert score_complexity(prompt, CFG) >= 40


def test_short_response_low_depth():
    assert score_depth("Use Kafka. It's best.", CFG) < 40


def test_structured_response_high_depth():
    response = """## Analysis of alternatives

### Option A: Kafka
- **Pros**: high throughput
- **Cons**: operational complexity

### Option B: Redis Streams
- **Pros**: latency
- **Cons**: durability limits

## Recommendation

We pick Kafka because volume justifies it. Still, there are trade-offs
worth stating, and the failure modes are concrete.

## Risks

1. Back-pressure - mitigation: autoscale
2. Poison messages - mitigation: DLQ
3. Schema drift - mitigation: registry
"""
    assert score_depth(response, CFG) >= 40


# ---------------------------------------------------------------------------
# UNIT: missing-aspect detection (structural, language-agnostic)
# ---------------------------------------------------------------------------

def test_missing_detects_no_structure_on_compound_prompt():
    prompt = (
        "Compare Postgres vs MongoDB for this use case. What are the "
        "trade-offs? What are the risks?"
    )
    response = "Postgres is better."
    missing = detect_missing_aspects(prompt, response, CFG)
    assert any("breakdown" in m.lower() or "structure" in m.lower() or "sub-question" in m.lower() for m in missing), (
        f"expected missing structure hint, got {missing}"
    )


def test_missing_detects_shallow_response_to_technical_prompt():
    prompt = (
        "I have a Node.js service with PostgreSQL that handles 10k QPS. "
        "The p99 latency doubled after last deploy. Investigate."
    )
    response = "Add an index."
    missing = detect_missing_aspects(prompt, response, CFG)
    assert len(missing) >= 1, f"expected at least one missing aspect, got {missing}"


def test_missing_empty_on_well_structured_response():
    prompt = "Compare option A and option B, and recommend one."
    response = """## Analysis

### Option A
- Pros: simple
- Cons: slow

### Option B
- Pros: fast
- Cons: complex

## Recommendation

Choose B because speed dominates.
"""
    missing = detect_missing_aspects(prompt, response, CFG)
    assert missing == [] or len(missing) <= 1


# ---------------------------------------------------------------------------
# UNIT: block extraction
# ---------------------------------------------------------------------------

def test_extract_text_from_string_content():
    assert extract_text_from_blocks("hello world") == "hello world"


def test_extract_text_from_block_list():
    blocks = [
        {"type": "thinking", "thinking": "internal reasoning"},
        {"type": "text", "text": "visible part 1"},
        {"type": "tool_use", "id": "x", "name": "Read", "input": {}},
        {"type": "text", "text": "visible part 2"},
    ]
    result = extract_text_from_blocks(blocks)
    assert "visible part 1" in result
    assert "visible part 2" in result
    assert "internal reasoning" not in result


def test_extract_text_from_empty_content():
    assert extract_text_from_blocks(None) == ""
    assert extract_text_from_blocks([]) == ""
    assert extract_text_from_blocks("") == ""


def test_extract_text_ignores_non_dict_blocks():
    blocks = ["raw string", {"type": "text", "text": "real text"}, 42]
    assert extract_text_from_blocks(blocks) == "real text"


# ---------------------------------------------------------------------------
# INTEGRATION: transcript parsing against fixtures
# ---------------------------------------------------------------------------

def test_parse_complex_shallow_fixture():
    fixture = FIXTURES_DIR / "complex_prompt_shallow_response.jsonl"
    user = extract_last_user_prompt(str(fixture))
    assistant = extract_last_assistant_text(str(fixture))
    assert "architecture" in user.lower()
    assert "Kafka" in assistant and len(assistant) < 200


def test_parse_complex_deep_fixture():
    fixture = FIXTURES_DIR / "complex_prompt_deep_response.jsonl"
    user = extract_last_user_prompt(str(fixture))
    assistant = extract_last_assistant_text(str(fixture))
    assert "architecture" in user.lower()
    assert len(assistant) > 800
    assert "##" in assistant
    assert "enumerate the three alternatives" not in assistant, (
        "thinking block leaked into visible output"
    )


def test_parse_trivial_fixture():
    fixture = FIXTURES_DIR / "trivial_prompt.jsonl"
    assert extract_last_user_prompt(str(fixture)) == "hi"


def test_parse_skips_tool_result_user_entries():
    fixture = FIXTURES_DIR / "multi_turn_tool_results.jsonl"
    user = extract_last_user_prompt(str(fixture))
    assert user == "Analyze this file and explain its architecture."


def test_parse_technical_fixture():
    fixture = FIXTURES_DIR / "technical_prompt_shallow_response.jsonl"
    user = extract_last_user_prompt(str(fixture))
    assistant = extract_last_assistant_text(str(fixture))
    assert "Node.js" in user
    assert "PostgreSQL" in user
    assert "sharding" in user
    # Critical regression: this prompt must score complex under new scoring
    assert score_complexity(user, CFG) >= 40, (
        f"technical prompt failed to register as complex, got "
        f"{score_complexity(user, CFG)}"
    )


def test_parse_missing_file_returns_empty():
    assert extract_last_user_prompt("/nonexistent/path.jsonl") == ""
    assert extract_last_assistant_text("/nonexistent/path.jsonl") == ""


# ---------------------------------------------------------------------------
# END-TO-END: subprocess invocation of analyze.py
# ---------------------------------------------------------------------------

def run_analyze(hook_input: dict) -> tuple[int, str, str]:
    result = subprocess.run(
        [sys.executable, str(REPO_ROOT / "hooks" / "lib" / "analyze.py"), str(CONFIG_PATH)],
        input=json.dumps(hook_input),
        capture_output=True,
        text=True,
        timeout=60,
    )
    return result.returncode, result.stdout, result.stderr


def test_e2e_shallow_response_triggers_block():
    fixture = FIXTURES_DIR / "complex_prompt_shallow_response.jsonl"
    hook_input = {
        "session_id": "test-session",
        "transcript_path": str(fixture),
        "last_assistant_message": "Use Kafka. It's the best option.",
        "stop_hook_active": False,
    }
    code, stdout, _ = run_analyze(hook_input)
    assert code == 0
    assert stdout.strip(), "expected JSON output when blocking"
    decision = json.loads(stdout)
    assert decision.get("decision") == "block"
    assert "reason" in decision and len(decision["reason"]) > 20
    # Stop-hook schema: hookSpecificOutput is NOT valid for Stop events.
    assert "hookSpecificOutput" not in decision
    allowed_keys = {
        "continue", "suppressOutput", "stopReason",
        "decision", "reason", "systemMessage", "permissionDecision",
    }
    assert set(decision.keys()).issubset(allowed_keys), (
        f"unknown fields: {set(decision.keys()) - allowed_keys}"
    )


def test_e2e_technical_prompt_triggers_block():
    """Regression: the Node.js + PostgreSQL sharding prompt scored 33 under
    the old keyword-based scorer (below threshold). Must now score >= 40.
    """
    fixture = FIXTURES_DIR / "technical_prompt_shallow_response.jsonl"
    hook_input = {
        "session_id": "test-session",
        "transcript_path": str(fixture),
        "last_assistant_message": "Try horizontal sharding first. Consider Citus.",
        "stop_hook_active": False,
    }
    code, stdout, _ = run_analyze(hook_input)
    assert code == 0
    assert stdout.strip(), "technical compound prompt must trigger block"
    decision = json.loads(stdout)
    assert decision.get("decision") == "block"


def test_e2e_deep_response_allows_stop():
    fixture = FIXTURES_DIR / "complex_prompt_deep_response.jsonl"
    deep_text = extract_last_assistant_text(str(fixture))
    hook_input = {
        "session_id": "test-session",
        "transcript_path": str(fixture),
        "last_assistant_message": deep_text,
        "stop_hook_active": False,
    }
    code, stdout, _ = run_analyze(hook_input)
    assert code == 0
    assert not stdout.strip(), f"expected empty stdout, got {stdout[:200]!r}"


def test_e2e_trivial_prompt_allows_stop():
    fixture = FIXTURES_DIR / "trivial_prompt.jsonl"
    hook_input = {
        "session_id": "test-session",
        "transcript_path": str(fixture),
        "last_assistant_message": "Hi. How can I help?",
        "stop_hook_active": False,
    }
    code, stdout, _ = run_analyze(hook_input)
    assert code == 0
    assert not stdout.strip()


def test_e2e_stop_hook_active_skips_everything():
    fixture = FIXTURES_DIR / "complex_prompt_shallow_response.jsonl"
    hook_input = {
        "session_id": "test-session",
        "transcript_path": str(fixture),
        "last_assistant_message": "Use Kafka.",
        "stop_hook_active": True,
    }
    code, stdout, _ = run_analyze(hook_input)
    assert code == 0
    assert not stdout.strip()


def test_e2e_malformed_input_fails_open():
    result = subprocess.run(
        [sys.executable, str(REPO_ROOT / "hooks" / "lib" / "analyze.py"), str(CONFIG_PATH)],
        input="{not valid json",
        capture_output=True,
        text=True,
        timeout=30,
    )
    assert result.returncode == 0
    assert not result.stdout.strip()


def test_e2e_missing_transcript_fails_open():
    hook_input = {
        "session_id": "test-session",
        "transcript_path": "/nonexistent/path.jsonl",
        "last_assistant_message": "Use Kafka.",
        "stop_hook_active": False,
    }
    code, stdout, _ = run_analyze(hook_input)
    assert code == 0
    assert not stdout.strip()


def test_e2e_missing_required_fields_fails_open():
    hook_input = {"session_id": "partial"}
    code, stdout, _ = run_analyze(hook_input)
    assert code == 0
    assert not stdout.strip()


def test_e2e_unexpected_exception_fails_open():
    """Fail-open contract: any uncaught exception inside analyze.main()
    must result in exit 0 + empty stdout. The guard is a quality tool;
    it must never block the user from finishing a session because of
    its own bugs.

    We provoke an uncaught path by giving analyze a structurally valid
    JSON config that is missing the required `thresholds` key, which
    leads to a KeyError at the threshold lookup. The outer
    try/except in analyze.main() must absorb it.
    """
    import os as _os
    import tempfile
    bad_cfg = {
        "profile": "broken-on-purpose",
        "telemetry_enabled": False,
        "blocking_message_template": "x",
        # thresholds is intentionally missing.
    }
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".json", delete=False, encoding="utf-8"
    ) as f:
        json.dump(bad_cfg, f)
        bad_cfg_path = f.name

    try:
        env = dict(_os.environ)
        env["ADAPTIVE_GUARD_TELEMETRY_DISABLED"] = "1"
        result = subprocess.run(
            [
                sys.executable,
                str(REPO_ROOT / "hooks" / "lib" / "analyze.py"),
                bad_cfg_path,
            ],
            input=json.dumps({
                "session_id": "fail-open-regression",
                "transcript_path": str(
                    FIXTURES_DIR / "complex_prompt_shallow_response.jsonl"
                ),
                "last_assistant_message": "Use Kafka.",
                "stop_hook_active": False,
            }),
            capture_output=True,
            text=True,
            timeout=30,
            env=env,
        )
        assert result.returncode == 0, (
            f"fail-open contract violated: exit {result.returncode}, "
            f"stderr={result.stderr!r}"
        )
        assert not result.stdout.strip(), (
            f"fail-open must produce empty stdout, got {result.stdout[:200]!r}"
        )
    finally:
        _os.unlink(bad_cfg_path)


# ---------------------------------------------------------------------------
# REGRESSION: audit findings and prior bugs
# ---------------------------------------------------------------------------

def test_profile_strict_deep_merge_preserves_base_keys():
    strict = load_config(str(REPO_ROOT / "config" / "profiles" / "strict.json"))
    assert "thresholds" in strict
    assert "blocking_message_template" in strict
    assert strict["thresholds"]["complexity_min_score"] == 30
    assert strict["thresholds"]["depth_min_score"] == 55


def test_profile_lenient_deep_merge_preserves_base_keys():
    lenient = load_config(str(REPO_ROOT / "config" / "profiles" / "lenient.json"))
    assert "thresholds" in lenient
    assert lenient["thresholds"]["complexity_min_score"] == 55
    assert lenient["thresholds"]["depth_min_score"] == 30


def test_extends_path_traversal_rejected():
    import tempfile
    malicious = {"extends": "../../../../etc/passwd", "profile": "evil"}
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".json", delete=False, encoding="utf-8"
    ) as f:
        json.dump(malicious, f)
        bad_path = f.name
    try:
        try:
            load_config(bad_path)
        except ValueError:
            return
        except Exception as e:
            assert False, f"expected ValueError, got {type(e).__name__}: {e}"
        assert False, "traversal must raise"
    finally:
        import os as _os
        _os.unlink(bad_path)


def test_template_replace_not_format():
    """build_block_reason must use str.replace, not str.format."""
    from output import build_block_reason
    evil_template = "{missing_aspects.__class__.__mro__[1].__subclasses__}"
    result = build_block_reason(evil_template, ["some aspect"])
    # replace leaves unknown placeholders intact; no Python format evaluation
    assert "__class__" in result


def test_template_placeholder_replacement_works():
    from output import build_block_reason
    template = "Header\n{missing_aspects}\nFooter"
    result = build_block_reason(template, ["alternatives", "risks"])
    assert "Header" in result
    assert "alternatives" in result
    assert "risks" in result
    assert "Footer" in result


def test_multimodal_user_prompt_extracted():
    import tempfile
    events = [
        {
            "type": "user",
            "message": {
                "role": "user",
                "content": [
                    {"type": "image", "source": {"type": "base64", "data": "..."}},
                    {"type": "text", "text": "Analyze this architecture screenshot."},
                ],
            },
        },
        {
            "type": "assistant",
            "message": {"role": "assistant", "content": [{"type": "text", "text": "OK."}]},
        },
    ]
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".jsonl", delete=False, encoding="utf-8"
    ) as f:
        for e in events:
            f.write(json.dumps(e) + "\n")
        path = f.name
    try:
        user = extract_last_user_prompt(path)
        assert "Analyze this architecture" in user
    finally:
        import os as _os
        _os.unlink(path)


def test_large_transcript_bounded():
    import tempfile
    valid = {
        "type": "user",
        "message": {"role": "user", "content": "Real prompt at end"},
    }
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".jsonl", delete=False, encoding="utf-8"
    ) as f:
        garbage = "{invalid json garbage " + "X" * 1000 + "}\n"
        for _ in range(4000):
            f.write(garbage)
        f.write(json.dumps(valid) + "\n")
        path = f.name
    try:
        user = extract_last_user_prompt(path)
        # Either returns the real prompt (in tail window) or "" (pushed out).
        assert user in ("", "Real prompt at end")
    finally:
        import os as _os
        _os.unlink(path)


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

def run_all():
    # Test isolation: subprocess invocations of analyze.py would otherwise
    # write to the real ~/.claude/telemetry file. Ensure all child
    # processes see the disable flag, even if the user forgot to set it.
    import os as _os
    _os.environ.setdefault("ADAPTIVE_GUARD_TELEMETRY_DISABLED", "1")
    tests = [
        # Structural
        test_structural_trivial_prompt_low_complexity,
        test_structural_technical_prompt_high_complexity,
        test_structural_design_prompt_high_complexity_spanish,
        test_structural_short_response_low_depth,
        test_structural_structured_response_high_depth,
        test_structural_returns_breakdown,
        test_explained_scoring_returns_breakdown_with_total,
        test_anchors_are_non_empty_and_substantive,
        test_privacy_no_prompt_text_in_breakdown_signals,
        # Wrapper
        test_trivial_prompt_low_complexity,
        test_complex_prompt_high_complexity,
        test_short_response_low_depth,
        test_structured_response_high_depth,
        # Missing-aspect
        test_missing_detects_no_structure_on_compound_prompt,
        test_missing_detects_shallow_response_to_technical_prompt,
        test_missing_empty_on_well_structured_response,
        # Block extraction
        test_extract_text_from_string_content,
        test_extract_text_from_block_list,
        test_extract_text_from_empty_content,
        test_extract_text_ignores_non_dict_blocks,
        # Parsing
        test_parse_complex_shallow_fixture,
        test_parse_complex_deep_fixture,
        test_parse_trivial_fixture,
        test_parse_skips_tool_result_user_entries,
        test_parse_technical_fixture,
        test_parse_missing_file_returns_empty,
        # E2E
        test_e2e_shallow_response_triggers_block,
        test_e2e_technical_prompt_triggers_block,
        test_e2e_deep_response_allows_stop,
        test_e2e_trivial_prompt_allows_stop,
        test_e2e_stop_hook_active_skips_everything,
        test_e2e_malformed_input_fails_open,
        test_e2e_missing_transcript_fails_open,
        test_e2e_missing_required_fields_fails_open,
        test_e2e_unexpected_exception_fails_open,
        # Regression
        test_profile_strict_deep_merge_preserves_base_keys,
        test_profile_lenient_deep_merge_preserves_base_keys,
        test_extends_path_traversal_rejected,
        test_template_replace_not_format,
        test_template_placeholder_replacement_works,
        test_multimodal_user_prompt_extracted,
        test_large_transcript_bounded,
    ]
    passed = 0
    failed = 0
    for t in tests:
        try:
            t()
            print(f"PASS: {t.__name__}")
            passed += 1
        except AssertionError as e:
            print(f"FAIL: {t.__name__} -- {e}")
            failed += 1
        except Exception as e:
            print(f"ERROR: {t.__name__} -- {type(e).__name__}: {e}")
            failed += 1
    print(f"\n{passed}/{passed + failed} passed")
    return failed == 0


if __name__ == "__main__":
    sys.exit(0 if run_all() else 1)
