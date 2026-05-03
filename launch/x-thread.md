# X launch thread — adaptive-guard v0.1.0

Six tweets. English (primary). Each fits inside the 280-character
limit; do not edit without re-counting. Replace `[REPO LINK]` with
the canonical URL before posting (`https://github.com/Sthiven-R/claude-code-adaptive-guard`).

The thread leads with the post-mortem (external validation) before
introducing the tool, so readers who never click the link still
take away the strongest piece of evidence.

---

## Tweet 1 — hook (227 chars)

> Anthropic's April 23 post-mortem admitted three product-layer regressions silently degraded Claude Code response depth for ~7 weeks.
>
> I built a tool that catches this from outside the harness. Open-sourced today.
>
> 🔗 [REPO LINK]

## Tweet 2 — reframe / dashboard analogy (268 chars)

> Adaptive Guard isn't "a hook that blocks bad answers."
>
> It's the dashboard for the post-mortem you'd otherwise never run. Every Claude Code Stop event flows through it; you can replay any of them later.
>
> Like the check-engine light on a car you can't open the hood of.

## Tweet 3 — how it works (265 chars)

> 3 functions, no LLM:
>
> 1. Score prompt complexity (length, density, tokens)
> 2. Score response depth (markdown, code, diversity)
> 3. Complex + shallow → block, force retry with explicit feedback
>
> Structural by default. Optional embedding for higher precision.

## Tweet 4 — privacy + honesty (250 chars)

> Two non-negotiables:
>
> – Fail-open: any error → exit 0. The guard is a quality tool, never a blocker.
> – No prompt or response text is ever logged. Only counts and integers. Telemetry lives at ~/.claude/telemetry/, on your machine, never anywhere else.

## Tweet 5 — dashboard (233 chars + media)

> Native desktop dashboard (Tauri + Svelte). Watches the telemetry file in real time — every decision appears in <1 second.
>
> Per-decision breakdown of every score axis. System tray, dark / light / auto theme, EN / ES UI.

**Attach: `launch/demo.gif`** (recipe in `gif-recipe.md`).

## Tweet 6 — CTA (251 chars)

> Free + MIT + zero account.
>
> Pre-release .msi / .dmg / .AppImage builds attached to the v0.1.0 release. Unsigned for now (signing on v0.2). First run shows the standard SmartScreen / Gatekeeper warning.
>
> [REPO LINK]

---

## Posting checklist

- [ ] Replace `[REPO LINK]` (4 occurrences across the thread).
- [ ] Verify each tweet is ≤280 characters using https://character-counter.com/x or X's draft view (X counts URLs as 23 chars regardless of length).
- [ ] Attach `launch/demo.gif` to tweet 5 specifically — tweets without media at the right beat lose 30%+ engagement.
- [ ] Post tweet 1, then reply with each subsequent tweet to thread them. Do not pre-schedule the entire thread as a "long-form post" — single tweets thread better in the algorithm and let people quote-RT individual beats.
- [ ] Pin tweet 1 to the profile until v0.2.

## Spanish version

Not provided. The brand voice is English-primary (see [`/BRAND.md`](../BRAND.md)). If a Spanish version is needed for LinkedIn / Mastodon / Hispanic-tech communities, draft separately — do not auto-translate this thread, the technical phrasing does not carry well.
