# Scoring explained

This is the single honest document about how adaptive-guard decides what to
do. Read it once, keep it as reference. The tool is deliberately inspectable:
every decision it makes can be reconstructed from the numbers recorded.

---

## The decision flow

```
1. User submits a prompt. Claude generates a response. Claude Code fires Stop hook.
2. adaptive-guard scores the prompt            -> COMPLEXITY score (0-100)
3. Is complexity < threshold? YES -> allow stop. Done. (decision = allow_simple_task)
4. adaptive-guard scores the response          -> DEPTH score (0-100)
5. Is depth >= threshold?    YES -> allow stop. Done. (decision = allow_deep_response)
6. Is depth < threshold?     YES -> emit block + reason. (decision = block)
```

Thresholds come from the active profile (`balanced`, `strict`, `lenient`).
Default is `balanced`: complexity_min_score = 40, depth_min_score = 40.

---

## The two scoring paths

Every score comes from either one or both of these paths:

### Path A - Structural (always on, zero dependencies)

Measures **form**: length, punctuation, markdown structure, technical-token
patterns, lexical statistics. Works identically on English, Spanish,
Portuguese, or any other language because it never looks at specific words -
it looks at shapes.

### Path B - Semantic (optional, requires fastembed)

Measures **meaning**: embeds the text and compares cosine similarity against
prototype anchors (examples of "simple" vs "complex" prompts, "deep" vs
"shallow" responses). Uses a multilingual embedding model, so a Spanish
prompt and its English equivalent score similarly.

### Blending

When both paths are available:

```
final_complexity = round(0.65 * semantic + 0.35 * structural)
final_depth      = round(0.55 * semantic + 0.45 * structural)
```

Semantic is weighted slightly higher because intent is usually more
informative than form. Structural is kept in the blend to ground the
decision: syntactically heavy prompts cannot be blurred out by a
semantically ambivalent embedding.

---

## Complexity axes

Every prompt is scored on five axes. Each has a point cap, so no single
signal can dominate. The totals cap at 100.

### Axis 1: length (0-20)

Normalized by chars. Prompts under 60 chars get 0 points. Saturates at
400 chars.

### Axis 2: questions (0-15)

Count of `?`. Multiple questions in a single turn are a strong signal of
compound-request intent. Boosted slightly when prompt is long.

### Axis 3: compound request (0-25)

`sentence_count * 5 + comma_count * 2 + semicolon_count * 3`. Measures
how many distinct clauses the prompt contains. "Do X, Y, and Z" scores
higher than "Do X".

### Axis 4: technical tokens (0-30)

Count of identifiable technical signals. Each detected pattern
contributes; the cap keeps any one category from dominating.

Patterns detected:
- camelCase (`myVariable`)
- snake_case (`my_function`)
- dotted identifiers (`Node.js`, `user.email`)
- version numbers (`v2.1.0`, `3.14`)
- inline code (backticks)
- acronyms (`SQL`, `HTTP`, `QPS`, `API`)
- mixed-case brand names (`PostgreSQL`, `MongoDB`)
- proper nouns mid-sentence (`Kafka`, `Redis`)
- numbers with units (`50k`, `100GB`)

### Axis 5: external references (0-10)

File paths, URLs, code blocks. Each contributes.

---

## Depth axes

Six axes. Same spirit as complexity: multiple signals, capped contributions,
language-agnostic.

### Axis 1: length (0-25)

Response word count. Responses under 30 words get 0 points. Saturates
around 600 words.

### Axis 2: headings (0-20)

Count of markdown headings (`##`, `###`, `####`, `#####`). Deduplicated so
`#### Foo` counts once, not four times.

### Axis 3: lists and tables (0-15)

Count of bullet/numbered list lines and markdown table separator rows.

### Axis 4: code blocks (0-10)

Count of fenced code blocks and inline code.

### Axis 5: lexical diversity (0-15)

Type-token ratio (unique words / total words). Shallow responses repeat
terms; deep ones vary.

- TTR >= 0.45: 15 points
- TTR >= 0.35: 10 points
- TTR >= 0.25:  5 points
- TTR <  0.25:  0 points

### Axis 6: sentence-length variance (0-15)

Standard deviation of sentence word counts. Flat list of bullets =
uniform = low. Mixed prose + structured lists = high.

