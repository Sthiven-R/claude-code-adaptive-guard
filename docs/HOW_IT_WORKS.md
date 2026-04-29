# How adaptive-guard works

## The problem

Claude Opus 4.7 uses **adaptive thinking**: a router inside the model decides, per turn, whether to engage extended reasoning or reply directly.

The router is imperfect. In practice it makes two systematic mistakes:

1. **Underthinking on complex tasks.** The router sometimes classifies a genuinely hard task as easy, skips thinking tokens, and emits a quick surface-level answer.
2. **Overthinking on simple tasks.** The inverse — trivial questions trigger long deliberations that waste tokens.

Research (arXiv:2501.18585, "Thoughts Are All Over the Place") documents both behaviors in o1-like reasoning models. Claude 4.7 is not immune.

In Opus 4.7, adaptive thinking is the **only** supported mode. You cannot disable it or set a manual budget via the API. The best you can do from outside the model is:

- Set the effort ceiling as high as possible (`effortLevel: max` in `settings.json`)
- Intercept the turn externally when the output looks shallow and give the model feedback

adaptive-guard is the second lever.

---

## The mechanism

Claude Code exposes lifecycle hooks. **Stop** fires every time Claude finishes a response.

The hook receives JSON on stdin:

```json
{
  "session_id": "<uuid>",
  "transcript_path": "<absolute path to session JSONL>",
  "cwd": "<working dir>",
  "permission_mode": "default|plan|acceptEdits|auto|dontAsk|bypassPermissions",
  "hook_event_name": "Stop",
  "last_assistant_message": "<text of Claude's final response>",
  "stop_hook_active": false
}
```

To force Claude to continue with additional context, the hook emits JSON on stdout with `exit 0`:

```json
{
  "decision": "block",
  "reason": "<explanation fed back to Claude>"
}
```

That is the entire output. **`hookSpecificOutput` is NOT a valid field for Stop events** — it only applies to `PreToolUse`, `UserPromptSubmit`, and `PostToolUse`. Including it on Stop causes Claude Code's schema validator to reject the decision and the block is silently discarded. The guard's `output.py` enforces the schema strictly (verified against the live CLI schema, April 2026).

adaptive-guard:

1. Reads `last_assistant_message` directly from the hook input (no transcript read needed for the response).
2. Reads `transcript_path` only to recover the **prior user prompt** (the hook input does not include it).
3. Scores the prompt with `complexity.py`.
4. If the prompt is below the complexity threshold, allows the stop with no further work.
5. Otherwise scores the response with `depth.py`.
6. If depth meets the threshold, allows the stop.
7. If depth is below the threshold, calls `detect_missing_aspects` to enumerate concrete structural gaps and emits the `block` decision with those gaps in the `reason`.

Claude Code surfaces `reason` back to the model on the forced continuation. The adaptive router sees an explicit "your previous answer was insufficient — here is why" signal — stronger than prompt hints because it is a closed feedback loop between turns.

### Why JSON stdout instead of `exit 2 + stderr`

The `exit 2` path is a legacy blocking mechanism. The JSON-on-stdout path (with `decision: "block"`) is the modern contract recommended in the Claude Code hooks reference and integrates with the UI.

### Transcript schema (for the prior user prompt)

The transcript is JSONL. Each line is an event:

```json
{
  "type": "user" | "assistant" | "system" | "queue-operation" | "attachment" | ...,
  "message": {
    "role": "user" | "assistant",
    "content": "<string>" | [<blocks>]
  },
  "uuid": "...",
  "timestamp": "ISO8601",
  "sessionId": "..."
}
```

For user entries, `message.content` is a string for typed prompts and a list (containing `tool_result` blocks) when it's a tool return. adaptive-guard walks backwards skipping list-content user entries whose blocks are tool returns, so `tool_result` payloads are not mistaken for the user's typed prompt. Multi-modal prompts (text + image blocks) are supported: only the `type: "text"` blocks contribute to the scored text.

The transcript reader reads the file's tail in 64 KB reverse chunks, with a 20 MB total cap and a 16 MB per-line cap. Long sessions only pay for the bytes actually consumed.

---

## Scoring at a glance

Every score is `0–100`. The decision flow is:

```
complexity < complexity_min_score   -> allow (decision: allow_simple_task)
depth      >= depth_min_score       -> allow (decision: allow_deep_response)
otherwise                           -> block (decision: block)
```

Two scoring paths combine for each side:

- **Structural** (always on, zero deps): measures FORM — length, markdown structure, regex token patterns (camelCase, snake_case, dotted identifiers, acronyms, etc.), lexical statistics. Language-agnostic by construction because it never matches specific words.
- **Semantic** (optional, requires `fastembed`): embeds the text and computes cosine similarity against prototype anchors. Multilingual model, so a Spanish prompt and its English equivalent score similarly.

When both paths run, scores are blended (0.65 semantic + 0.35 structural for complexity; 0.55 / 0.45 for depth).

