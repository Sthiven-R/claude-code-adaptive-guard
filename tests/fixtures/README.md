# Test fixtures

JSONL transcripts in the real Claude Code v2.1.x schema, used by `tests/test_depth.py`.

Each fixture is a minimal session representing one specific scenario:

| File | Scenario | Expected guard decision |
|---|---|---|
| `complex_prompt_shallow_response.jsonl` | User asks for architectural design with trade-offs; assistant answers in one sentence | **block** |
| `complex_prompt_deep_response.jsonl` | Same prompt; assistant answers with structured analysis, alternatives, risks, mitigations | **allow** |
| `trivial_prompt.jsonl` | User says "hola"; trivial response | **allow** (complexity filter kicks in) |
| `multi_turn_tool_results.jsonl` | Multi-turn with tool_result payloads as user entries; used to verify parser skips non-prompt user entries | parser should find the original text prompt |

## Schema (real)

All entries follow the actual Claude Code transcript JSONL format:

```json
{
  "type": "user" | "assistant" | ...,
  "message": {
    "role": "user" | "assistant",
    "content": "<string>" | [<blocks>]
  },
  "timestamp": "ISO8601",
  "uuid": "...",
  "sessionId": "..."
}
```

Assistant content blocks:

```json
[
  {"type": "thinking", "thinking": "..."},   // process — NOT counted as output
  {"type": "text",     "text": "..."},       // visible output — counted for depth
  {"type": "tool_use", "name": "...", ...},  // actions
  {"type": "tool_result", ...}               // tool returns
]
```

Fixtures are synthesized. No real conversation content ships with the repo.
