# Telemetry

adaptive-guard writes one JSONL line per decision to `~/.claude/telemetry/adaptive-guard.jsonl`.

## What is logged

Only anonymized metrics:

```json
{
  "ts": "2026-04-17T10:30:00Z",
  "session_id": "abc12345",
  "profile": "balanced",
  "complexity": 67,
  "depth": 28,
  "decision": "block",
  "missing_count": 2
}
```

- `session_id` is truncated to 8 chars (enough for local grouping, not enough to uniquely identify)
- `complexity` and `depth` are integer scores, not content
- `decision` is one of: `allow_simple_task`, `allow_deep_response`, `block`

## What is NEVER logged

- Prompt content
- Response content
- Transcript content
- File paths from user's machine (beyond the guard's own log path)
- API keys, tokens, secrets

## Disabling

In `config/default.json`:

```json
{
  "telemetry_enabled": false
}
```

Or set the env var `ADAPTIVE_GUARD_CONFIG` to a config file that has `telemetry_enabled: false`.

## Rotation

The log is capped at ~10000 entries (~2MB). When exceeded, the guard keeps only the most recent 10000.

## Where to see it

```bash
tail -f ~/.claude/telemetry/adaptive-guard.jsonl
```
