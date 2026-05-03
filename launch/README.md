# launch/

Material for the public launch of adaptive-guard v0.1.0. Three
deliverables, one canonical voice (see [`/BRAND.md`](../BRAND.md)).

| File | What it is | Status |
|---|---|---|
| [`x-thread.md`](x-thread.md) | Six-tweet thread for X. EN primary. Each tweet pre-counted to fit 280 chars. | Ready to copy-paste |
| [`video-script.md`](video-script.md) | 60-second narrated demo video script. Optional but lifts day-one engagement. | Ready to record |
| [`gif-recipe.md`](gif-recipe.md) | Recipe for the 8–15 s `demo.gif` attached to tweet 5. Single most important asset of the launch. | Ready to record |

## Order of operations on launch day

1. **Generate `assets/demo.gif`** following `gif-recipe.md`.
   Without this, tweet 5 has no media and the thread loses its
   strongest beat.
2. **(Optional) Record the 60s video** following `video-script.md`.
   Skip if time-constrained — the GIF alone carries a competent
   launch. The video is for sustained traction beyond day one.
3. **Upload the social preview PNG** at GitHub Settings → General
   → Social preview. Source SVG: [`/assets/social-preview.svg`](../assets/social-preview.svg).
4. **Verify the v0.1.0 release** has all three platform bundles
   attached and the body matches the README's "first-run warning"
   block.
5. **Post the thread** following the checklist in `x-thread.md`.
6. **Pin tweet 1** to the profile until v0.2.

## What we are NOT shipping at launch

- A landing page. The README is the landing page for v0.1. Adding
  a separate site is on the v0.2/v0.3 roadmap if there is traction
  warranting it.
- An ES translation of the thread. The brand voice is EN-primary;
  ES is for the dashboard UI strings, not for marketing.
- Influencer outreach, paid promotion, Hacker News submission. The
  launch is the thread + repo metadata. If the thread lands, HN
  submission is a day-three move; if it does not, HN burns a card
  we cannot reuse.

## After launch

The `/schedule` routine `trig_012N2HqSPNKsctcPaxxiWcE6` fires on
2026-05-06 to audit the repo's first-week traction (stars, forks,
issues, external PRs). Use that report to decide whether to
escalate to Tier 3 polish (landing page, brand kit, paid demo
video) or hold at Tier 2 and let the work speak.
