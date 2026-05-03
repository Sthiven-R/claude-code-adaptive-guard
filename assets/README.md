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
