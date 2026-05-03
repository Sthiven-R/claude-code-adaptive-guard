# 60-second demo video — script

A narrated screen-capture of the dashboard catching a real
shallow-response regression. Optional — the X thread can launch
without it (using only `demo.gif`), but a video lifts engagement
significantly on day-one launches.

This script is timed in beats, not exact timestamps. Read the
narration at a normal pace; if a beat overruns, trim a sub-clause
rather than rushing.

---

## Production specs

- **Length:** 60 seconds (hard cap — X autoplays muted; the first
  3 seconds carry the hook, the rest is bonus).
- **Resolution:** 1920×1080 master, 1280×720 export for X.
- **Framerate:** 30 fps capture, 30 fps export.
- **Audio:** mono, normalized to -16 LUFS, no music.
- **Captions:** burnt-in, top-aligned, 36 px Inter Semibold, 90 %
  opacity dark backdrop. ~70% of X autoplays are muted.
- **End frame:** 2-second hold on the repo URL + handle.

## Recording setup

Two windows side by side:
- **Left half:** Claude Code session.
- **Right half:** Adaptive Guard dashboard with theme = dark, EN.

Pre-load the dashboard with ~50 historical decisions so the stats
header has real numbers, not zeros. Filter to "today" so the
recent-decisions list is short and readable.

---

## Scene 1 — the problem (0:00 → 0:08)

**Visual:** Cold open on the Claude Code window. A complex prompt
visible (suggested fixture: a real one from your dev work, NOT
"how do I sort a list").

**Narration:**
> "Claude Opus 4.7 decides on its own how deeply to think about
> each task. When it underestimates a complex prompt, you get a
> shallow answer — and you might never notice."

## Scene 2 — the post-mortem (0:08 → 0:16)

**Visual:** Cut to the Anthropic post-mortem page, scrolled to the
"effort cut from high to medium" paragraph. Highlight box around
that line.

**Narration:**
> "Anthropic published a post-mortem on April 23 admitting this
> happened silently for seven weeks. Their internal evals didn't
> catch it."

## Scene 3 — the tool (0:16 → 0:28)

**Visual:** Cut to the dashboard, full-screen. Hero on the brand,
then pan down to the stats header showing real counts. The
LIVE indicator pulses.

**Narration:**
> "Adaptive Guard scores every Claude Code response from outside
> the harness. Complexity of the prompt, depth of the response,
> two numbers per turn. When they don't match, the guard blocks
> the stop and forces a retry with explicit feedback."

## Scene 4 — a real decision (0:28 → 0:42)

**Visual:** Trigger a real Claude Code response that scores low
on depth. The new decision appears in the dashboard within ~1
second. Click to expand the card. Show the breakdown panel:
every score axis, every detected signal.

**Narration:**
> "Here's a live decision. The prompt scored 71 for complexity —
> a multi-part architectural question. The response came back at
> depth 28 — a single paragraph. The guard blocked the stop and
> told Claude exactly what was missing: structural breakdown,
> failure-mode analysis, code blocks. Claude retried with the
> full answer."

## Scene 5 — the principles (0:42 → 0:54)

**Visual:** Scroll the dashboard to the privacy panel / settings.
Show the telemetry file path, the language toggle, the theme
toggle.

**Narration:**
> "No prompts or responses are ever logged — only score numbers,
> on your machine. The guard fails open: any error, the stop is
> allowed. It is a quality tool, never a blocker. Free and open
> source under MIT."

## Scene 6 — CTA (0:54 → 1:00)

**Visual:** End frame: large logo, repo URL, handle. Hold static
for the full 6 seconds (the hold is what people screenshot).

**Narration:**
> "github.com/Sthiven-R/claude-code-adaptive-guard. Link in the
> thread."

---

## Recording tools

- **Windows:** ScreenStudio, OBS Studio, or Camtasia.
- **macOS:** ScreenStudio (cinematic auto-zoom is the differentiator),
  ScreenFlow, or QuickTime + iMovie for a free path.
- **Linux:** OBS Studio + Kdenlive.

For voiceover: any USB condenser mic. Record in a closed room,
pop filter, monotone delivery (the brand voice is dry, not
enthusiastic — let the work speak).

## Post-production checklist

- [ ] Trim each scene to its beat; remove dead air > 0.4 s.
- [ ] Apply -16 LUFS normalization to the voice track.
- [ ] Burn captions matching the narration verbatim.
- [ ] Export H.264 MP4, ≤ 25 MB (X compresses anything larger).
- [ ] Verify the first 3 seconds work as a silent cold-open hook
      (X autoplays muted; if scene 1 makes no sense without audio,
      reshoot it with on-screen text).
