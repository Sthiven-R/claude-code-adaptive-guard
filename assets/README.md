# assets/

Brand and marketing artifacts. The dashboard ships its own copies
of the logo and favicon under `dashboard/`; this directory holds
the **public-facing** assets used by the README, GitHub Settings,
and external launch material.

## Files

| File | Use | How to regenerate |
|---|---|---|
| `logo.svg` | The horizontal lockup (mark + wordmark) shown at the top of the README. | Hand-edited SVG; keep the mark path identical to `dashboard/src/lib/components/Logo.svelte`. |
| `social-preview.svg` | Source for the GitHub social card (1280×640). | Hand-edited SVG. **Convert to PNG before uploading** — GitHub does not accept SVG. |
| `dashboard-hero.svg` | Provisional README hero. **Mock**, not a real screenshot. | Replace with a real screenshot once the desktop bundle is built (see below). |

## Converting SVG to PNG

GitHub's social-preview uploader requires PNG. Pick one of:

- **Figma**: import the SVG, export `@1x` PNG (1280×640).
- **Inkscape**: `inkscape -e social-preview.png -w 1280 -h 640 social-preview.svg`
- **ImageMagick**: `magick social-preview.svg social-preview.png`
- **Online**: any "SVG to PNG" converter at the listed dimensions.

Upload the resulting PNG at: GitHub → Settings → General → Social
preview → Edit.

### Font fallback during conversion

`social-preview.svg` references **Inter** (sans) and **JetBrains Mono**
for the wordmark. The renderer that converts the SVG to PNG looks for
those fonts in the host system; if they are missing, it silently
falls back to the next family in the SVG `font-family` chain
(`ui-monospace, monospace` for the wordmark, `system-ui, sans-serif`
for the tagline). The fallback PNG is still readable, but the
wordmark loses its custom letter-spacing and the visual weight shifts.

For brand-perfect output, install both fonts before converting:

- **Inter**: <https://rsms.me/inter/> (TTF/OTF download).
- **JetBrains Mono**: <https://www.jetbrains.com/lp/mono/> (TTF download).

Figma users: install once on the OS, then "Re-link missing fonts"
in the Figma file. ImageMagick / Inkscape pick up system fonts
automatically once they are installed.

## Replacing the provisional dashboard hero

The current `dashboard-hero.svg` is a hand-drawn mock that
approximates the real layout. To replace it with an actual
screenshot:

```bash
cd dashboard
npm install         # first time only
npm run tauri dev   # launches the desktop window
```

With the window open and some telemetry loaded:

1. Resize the window to ~1600×900 (close to the README aspect).
2. Capture using Snipping Tool (Win), Shottr (macOS) or Flameshot (Linux).
3. Save as `assets/dashboard-real.png` (compress with `pngquant --quality=80-95` if > 400 kB).
4. In `README.md`, replace `assets/dashboard-hero.svg` with `assets/dashboard-real.png`.
5. Keep the mock in the repo as a fallback / historical reference.

## Brand source of truth

For voice, color, logo direction and tagline decisions, see
[`/BRAND.md`](../BRAND.md) at the repo root. Anything in this
directory must follow what that document declares.