The full axis-by-axis breakdown — every signal counted, every point assigned — is documented in [SCORING.md](SCORING.md). That file is the single source of truth for the scoring math; the code in `complexity.py`, `depth.py`, and `structural.py` matches it exactly.

> **No keyword lists are used anywhere.** The complexity and depth scorers operate on regex patterns over structure, not on enumerated word lists. This is the core design decision recorded in [ADR-001-No-hardcoded-keywords](../docs/) — adding support for a new technical domain means adding 1–2 prototype examples to `anchors.py`, not maintaining word lists.

---

## Missing-aspect detection (only on block)

When the guard decides to block, `detect_missing_aspects` (in `depth.py`) inspects the prompt-response pair for five language-agnostic structural gaps. Each gap detected becomes one bullet in the block reason fed back to Claude.

1. **Compound prompt without structured response.** Prompt has `>= 2` `?` characters OR `>= 30` words with `>= 2` commas, but the response has no headings, lists, tables, or code blocks. → "a structured breakdown (sections, lists, or step-by-step form) for the compound request".
2. **Short response to technical prompt.** Prompt has `>= 15` words and contains technical patterns (dotted identifiers, acronyms, mixed-caps, inline code), but the response is shorter than `3 ×` the prompt's word count. → "sufficient expansion for the technical scope of the question".
3. **Multi-question without enumeration.** Prompt has `>= 2` `?` characters but the response has neither lists nor headings. → "explicit treatment of each distinct sub-question".
4. **Repetitive response.** Response has `>= 40` tokens but type-token ratio (`unique / total`) is `< 0.3`. → "lexical and conceptual variety (the response repeats itself)".
5. **Technical prompt without an example.** Prompt is technical (same definition as #2), `>= 15` words, but the response has no code blocks and is under 150 words. → "a concrete example, command, or code snippet to anchor the answer".

Each phrase is emitted verbatim into the `reason` template. Phrasing is generic so the same detection works on prompts in any language.

---

## Anti-loop safety

Claude Code sets `stop_hook_active: true` on the second attempt of a forced continuation. `analyze.py` checks this flag first — if already in a forced state, the guard returns exit 0 immediately without re-evaluating. This bounds the loop to exactly one re-prompt per complex turn.

For defense in depth, `max_retries` in config is also a documented ceiling (default `2`).

---

## Fail-open philosophy

Every error path in adaptive-guard results in exit 0 (allow stop):

- No Python interpreter available → `adaptive-guard.sh` exits 0.
- The analyzer file is missing or unreadable → `adaptive-guard.sh` exits 0.
- stdin is empty or not valid JSON → `analyze.py` returns 0.
- `transcript_path` is missing, unreadable, a symlink, or a non-regular file → empty prompt, returns 0.
- Config file malformed or any uncaught exception inside `main()` → caught, logged anonymously to `~/.claude/telemetry/adaptive-guard.err.log`, returns 0.
- Telemetry write itself fails → swallowed silently; the user's session is never blocked because of telemetry I/O.

The guard is a quality tool, not a blocker. It must never prevent a user from finishing a session because of its own bugs. This is enforced by `tests/test_depth.py::test_e2e_unexpected_exception_fails_open` and the four other `*_fails_open` regression tests.

---

## Performance characteristics

The hook is on the critical path of every Claude Code turn, so latency matters.

- **Pure structural scoring** (default): typically `< 50 ms` per turn on commodity hardware. Bash startup + fresh Python process dominate; the actual scoring is microseconds.
- **Two-stage gating saves work**: if the prompt scores below `complexity_min_score`, depth scoring is skipped entirely. Trivial prompts ("hi", "what time is it?") finish in the structural-only fast path.
- **Optional semantic layer** (`fastembed` installed): adds a meaningful cost. Claude Code spawns a fresh Python process per Stop event, so the embedding model (`paraphrase-multilingual-MiniLM-L12-v2`, ~120 MB ONNX) re-initializes on each turn. First-ever invocation downloads the model (~30 s on a typical link); subsequent invocations reload from disk (~100–300 ms).
- **Anchor embeddings cache**: anchor strings are immutable module-level lists, so their embeddings are computed once per process and reused across all texts scored within that process.

If sub-second cold-start matters and you don't need semantic scoring, leave `fastembed` uninstalled — pure structural is the default. A persistent-daemon mode (one process across turns, no model reload cost) is on the v0.2 roadmap.

---

## Why heuristics, not an LLM judge (for v0.1)

A local LLM judge (v0.3 roadmap) would be more precise but:

1. Requires a local inference stack (llama.cpp, LM Studio).
2. Adds 2–5 s latency per Stop in the hot path.
3. Installs are painful for users without GPUs.

Heuristics are deterministic, work on any machine with Python, and stay fast in the structural-only path. They produce more false positives and false negatives than an LLM judge — the right trade-off for the default. The semantic blend with `fastembed` narrows that gap when the user accepts the cold-start cost.
