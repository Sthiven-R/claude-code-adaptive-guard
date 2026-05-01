# Adaptive Guard — brand notes

This document is the source of truth for the project's identity.
Anyone touching the dashboard, the README, the GitHub repo, or
the X / launch material should read this first.

It is short on purpose. If a question is not answered here, the
answer is "decide and add it here so the next person does not
have to re-decide."

---

## Voice

**Primary language: English.** The README, GitHub repo description,
release notes, and the X thread are written in English because the
audience is the global Claude Code community.

The product itself stays bilingual via `dashboard/src/lib/i18n.ts`
(English + Spanish). That is a UX decision, not a brand decision —
the brand voice does not switch.

**Tone.** Direct, technical, slightly dry. We talk like the operator
the product is built for: senior engineers who want signal, not
marketing. No emoji in copy. No "✨ amazing ✨" language. State the
behavior; let the reader decide if it is amazing.

**Reframe (post-mortem / dashboard analogy).** The product is not
"a hook that blocks bad answers." It is the **dashboard for the
post-mortem you would otherwise never run**. Every Claude Code
decision flows through it; you can replay any of them later. When
copy needs to explain what the product is, lead with that frame,
not with the three hook functions.

---

## Wordmark

```
adaptive·guard
```

Lowercase. Middle dot (`·`, U+00B7) separating the two words. Set
in JetBrains Mono semibold. The mid-dot is the cyan accent color;
the rest is the ink color.

Avoid:
- Title case ("Adaptive Guard") — except in display contexts where
  prose grammar demands it (sentence start, headings).
- Hyphen ("adaptive-guard") — that is the package / CLI name, not
  the brand name. Use it in code blocks, not in body copy.
- Camel case, all-caps, ampersand variants — none of these.

---

## Symbol

A 3/4 arc opening at the top-right, with a 5-segment heartbeat
trace crossing the center.

The arc reads as **guard** (a shield, but open — not a fortress).
The pulse reads as **monitoring** (every decision is observed, in
real time). The asymmetry is deliberate: a perfectly symmetric
mark reads as decorative; the off-center pulse reads as live.

Implementation: `dashboard/src/lib/components/Logo.svelte`.
Standalone favicon copy: `dashboard/public/favicon.svg`.

---

## Color signature

**Cyan, electric (`#22d3ee` / `--cyan-400`).** Saturated, high
chroma. Reads as "live signal" rather than "corporate blue."
Used for: the wordmark mid-dot, the symbol strokes, accent UI
elements (focus rings, the LIVE indicator pulse, links).

**Alert (`#f97316` / `--alert-500`).** Amber-orange. Used for
regression / unexpected-state surfaces — the "check engine"
moments the dashboard is built to surface.

**Danger (`#f87171` / `--danger-400`).** Red. Reserved for the
`block` decision badge and unrecoverable errors. Distinct from
alert: alert means "look at this," danger means "this was
blocked."

**Success (`#4ade80` / `--success-400`).** Green. Reserved for
the `allow_deep_response` decision badge.

The full ramps and semantic tokens live in
`dashboard/src/lib/styles/tokens.css`.

---

## Tagline

**Primary (EN, 1 line):**

> Live telemetry for every Claude Code decision.

**Secondary (ES, 1 line — for the Spanish UI strings, not for X):**

> Telemetría en vivo de cada decisión de Claude Code.

Use the primary tagline in:
- The README hero
- The X bio / pinned thread
- The GitHub repo description (truncate to fit 120 chars)

The tagline is not the elevator pitch. The elevator pitch is the
post-mortem reframe (above). The tagline is the one-line label
that goes under the logo on the README hero image.

---

## What this brand is NOT

- It is **not** an AI safety tool. It does not claim to make
  Claude Code "safer." It surfaces what Claude Code did so the
  operator can decide.
- It is **not** a "productivity booster." We do not promise faster
  shipping; we promise visibility.
- It is **not** cloud-based. Everything is local. The brand
  language should never imply "we collect" or "we analyze on our
  servers" — there are no servers.

If a copywriting choice would conflict with any of the three
above, it is wrong, no matter how good it sounds.
