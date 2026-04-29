"""
output.py - Format and emit Stop-hook decisions.

Claude Code's Stop-hook JSON schema accepts:
  - decision: "approve" | "block"
  - reason:   string (fed back to Claude on block)
  - continue, suppressOutput, stopReason, systemMessage (optional)

hookSpecificOutput is valid ONLY for PreToolUse / UserPromptSubmit /
PostToolUse events. Including it on Stop causes schema validation to
fail and Claude Code silently discards the decision. Verified against
the live CLI schema error (Apr 2026).
"""
from __future__ import annotations

import json

_MISSING_ASPECTS_PLACEHOLDER = "{missing_aspects}"
_REASON_MAX_CHARS = 9000  # hook output cap is 10k


def build_block_reason(template: str, missing: list[str]) -> str:
    """Format the blocking message using the configured template.

    Security: uses `str.replace()`, not `str.format()`. Python's format
    mini-language allows attribute access (`{x.__class__.__mro__}`),
    which combined with an attacker-controlled template would enable
    code-access attacks. `.replace()` treats the placeholder as a
    literal substring.
    """
    if missing:
        bullets = "\n  - " + "\n  - ".join(missing)
    else:
        bullets = "\n  - general depth insufficient for the complexity of the task"
    return template.replace(_MISSING_ASPECTS_PLACEHOLDER, bullets)


def emit_block_decision(reason: str) -> None:
    """Emit a Stop-hook block decision to stdout (exit 0 is still used).

    Output shape contains only the keys allowed by the Stop-hook schema.
    """
    reason_capped = reason[:_REASON_MAX_CHARS]
    output = {
        "decision": "block",
        "reason": reason_capped,
    }
    print(json.dumps(output))