- variance >= 10: 15 points
- variance >=  5:  8 points

---

## Missing-aspect detection (only when blocking)

When the guard decides to block, it scans the prompt-response pair for
five generic structural gaps. Each detected gap becomes one line in the
block reason. This is what gets fed back to Claude as guidance.

1. **Compound prompt, no structured response.** Prompt has >= 2
   questions OR >= 30 words with >= 2 commas, but the response has no
   headings, lists, tables, or code blocks.
2. **Short response to technical prompt.** Prompt has >= 15 words and
   contains technical patterns (dotted names, acronyms, mixed-caps),
   but response is less than 3x the prompt word count.
3. **Multi-question prompt without enumeration.** Prompt has >= 2
   questions but response has no list and no headings.
4. **Repetitive response.** Response has >= 40 words but TTR < 0.3.
5. **Technical prompt without an example.** Prompt is technical
   (same definition as 2), >= 15 words, but response has no code
   blocks and is under 150 words.

Gaps are described in English, generic phrasing. No keyword matching.

---

## How to inspect any decision

Every decision written to telemetry includes the full axis-by-axis
breakdown PLUS the detected signals (which specific tech tokens were
found, how many commas, what the TTR was, etc.). There are three ways
to inspect:

### 1. Last decision with full detail

```bash
./scripts/stats.sh --last
./scripts/stats.sh --last 5     # last 5 with full detail
```

This shows the axes, signals, thresholds, and decision for each. If a
prompt scored 37 when you expected 50, this tells you exactly which axis
underscored it.

### 2. Simulate a scoring

```bash
./scripts/explain.sh
```

Paste any prompt and optional response. Get the same breakdown that
telemetry would record if this were a real turn. Use this to calibrate
your mental model of the scoring.

### 3. Filter telemetry by session

```bash
./scripts/stats.sh --session 5c838959
```

Shows all decisions made within a specific Claude Code session.

---

## Calibration

The thresholds in `config/default.json` are defaults, not commandments.
If the guard isn't catching prompts you consider complex:

1. Run `./scripts/stats.sh --last` immediately after a turn that
   should have blocked but didn't.
2. Look at the complexity total and the axis breakdown.
3. Choose one:
   - Lower `complexity_min_score` in the profile you use.
   - Add one or two representative prompts to `COMPLEX_PROMPT_ANCHORS` in
     `hooks/lib/anchors.py` and install fastembed for semantic matching.
   - Accept that this specific prompt is borderline by design.

The same logic applies to depth: if responses that feel shallow are
passing through, inspect the depth axes and either lower
`depth_min_score` or raise the lexical diversity / structure requirements.

---

## What the telemetry looks like

One JSONL line per decision:

```json
{
  "ts": "2026-04-21T04:42:09+00:00",
  "session_id": "5c838959",
  "profile": "balanced",
  "decision": "allow_simple_task",
  "complexity": 37,
  "depth": null,
  "missing_count": 0,
  "prompt_chars": 216,
  "response_chars": 2612,
  "thresholds": {
    "complexity_min_score": 40,
    "depth_min_score": 40
  },
  "complexity_breakdown": {
    "total": 37,
    "structural": 37,
    "semantic": null,
    "blend_weights": null,
    "axes": {
      "length":       11,
      "questions":     0,
      "compound":     12,
      "technical":    12,
      "external_refs": 2
    },
    "signals": {
      "char_count":     216,
      "word_count":      30,
      "sentence_count":   2,
      "comma_count":      1,
      "semicolon_count":  0,
      "question_count":   0,
      "tech_token_counts": {
        "camelCase":          0,
        "snake_case":         0,
        "dotted_ident":       1,
        "version":            0,
        "inline_code":        0,
        "acronyms":           0,
        "mixed_caps":         1,
        "proper_nouns":       2,
        "numbers_with_units": 0
      },
      "tech_hit_total":    4,
      "path_count":        0,
      "url_count":         0,
      "code_block_count":  0
    }
  }
}
```

No prompt text. No response text. Only integers and counts of detected
patterns. Matched substrings (paths, URLs, inline code, identifier
strings) are deliberately NOT recorded — those routinely contain user
secrets (API tokens, home-dir credentials), so storing them verbatim
would violate the privacy contract.
