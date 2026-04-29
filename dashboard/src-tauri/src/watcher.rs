//! Live file watcher for the telemetry JSONL.
//!
//! Spawns a background thread that observes the telemetry file (and its
//! parent directory, to survive atomic rename rotations) and emits a
//! Tauri event `telemetry-changed` whenever the file's contents change.
//!
//! Debounce is applied so that rotation (write new tmp -> os.replace)
//! produces a single event, not a flurry of them.

use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::telemetry;

/// Resolve ~/.claude/telemetry (directory) and the JSONL path.
fn telemetry_paths() -> Option<(PathBuf, PathBuf)> {
    let home = dirs::home_dir()?;
    let dir = home.join(".claude").join("telemetry");
    let file = dir.join("adaptive-guard.jsonl");
    Some((dir, file))
}

/// Spawn the watcher thread. Never fails the app startup: if the
/// watcher cannot be set up, the dashboard still works in
/// poll-on-refresh mode.
pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let (dir, file) = match telemetry_paths() {
            Some(p) => p,
            None => return,
        };

        // Make sure the directory exists so the watcher has something
        // to observe. Ignore errors; if we cannot create it, the
        // telemetry itself cannot write either.
        let _ = std::fs::create_dir_all(&dir);

        let (tx, rx) = channel::<DebounceEventResult>();

        let debouncer = match new_debouncer(Duration::from_millis(250), None, tx) {
            Ok(d) => d,
            Err(_) => return,
        };

        // Watch the parent dir (not just the file), so atomic renames
        // during log rotation are still observed.
        //
        // NOTE: the `Watcher::watch` call requires `&mut` on the
        // underlying watcher, but the debouncer's `.watcher()` returns
        // one. We bind mutably here.
        {
            let mut debouncer = debouncer;
            if debouncer
                .watcher()
                .watch(&dir, RecursiveMode::NonRecursive)
                .is_err()
            {
                return;
            }

            // The debouncer owns the sender; when this loop exits (app
            // shutdown), the debouncer drops and the channel closes.
            // The debouncer's lifetime ends with this scope — the
            // explicit `let _ =` previously here was redundant.
            while let Ok(res) = rx.recv() {
                if let Ok(events) = res {
                    // Watching the parent dir non-recursively means any
                    // event with our file path is ours.
                    let touched = events.iter().any(|e| e.paths.contains(&file));
                    if touched {
                        // Invalidate the parsed-records cache before
                        // notifying the frontend so the next
                        // telemetry_* command reloads from disk.
                        telemetry::invalidate_cache();
                        let _ = app.emit("telemetry-changed", ());
                    }
                }
                // Errors are non-fatal; keep watching.
            }
        }
    });
}
