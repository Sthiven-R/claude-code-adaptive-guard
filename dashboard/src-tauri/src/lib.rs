//! adaptive-guard-dashboard library entry.
//!
//! Exposes Tauri commands that the Svelte frontend invokes:
//!   - Telemetry reads (status, recent, stats, histogram). Read-only.
//!   - Hook lifecycle (status, install, uninstall). Writes to
//!     `~/.claude/settings.json` with backup + atomic rename.
//!
//! Also spawns a background file watcher that emits `telemetry-changed`
//! events so the UI can refresh automatically, and installs a system
//! tray icon for quiet always-on monitoring.

mod install;
mod telemetry;
mod tray;
mod watcher;

use install::{HookStatus, InstallResult};
use tauri::WindowEvent;
use telemetry::{HistogramBucket, TelemetryRecord, TelemetryStats, TelemetryStatus};

/// Return status of the telemetry file: path, existence, record count.
#[tauri::command]
fn telemetry_status() -> TelemetryStatus {
    telemetry::status()
}

/// Return the most recent `limit` decisions (default 100).
#[tauri::command]
fn telemetry_recent(limit: Option<usize>) -> Vec<TelemetryRecord> {
    telemetry::recent(limit.unwrap_or(100))
}

/// Return aggregated statistics across all records.
#[tauri::command]
fn telemetry_stats() -> TelemetryStats {
    telemetry::stats()
}

/// Return a histogram for a score dimension ("complexity" or "depth").
#[tauri::command]
fn telemetry_histogram(dim: String) -> Vec<HistogramBucket> {
    telemetry::histogram(&dim)
}

/// Return the current state of the Stop hook in `~/.claude/settings.json`.
#[tauri::command]
fn hook_status() -> HookStatus {
    install::status()
}

/// Install the adaptive-guard Stop hook. Idempotent: a second call after
/// success is a no-op.
#[tauri::command]
fn hook_install() -> InstallResult {
    install::install()
}

/// Remove the adaptive-guard Stop hook. Preserves third-party hooks.
#[tauri::command]
fn hook_uninstall() -> InstallResult {
    install::uninstall()
}

/// Return the dashboard's compiled version. Resolved at build time from
/// `Cargo.toml` so we have a single source of truth — the four sites
/// listed in `release.yml` (VERSION, package.json, Cargo.toml,
/// tauri.conf.json) flow into this command at build time and out to
/// every UI label that needs it.
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // NOTE: we do NOT register the shell plugin. The dashboard does
        // not open URLs or spawn processes — install/uninstall happens
        // through `install.rs` writing to settings.json directly.
        // Principle of least privilege (audit finding LOW-4).
        .invoke_handler(tauri::generate_handler![
            telemetry_status,
            telemetry_recent,
            telemetry_stats,
            telemetry_histogram,
            hook_status,
            hook_install,
            hook_uninstall,
            app_version,
        ])
        .setup(|app| {
            // Background file watcher: emits "telemetry-changed" events
            // whenever the JSONL is updated. Never fails the startup.
            watcher::spawn(app.handle().clone());

            // System tray with Show / Hide / Quit menu.
            if let Err(e) = tray::setup(app.handle()) {
                eprintln!("failed to init tray: {}", e);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Close-to-tray: when the user closes the main window, hide
            // it instead of quitting. The tray icon and file watcher
            // stay alive. "Quit" from the tray menu fully exits.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
