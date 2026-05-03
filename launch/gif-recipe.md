# `launch/demo.gif` — recording recipe

The GIF that ships in tweet 5 of the X thread. Goal: 8–15 seconds,
under 5 MB, loops cleanly. Single most important asset of the
launch — the thread can ship without the video, but every X
launch with technical content needs a moving image at the
"show me" beat or the conversion drops in half.

Shipped in this directory only as a recipe, not as a binary —
GIFs are large, regenerable, and bloat git history.

---

## What to capture

A single, scripted ~12-second moment. Three beats:

  Beat 1 (0–3 s) — Dashboard idle, LIVE indicator pulsing,
                   recent-decisions list visible.
  Beat 2 (3–8 s) — A new decision card slides in from the top
                   (real telemetry event, triggered by you running
                   a Claude Code prompt in another window).
  Beat 3 (8–12 s) — Hover the new card → click to expand → the
                   breakdown panel appears → hold 1 second on
                   the score axes.

The viewer should be able to follow the story with no audio,
no captions: "something arrived → I clicked it → I see what
happened." If you cannot describe the GIF in one sentence, it
is too busy.

## Setup before recording

1. **Pre-load telemetry**: ~30–50 prior decisions so the stats
   header reads "Total: 47", not "Total: 1". Bare zero counters
   undercut the "live telemetry" claim.
2. **Theme: dark, EN UI, balanced profile.** Keep the variables
   minimal — switching themes mid-clip splits attention.
3. **Resize the dashboard window to 1280×720.** Captures at the
   GIF's eventual rendered size avoid resampling artifacts.
4. **Hide the OS taskbar / dock**, hide window chrome if your
   recorder allows. The viewer sees the app, not Windows.
5. **Pre-script the prompt** you will send to Claude Code so
   timing is predictable. The prompt should be one that scores
   high complexity (a multi-part architectural question) and
   gets blocked — the visible decision in beat 2 should be a
   `BLOCK` (red), not a `SKIP` (gray). BLOCK is the visually
   distinctive case; SKIPs read as "nothing happened."

## Recording tools

- **Windows:**
  - [ScreenToGif](https://www.screentogif.com/) (open source,
    captures GIF natively, has a built-in editor).
  - OBS Studio + ffmpeg conversion (overkill but flexible).
- **macOS:**
  - [Gifski](https://gif.ski/) for high-quality GIF encoding from
    QuickTime captures.
  - Kap (free, simple).
- **Linux:**
  - [Peek](https://github.com/phw/peek) (X11 only, simple).
  - byzanz-record + ffmpeg.

## Optimization

The raw capture will be 20–60 MB. X's per-tweet GIF cap is 15 MB,
but anything over ~5 MB plays sluggishly on mobile.

**Two-pass ffmpeg pipeline** (works regardless of recorder):

```bash
# 1. Generate a custom palette from the source video for better colors.
ffmpeg -i raw-capture.mp4 -vf "fps=15,scale=1280:-1:flags=lanczos,palettegen" palette.png

# 2. Encode the GIF using the palette.
ffmpeg -i raw-capture.mp4 -i palette.png \
  -filter_complex "fps=15,scale=1280:-1:flags=lanczos[x];[x][1:v]paletteuse" \
  demo.gif
```

If the result is still > 5 MB, drop fps to 12 or scale to 960
wide. Frame rate before resolution — the dashboard does not need
fluid 24 fps motion.

**Alternative: ship a video instead of a GIF.** X accepts MP4 in
tweets, plays inline, and a 12-second 720p MP4 lands at ~1 MB
with crisp text. The trade-off is lower autoplay reliability on
embedded views (Mastodon, RSS readers). For the launch tweet
itself, MP4 wins; for the README, GIF wins.

## Hosting

Once recorded and optimized, host the file at one of:

- `assets/demo.gif` in this repo (committed, served by GitHub raw).
  Pros: no external dependency. Cons: bloats clone size by ~3 MB.
- A GitHub release attachment (upload to the v0.1.0 release page).
  Pros: outside the working tree. Cons: one extra click to update.
- An external host (Cloudflare Images, Imgur). Pros: optimal
  delivery. Cons: another moving part.

Recommendation: commit to `assets/demo.gif` for v0.1. Revisit if
the file approaches 8 MB.

## Quality bar

Before posting, verify:

- [ ] Plays in a loop without a visible "snap back" frame
      (start frame ≈ end frame).
- [ ] Text in the dashboard is legible at 1× zoom on a phone.
- [ ] The cursor is visible (recorders usually have a "show
      cursor" toggle — turn it on).
- [ ] No personal data on screen: real session IDs are fine
      (they are random hex), but check the OS clock and any
      window titles in the background.
- [ ] File size under 5 MB.
