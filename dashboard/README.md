# Adaptive Guard — Desktop Dashboard

Live monitoring UI for adaptive-guard. Built with **Tauri v2 + Svelte + TypeScript**.

Shows the telemetry written by the Stop hook in real time:
- Recent decisions with per-axis breakdowns
- Score distributions
- Session filtering
- Approximate token counter

Runs on Windows, macOS, and Linux. Single-digit MB installer. ~30 MB RAM idle.

---

## Development setup (one-time)

Requires:
- **Node.js 20+** and npm
- **Rust 1.78+** and Cargo
- Platform build tools for Tauri v2:
  - **Windows**: Microsoft C++ Build Tools (Visual Studio Installer)
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `webkit2gtk-4.1-dev`, `libgtk-3-dev`, and friends

### Install dependencies

```bash
cd dashboard
npm install
```

Rust dependencies are fetched automatically by `cargo` on first build.

### Run in development

```bash
npm run tauri dev
```

This starts Vite (frontend HMR on `http://localhost:1420`) and launches the Tauri window pointed at it. Hot-reload on Svelte edits. Rust changes trigger a rebuild.

### Build for release

```bash
npm run tauri build
```

Produces native installers in `src-tauri/target/release/bundle/`:
- `msi/` on Windows
- `dmg/` on macOS
- `deb/`, `appimage/`, `rpm/` on Linux

The first release build is slow (compiles dependencies). Subsequent builds are incremental.

---

## Architecture

```
dashboard/
├── src/                  Svelte + TypeScript frontend
│   ├── App.svelte        Main dashboard view
│   ├── main.ts           Entry point
│   └── app.css           Dark theme
├── src-tauri/            Rust backend
│   ├── src/
│   │   ├── main.rs       Entry point
│   │   ├── lib.rs        Tauri commands
│   │   └── telemetry.rs  JSONL reader
│   ├── Cargo.toml        Rust dependencies
│   └── tauri.conf.json   App configuration
└── package.json
```

The Rust backend exposes commands via Tauri IPC:

- `telemetry_status()` → `{ path, exists, record_count, error }`
- `telemetry_recent(limit)` → `Vec<TelemetryRecord>`

The Svelte frontend calls them through `@tauri-apps/api/core#invoke`.

All data flow is **one-way and read-only**: telemetry → UI. The dashboard never writes to the telemetry file.

---

## Current status — Sprint 2

Scaffold in place. Backend reads telemetry. Frontend shows status + recent 10 decisions in a table.

Next sprints:
- Sprint 3: expandable breakdown panel, score histograms
- Sprint 4: live file watching, token counter, session filters
- Sprint 5: welcome flow with built-in hook installer
- Sprint 6: auto-updater via GitHub Releases
- Sprint 7: cross-OS testing and release

---

## Icons

Real icons ship in Sprint 5. For dev, Tauri's default icons are used. If the build complains about missing icon files, create them once with the Tauri CLI:

```bash
npm run tauri icon path/to/source-icon.png
```

Or place placeholder PNGs in `src-tauri/icons/` matching the names in `tauri.conf.json`.
