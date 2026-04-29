# Troubleshooting

## The hook isn't firing

1. Check the hook is registered:
   ```bash
   cat ~/.claude/settings.json | grep adaptive-guard
   ```
   You should see the hook command.

2. Restart Claude Code. Hooks load at session start.

3. Verify Claude Code version is `v2.1.111` or later:
   ```bash
   claude --version
   ```

---

## The hook fires but never blocks

Likely the complexity threshold is filtering out your prompts. Check telemetry:

```bash
tail ~/.claude/telemetry/adaptive-guard.jsonl
```

Decisions of type `allow_simple_task` mean the prompt didn't meet the complexity bar. Lower `thresholds.complexity_min_score` in config, or switch to the `strict` profile:

```bash
./scripts/install.sh --profile strict
```

---

## The hook blocks too often

Opposite problem. Switch to the `lenient` profile:

```bash
./scripts/install.sh --profile lenient
```

Or edit `config/default.json` and raise `thresholds.depth_min_score`.

---

## Infinite loop (Claude keeps re-answering)

This should be rare. adaptive-guard defends against loops in two ways:

1. **`stop_hook_active` check**: if Claude Code sets this flag when it is already in a forced-continuation state, the guard exits immediately without re-blocking. (Note: this field is a defensive check — we read it defensively whether or not it is always populated for Stop events.)
2. **Single re-block per turn**: in practice, after one `decision: block` the guard's second invocation on the same turn will either see a deeper response (and allow) or the same superficial response (which can block once more, bounded by Claude Code's own mechanisms).

If you still experience a loop:

1. Immediately uninstall: `./scripts/uninstall.sh`
2. Open an issue with the telemetry log contents

---

## Python errors in telemetry

Check `~/.claude/telemetry/adaptive-guard.err.log` for Python exceptions. Common causes:

- `json.JSONDecodeError` on transcript → Claude Code may have updated the transcript format. Open an issue.
- `FileNotFoundError` → transcript path doesn't exist. Usually a Claude Code bug, not the guard's.

---

## How do I know it's working?

Run a complex prompt that you know the router tends to underestimate. Watch the Claude Code UI — if the guard blocks, Claude will continue the turn automatically with deeper analysis.

You can also tail the telemetry log:

```bash
tail -f ~/.claude/telemetry/adaptive-guard.jsonl
```

Decisions of type `block` mean the guard intervened.

---

## I want to see what scores my prompts are getting

Run the analyzer manually with a simulated hook payload:

```bash
echo '{
  "session_id": "manual",
  "transcript_path": "/path/to/transcript.jsonl",
  "last_assistant_message": "the response text here",
  "stop_hook_active": false
}' | python3 hooks/lib/analyze.py config/default.json
echo "Exit code: $?"
```

- **Empty stdout + exit 0** → allowed (response was acceptable OR task was trivial).
- **JSON stdout `{decision: "block", reason: ...}` + exit 0** → blocked. The `reason` is what Claude Code feeds back to the model.

For more detailed scoring per axis, write a small script that imports `complexity.py` and `depth.py` directly and prints each axis contribution.
