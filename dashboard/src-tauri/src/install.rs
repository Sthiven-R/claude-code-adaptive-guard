//! Install/uninstall the adaptive-guard Stop hook in `~/.claude/settings.json`.
//!
//! Mirrors `scripts/install.sh` and `scripts/uninstall.sh` exactly so the
//! dashboard and the CLI are interchangeable: a hook installed by either
//! is detected and removed by either.
//!
//! Invariants:
//! - Backup before any write. Backup name: `settings.json.backup-YYYYMMDD-HHMMSS` (UTC).
//! - Atomic write: tmp file + rename.
//! - Idempotent install: a second invocation on the same settings.json is a no-op.
//! - Uninstall preserves third-party hooks. We match by `id == "adaptive-guard"`
//!   or by the very specific simple-form command. Composed commands
//!   (containing `&&`, `||`, `;`, `|`, `$(`, backticks) are user work and
//!   never removed by us.
//! - Path safety: if the resolved bash or hook path contains shell
//!   metacharacters, we refuse to install rather than emit a fragile
//!   command string.
//! - REPO_ROOT comes from `~/.adaptive-guard/config` (written by
//!   `scripts/setup-global.sh`). The dashboard does not bootstrap the
//!   repo — it assumes the CLI was set up first.

use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const HOOK_ID: &str = "adaptive-guard";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct HookStatus {
    pub installed: bool,
    pub repo_root: Option<String>,
    pub config_path: String,
    pub settings_path: String,
    /// Set when the underlying state could not be determined (e.g. config
    /// file missing). Distinct from `installed = false`: the latter means
    /// "the hook isn't there"; an `error` means "we can't tell".
    pub error: Option<String>,
}

/// Result of an install or uninstall action.
///
/// `backup_path` is `Some(path)` whenever a backup was made — and we
/// make a backup before *any* read of `settings.json`, including the
/// "already installed, no changes" path. Callers can therefore expect
/// a fresh backup file on disk for every successful invocation, even
/// no-ops. `None` only means the action failed before we got to the
/// backup step (e.g. settings.json did not exist on uninstall, or the
/// repo config could not be read on install).
#[derive(Serialize)]
pub struct InstallResult {
    pub ok: bool,
    pub message: String,
    pub backup_path: Option<String>,
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

fn home() -> Option<PathBuf> {
    dirs::home_dir()
}

fn config_path() -> Option<PathBuf> {
    home().map(|h| h.join(".adaptive-guard").join("config"))
}

fn settings_path() -> Option<PathBuf> {
    home().map(|h| h.join(".claude").join("settings.json"))
}

fn read_repo_root() -> Result<PathBuf, String> {
    let cfg = config_path()
        .ok_or_else(|| "Home directory not resolvable".to_string())?;
    let f = fs::File::open(&cfg).map_err(|e| {
        format!(
            "Could not read {}. Run `adaptive-guard setup-global` from the repo first. ({})",
            cfg.display(),
            e
        )
    })?;
    for line in BufReader::new(f).lines() {
        let line = line.map_err(|e| e.to_string())?;
        if let Some(rest) = line.strip_prefix("REPO_ROOT=") {
            let p = PathBuf::from(rest.trim());
            if !p.is_dir() {
                return Err(format!(
                    "REPO_ROOT in {} points to a path that does not exist: {}",
                    cfg.display(),
                    p.display()
                ));
            }
            return Ok(p);
        }
    }
    Err(format!("REPO_ROOT= line not found in {}", cfg.display()))
}

fn hook_script_path(repo_root: &Path) -> PathBuf {
    repo_root.join("hooks").join("adaptive-guard.sh")
}

fn default_config_path(repo_root: &Path) -> PathBuf {
    repo_root.join("config").join("default.json")
}

// ---------------------------------------------------------------------------
// MSYS POSIX path conversion
// ---------------------------------------------------------------------------

/// Convert `C:\Foo\Bar` or `C:/Foo/Bar` to `/c/Foo/Bar`. Already-POSIX
/// inputs pass through unchanged. Required because the hook command is
/// invoked by `/usr/bin/bash` which expects MSYS-style paths on Windows.
fn to_msys_posix(p: &str) -> String {
    let bytes = p.as_bytes();
    if bytes.len() >= 3
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
        && bytes[0].is_ascii_alphabetic()
    {
        let drive = (bytes[0] as char).to_ascii_lowercase();
        let rest: String = p[3..].chars().map(|c| if c == '\\' { '/' } else { c }).collect();
        format!("/{}/{}", drive, rest)
    } else {
        p.replace('\\', "/")
    }
}

/// Discover an absolute bash path to emit in the hook command. Matches
/// the CLI script's behavior: prefer `/usr/bin/bash` if it exists,
/// otherwise look in standard Git Bash install locations on Windows,
/// otherwise resolve via PATH. The path is always returned in MSYS
/// POSIX form so `/usr/bin/bash` (or its discovered substitute) can run
/// the hook regardless of the shell context Claude Code spawns from.
fn bash_command_path() -> String {
    // POSIX systems (and Git Bash distributions that ship a stable
    // /usr/bin/bash symlink) — preferred for portability.
    for candidate in ["/usr/bin/bash", "/bin/bash"] {
        if Path::new(candidate).is_file() {
            return candidate.to_string();
        }
    }
    // Windows: standard Git Bash install locations. We check these
    // before PATH because they are deterministic and don't depend on
    // the user's PATH ordering.
    if cfg!(windows) {
        for candidate in [
            r"C:\Program Files\Git\usr\bin\bash.exe",
            r"C:\Program Files (x86)\Git\usr\bin\bash.exe",
        ] {
            if Path::new(candidate).is_file() {
                return to_msys_posix(candidate);
            }
        }
    }
    // PATH lookup as last resort. We strip the `.exe` because the
    // emitted command is run by `/usr/bin/bash` (or equivalent), which
    // resolves either form.
    if let Ok(path_var) = std::env::var("PATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        let names: &[&str] = if cfg!(windows) {
            &["bash.exe", "bash"]
        } else {
            &["bash"]
        };
        for dir in path_var.split(sep) {
            if dir.is_empty() {
                continue;
            }
            for name in names {
                let p = Path::new(dir).join(name);
                if p.is_file() {
                    let s = p.display().to_string();
                    return if cfg!(windows) { to_msys_posix(&s) } else { s };
                }
            }
        }
    }
    // Fallback to the conventional POSIX path. If it doesn't exist on
    // this machine the install will succeed but the hook will fail at
    // run time — at which point the user has actionable info to fix
    // their environment.
    "/usr/bin/bash".to_string()
}

// ---------------------------------------------------------------------------
// Path safety
// ---------------------------------------------------------------------------

// Inside POSIX single quotes, every byte except `'` is literal. The
// `posix_single_quote` helper handles `'` via the close-escape-reopen
// idiom, so shell metacharacters like `$`, `&`, `()`, `[]` etc. are
// safe inside our quoted paths and don't belong in this rejection list.
//
// What we DO reject are bytes that either break path validity on real
// filesystems (newlines, null) or signal a conversion bug (a stray
// backslash that `to_msys_posix` should have stripped). Narrow on
// purpose — rejecting normal Windows paths like
// `C:\Users\Foo & Bar\repo` was friction without commensurate safety.
const DANGEROUS: &[char] = &['\n', '\r', '\0', '\\'];

fn assert_safe(p: &str, label: &str) -> Result<(), String> {
    let bad: Vec<char> = p.chars().filter(|c| DANGEROUS.contains(c)).collect();
    if !bad.is_empty() {
        let mut sorted: Vec<char> = bad;
        sorted.sort();
        sorted.dedup();
        return Err(format!(
            "{} contains characters that are invalid in a POSIX path or signal a path-conversion bug ({:?}): {}. \
             A backslash here usually means MSYS conversion did not run; newlines or NULs cannot be valid in any path.",
            label, sorted, p
        ));
    }
    Ok(())
}

fn posix_single_quote(p: &str) -> String {
    format!("'{}'", p.replace('\'', r"'\''"))
}

// ---------------------------------------------------------------------------
// Backup + atomic write
// ---------------------------------------------------------------------------

fn timestamp_utc() -> Result<String, String> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            format!(
                "System clock is before the UNIX epoch ({}). Refusing to create a backup with timestamp 19700101-000000 \
                 — that would silently overwrite older backups. Fix the system clock and retry.",
                e
            )
        })?
        .as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = (secs / 86400) as i64;
    let (y, mo, d) = days_to_ymd(days);
    Ok(format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        y, mo, d, h, m, s
    ))
}

/// Howard Hinnant's `civil_from_days`: days since 1970-01-01 (UTC) → (Y, M, D).
/// Correct for any reasonable date including leap years and centuries.
fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    let days = days + 719468;
    let era = if days >= 0 { days / 146097 } else { (days - 146096) / 146097 };
    let doe = (days - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

fn make_backup(settings: &Path) -> Result<PathBuf, String> {
    let ts = timestamp_utc()?;
    let parent = settings
        .parent()
        .ok_or_else(|| format!("settings path has no parent: {}", settings.display()))?;
    let file_name = settings
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "settings file name not UTF-8".to_string())?;
    let backup = parent.join(format!("{}.backup-{}", file_name, ts));
    fs::copy(settings, &backup).map_err(|e| format!("Backup failed: {}", e))?;
    Ok(backup)
}

/// Tighten a freshly created settings.json to 0600 on Unix. Windows
/// inherits user-profile-only permissions from its ACL default, so
/// nothing to do there. Returns `Some(warning)` if chmod failed on
/// Unix — the install proceeds, but the caller surfaces the warning
/// to the user so they can fix it before secrets accumulate.
#[cfg(unix)]
fn restrict_settings_perms(settings: &Path) -> Option<String> {
    use std::os::unix::fs::PermissionsExt;
    match fs::set_permissions(settings, fs::Permissions::from_mode(0o600)) {
        Ok(()) => None,
        Err(e) => Some(format!(
            "Warning: could not restrict {} to 0600 ({}). The file may be world-readable. Run `chmod 600 {}` manually.",
            settings.display(),
            e,
            settings.display()
        )),
    }
}

#[cfg(not(unix))]
fn restrict_settings_perms(_settings: &Path) -> Option<String> {
    None
}

fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("write path has no parent: {}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "write file name not UTF-8".to_string())?;
    let tmp = parent.join(format!(".{}.tmp", file_name));
    fs::write(&tmp, content).map_err(|e| format!("Write tmp failed: {}", e))?;
    fs::rename(&tmp, path).map_err(|e| {
        // Best-effort cleanup of orphan tmp.
        let _ = fs::remove_file(&tmp);
        format!("Atomic rename failed: {}", e)
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Hook detection
// ---------------------------------------------------------------------------

/// Tokenize a shell command string respecting single-quoted segments
/// AND the `'\''` close-escape-reopen idiom that `posix_single_quote`
/// emits when a path contains a single quote. Outside of single quotes,
/// a backslash escapes the next character (the standard POSIX rule),
/// which is what makes the idiom round-trip cleanly:
///
///   posix_single_quote("foo'bar")  -> 'foo'\''bar'
///   shell_tokenize("'foo'\\''bar'") -> ["foo'bar"]
///
/// Without backslash handling, a hook installed on a path containing a
/// single quote would not be re-detected by `is_our_hook` and a second
/// install would create a duplicate entry. This is rare in practice but
/// the cost of getting it right is low.
fn shell_tokenize(cmd: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_squote = false;
    let mut chars = cmd.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => in_squote = !in_squote,
            '\\' if !in_squote => {
                // Outside single quotes: backslash escapes the next char.
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            ' ' | '\t' if !in_squote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Strict match: a hook is "ours" only if the explicit id matches OR the
/// command is the simple two-token form ending in `adaptive-guard.sh`.
/// Never match by substring alone — a user-composed command like
/// `bash adaptive-guard.sh && bash other.sh` must NOT be removed.
///
/// The simple-form fallback exists because older installs of this hook
/// did not write an `id` field. We must still detect them so a CLI
/// install + dashboard reinstall is idempotent.
fn is_our_hook(h: &Value) -> bool {
    if h.get("id").and_then(|v| v.as_str()) == Some(HOOK_ID) {
        return true;
    }
    let cmd = match h.get("command").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return false,
    };
    if !cmd.contains("adaptive-guard.sh") {
        return false;
    }
    if ["&&", "||", ";", "|", "$(", "`"].iter().any(|t| cmd.contains(t)) {
        return false;
    }
    let parts = shell_tokenize(cmd);
    parts.len() == 2 && parts[1].ends_with("adaptive-guard.sh")
}

fn detect_installed(settings: &Value) -> bool {
    let stop = match settings
        .get("hooks")
        .and_then(|h| h.get("Stop"))
        .and_then(|s| s.as_array())
    {
        Some(a) => a,
        None => return false,
    };
    for group in stop {
        if let Some(hooks) = group.get("hooks").and_then(|h| h.as_array()) {
            for h in hooks {
                if is_our_hook(h) {
                    return true;
                }
            }
        }
    }
    false
}

/// Mutate `value` to add the adaptive-guard Stop hook entry and the
/// `ADAPTIVE_GUARD_CONFIG` env var. Pure (no I/O); see `install_inner`
/// for the file-touching wrapper. Returns Err if `value` is not a JSON
/// object or has incompatible shape under `hooks` / `env`.
fn apply_install_to_value(
    value: &mut Value,
    command: String,
    config_posix: String,
) -> Result<(), String> {
    let obj = value
        .as_object_mut()
        .ok_or_else(|| "settings must be a JSON object at the top level".to_string())?;

    let hooks_entry = obj.entry("hooks".to_string()).or_insert_with(|| json!({}));
    let hooks_obj = hooks_entry.as_object_mut().ok_or_else(|| {
        "settings.json `hooks` exists but is not a JSON object — refusing to modify".to_string()
    })?;
    let stop_entry = hooks_obj
        .entry("Stop".to_string())
        .or_insert_with(|| json!([]));
    let stop_arr = stop_entry.as_array_mut().ok_or_else(|| {
        "settings.json `hooks.Stop` exists but is not a JSON array — refusing to modify".to_string()
    })?;

    stop_arr.push(json!({
        "hooks": [{
            "id": HOOK_ID,
            "type": "command",
            "command": command,
            "timeout": 30,
        }]
    }));

    let env_entry = obj.entry("env".to_string()).or_insert_with(|| json!({}));
    let env_obj = env_entry.as_object_mut().ok_or_else(|| {
        "settings.json `env` exists but is not a JSON object — refusing to modify".to_string()
    })?;
    env_obj.insert(
        "ADAPTIVE_GUARD_CONFIG".to_string(),
        Value::String(config_posix),
    );

    Ok(())
}

/// Mutate `value` to remove every adaptive-guard hook entry and the
/// `ADAPTIVE_GUARD_CONFIG` env var. Returns `true` if anything actually
/// changed.
///
/// Round-trip invariant: `install → uninstall` on a previously empty
/// settings file produces an empty file again, not `{"hooks":{"Stop":[]},"env":{}}`.
/// To preserve that, this function cascades cleanup of empty containers
/// it created on install:
///
/// - Hook groups whose `hooks` array becomes empty after removal are dropped.
/// - `hooks.Stop` is removed if it is left empty after dropping our groups.
/// - `hooks` is removed if it is left empty after `Stop` is gone.
/// - `env` is removed if it is left empty after `ADAPTIVE_GUARD_CONFIG` is gone.
///
/// Third-party hooks under any event (PreToolUse, Stop with non-our entries,
/// etc.) and any other key in `env` are preserved untouched.
fn apply_uninstall_from_value(value: &mut Value) -> bool {
    let mut changed = false;
    let obj = match value.as_object_mut() {
        Some(o) => o,
        None => return false,
    };

    // ---- Process hooks.Stop and cascade cleanup. ----
    let hooks_now_empty = if let Some(hooks_obj) = obj
        .get_mut("hooks")
        .and_then(|h| h.as_object_mut())
    {
        let stop_now_empty = if let Some(stop_arr) = hooks_obj
            .get_mut("Stop")
            .and_then(|s| s.as_array_mut())
        {
            let mut new_groups: Vec<Value> = Vec::with_capacity(stop_arr.len());
            for group in stop_arr.drain(..) {
                let mut group_obj = match group {
                    Value::Object(m) => m,
                    other => {
                        new_groups.push(other);
                        continue;
                    }
                };
                if let Some(hooks_arr) =
                    group_obj.get_mut("hooks").and_then(|h| h.as_array_mut())
                {
                    let before = hooks_arr.len();
                    hooks_arr.retain(|h| !is_our_hook(h));
                    if hooks_arr.len() != before {
                        changed = true;
                    }
                }
                let keep = group_obj
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .is_some_and(|a| !a.is_empty());
                if keep {
                    new_groups.push(Value::Object(group_obj));
                } else {
                    changed = true;
                }
            }
            *stop_arr = new_groups;
            stop_arr.is_empty()
        } else {
            false
        };
        if stop_now_empty {
            hooks_obj.remove("Stop");
            changed = true;
        }
        hooks_obj.is_empty()
    } else {
        false
    };
    if hooks_now_empty {
        obj.remove("hooks");
        changed = true;
    }

    // ---- Process env and cascade cleanup. ----
    let env_now_empty = if let Some(env_obj) = obj.get_mut("env").and_then(|e| e.as_object_mut())
    {
        if env_obj.remove("ADAPTIVE_GUARD_CONFIG").is_some() {
            changed = true;
        }
        env_obj.is_empty()
    } else {
        false
    };
    if env_now_empty {
        obj.remove("env");
        changed = true;
    }

    changed
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn status() -> HookStatus {
    let cfg = config_path();
    let settings = settings_path();
    let cfg_display = cfg
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(home not resolvable)".to_string());
    let settings_display = settings
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(home not resolvable)".to_string());

    let (repo_root, repo_err) = match read_repo_root() {
        Ok(p) => (Some(p.display().to_string()), None),
        Err(e) => (None, Some(e)),
    };

    // Three states:
    //   - settings file does not exist: installed=false, no error (normal first run).
    //   - settings file exists but is not valid JSON: installed=false, error set
    //     ("we cannot tell" — refuse to claim either way).
    //   - settings file exists and parses: installed reflects detection.
    // Conflating broken-JSON with not-installed would lead the UI to offer
    // an "Install hook" button on a corrupted file — install_inner refuses
    // anyway, but the UX would be misleading.
    let (installed, parse_err) = match settings.as_ref() {
        Some(p) if p.exists() => match fs::read_to_string(p) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(v) => (detect_installed(&v), None),
                Err(e) => (
                    false,
                    Some(format!(
                        "settings.json at {} is not valid JSON ({}). Cannot determine hook state. Restore from a backup or fix manually.",
                        p.display(),
                        e
                    )),
                ),
            },
            Err(e) => (
                false,
                Some(format!("Could not read {}: {}", p.display(), e)),
            ),
        },
        _ => (false, None),
    };

    // If `read_repo_root` errored, that takes precedence in `error` because
    // the user must fix it before any install can succeed. Parse errors
    // override only if the repo root resolved fine.
    let error = repo_err.or(parse_err);

    HookStatus {
        installed,
        repo_root,
        config_path: cfg_display,
        settings_path: settings_display,
        error,
    }
}

pub fn install() -> InstallResult {
    match install_inner() {
        Ok(r) => r,
        Err(e) => InstallResult {
            ok: false,
            message: e,
            backup_path: None,
        },
    }
}

fn install_inner() -> Result<InstallResult, String> {
    let repo_root = read_repo_root()?;
    let hook_script = hook_script_path(&repo_root);
    if !hook_script.is_file() {
        return Err(format!(
            "Hook script not found at {}. The repo may be incomplete.",
            hook_script.display()
        ));
    }

    let bash_posix = bash_command_path();
    let hook_posix = to_msys_posix(&hook_script.display().to_string());
    let config_posix = to_msys_posix(&default_config_path(&repo_root).display().to_string());

    assert_safe(&bash_posix, "bash path")?;
    assert_safe(&hook_posix, "hook script path")?;
    // config_posix is only embedded in JSON (not in a shell command), so
    // shell-metacharacter safety is not required there.

    let command = format!(
        "{} {}",
        posix_single_quote(&bash_posix),
        posix_single_quote(&hook_posix)
    );

    let settings = settings_path().ok_or_else(|| "Home directory not resolvable".to_string())?;
    if let Some(parent) = settings.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {}: {}", parent.display(), e))?;
    }

    // If we have to create a fresh settings.json, also tighten its
    // permissions on Unix. If chmod fails we surface a warning (the
    // file would otherwise be 0644, world-readable, and may eventually
    // contain credentials that Claude Code itself adds). Wrapped in
    // `restrict_settings_perms` so the cfg(unix)/cfg(not(unix)) split
    // stays out of the install flow and we don't need a `mut` binding
    // that is dead on Windows (clippy's unused_mut would reject it).
    let chmod_warning: Option<String> = if !settings.exists() {
        fs::write(&settings, "{}\n").map_err(|e| {
            format!(
                "Could not create empty settings.json at {}: {}",
                settings.display(),
                e
            )
        })?;
        restrict_settings_perms(&settings)
    } else {
        None
    };

    let backup = make_backup(&settings)?;
    let backup_str = backup.display().to_string();

    let original = fs::read_to_string(&settings).map_err(|e| format!("Read failed: {}", e))?;
    let mut value: Value = serde_json::from_str(&original).map_err(|e| {
        format!(
            "settings.json is not valid JSON: {}. A backup was already created at {}.",
            e, backup_str
        )
    })?;
    if !value.is_object() {
        return Err(format!(
            "settings.json must be a JSON object at the top level. Backup at {}.",
            backup_str
        ));
    }

    if detect_installed(&value) {
        let msg = match &chmod_warning {
            Some(w) => format!("Already installed — no changes. {}", w),
            None => "Already installed — no changes.".to_string(),
        };
        return Ok(InstallResult {
            ok: true,
            message: msg,
            backup_path: Some(backup_str),
        });
    }

    apply_install_to_value(&mut value, command, config_posix)?;

    let serialized =
        serde_json::to_string_pretty(&value).map_err(|e| format!("Serialize failed: {}", e))?
            + "\n";

    atomic_write(&settings, &serialized)?;

    let base_msg =
        "Installed adaptive-guard hook. Restart Claude Code for it to take effect.";
    let msg = match chmod_warning {
        Some(w) => format!("{} {}", base_msg, w),
        None => base_msg.to_string(),
    };
    Ok(InstallResult {
        ok: true,
        message: msg,
        backup_path: Some(backup_str),
    })
}

pub fn uninstall() -> InstallResult {
    match uninstall_inner() {
        Ok(r) => r,
        Err(e) => InstallResult {
            ok: false,
            message: e,
            backup_path: None,
        },
    }
}

fn uninstall_inner() -> Result<InstallResult, String> {
    let settings = settings_path().ok_or_else(|| "Home directory not resolvable".to_string())?;
    if !settings.exists() {
        return Ok(InstallResult {
            ok: true,
            message: format!(
                "No settings.json at {}. Nothing to remove.",
                settings.display()
            ),
            backup_path: None,
        });
    }

    let backup = make_backup(&settings)?;
    let backup_str = backup.display().to_string();

    let original = fs::read_to_string(&settings).map_err(|e| format!("Read failed: {}", e))?;
    let mut value: Value = serde_json::from_str(&original).map_err(|e| {
        format!(
            "settings.json is not valid JSON: {}. Backup at {}.",
            e, backup_str
        )
    })?;

    let changed = apply_uninstall_from_value(&mut value);

    if changed {
        let serialized = serde_json::to_string_pretty(&value)
            .map_err(|e| format!("Serialize failed: {}", e))?
            + "\n";
        atomic_write(&settings, &serialized)?;
        Ok(InstallResult {
            ok: true,
            message: "adaptive-guard removed from settings.json.".to_string(),
            backup_path: Some(backup_str),
        })
    } else {
        Ok(InstallResult {
            ok: true,
            message: "adaptive-guard not found in settings.json — no changes.".to_string(),
            backup_path: Some(backup_str),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msys_posix_converts_drive_letter() {
        assert_eq!(to_msys_posix("C:\\Users\\foo"), "/c/Users/foo");
        assert_eq!(to_msys_posix("D:/path/to/file"), "/d/path/to/file");
    }

    #[test]
    fn msys_posix_passes_through_posix_paths() {
        assert_eq!(to_msys_posix("/usr/bin/bash"), "/usr/bin/bash");
        assert_eq!(to_msys_posix("/c/already/converted"), "/c/already/converted");
    }

    #[test]
    fn assert_safe_rejects_newline() {
        assert!(assert_safe("/path/with\nnewline", "test").is_err());
    }

    #[test]
    fn assert_safe_rejects_backslash() {
        // A stray backslash signals to_msys_posix didn't run.
        assert!(assert_safe(r"C:\Users\foo\repo", "test").is_err());
    }

    #[test]
    fn assert_safe_accepts_clean_path() {
        assert!(assert_safe("/c/Users/foo/repo/hooks/adaptive-guard.sh", "test").is_ok());
    }

    #[test]
    fn assert_safe_accepts_path_with_shell_meta_inside_quotes() {
        // Inside posix_single_quote these are all literal — accepting
        // them removes pointless friction for normal Windows paths.
        assert!(assert_safe("/c/Users/Foo & Bar/repo (backup)/adaptive-guard.sh", "test").is_ok());
        assert!(assert_safe("/c/Users/$home/repo!/adaptive-guard.sh", "test").is_ok());
    }

    #[test]
    fn posix_single_quote_escapes_inner_quote() {
        assert_eq!(posix_single_quote("foo'bar"), r"'foo'\''bar'");
    }

    #[test]
    fn detect_installed_finds_by_id() {
        let v: Value = serde_json::from_str(
            r#"{"hooks":{"Stop":[{"hooks":[{"id":"adaptive-guard","type":"command","command":"x"}]}]}}"#,
        )
        .unwrap();
        assert!(detect_installed(&v));
    }

    #[test]
    fn detect_installed_skips_composed_command() {
        let v: Value = serde_json::from_str(
            r#"{"hooks":{"Stop":[{"hooks":[{"command":"bash adaptive-guard.sh && bash other.sh"}]}]}}"#,
        )
        .unwrap();
        assert!(!detect_installed(&v));
    }

    #[test]
    fn detect_installed_finds_simple_form() {
        let v: Value = serde_json::from_str(
            r#"{"hooks":{"Stop":[{"hooks":[{"command":"bash /repo/hooks/adaptive-guard.sh"}]}]}}"#,
        )
        .unwrap();
        assert!(detect_installed(&v));
    }

    #[test]
    fn detect_installed_finds_quoted_paths_with_spaces() {
        // Real-world case: install made by an older CLI that did not set
        // `id`, on a Windows path that contains spaces. The whole command
        // string after JSON-decoding is two single-quoted segments.
        let v: Value = serde_json::from_str(
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"'/c/Program Files/Git/usr/bin/bash' '/c/Users/Sthiven R/Desktop/claude-code-adaptive-guard/hooks/adaptive-guard.sh'","timeout":30}]}]}}"#,
        )
        .unwrap();
        assert!(detect_installed(&v), "must detect quoted-paths simple form");
    }

    #[test]
    fn shell_tokenize_handles_quoted_spaces() {
        let tokens = shell_tokenize("'/c/Program Files/bash' '/c/Users/x R/repo/file.sh'");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], "/c/Program Files/bash");
        assert_eq!(tokens[1], "/c/Users/x R/repo/file.sh");
    }

    #[test]
    fn shell_tokenize_handles_unquoted() {
        let tokens = shell_tokenize("/bin/bash /repo/hooks/adaptive-guard.sh");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[1], "/repo/hooks/adaptive-guard.sh");
    }

    #[test]
    fn posix_single_quote_round_trips_via_shell_tokenize() {
        // End-to-end safety net for paths containing a single quote.
        // posix_single_quote emits 'foo'\''bar' which shell_tokenize
        // must decode back to "foo'bar" — otherwise reinstall on a
        // path with `'` would silently create a duplicate hook entry.
        let path = "/c/Users/Foo's project/repo/hooks/adaptive-guard.sh";
        let quoted_bash = posix_single_quote("/usr/bin/bash");
        let quoted_path = posix_single_quote(path);
        let cmd = format!("{} {}", quoted_bash, quoted_path);
        let tokens = shell_tokenize(&cmd);
        assert_eq!(tokens.len(), 2, "tokens: {:?}", tokens);
        assert_eq!(tokens[0], "/usr/bin/bash");
        assert_eq!(tokens[1], path, "single-quote escape did not round-trip");
    }

    #[test]
    fn assert_safe_accepts_path_with_apostrophe() {
        // Apostrophes are legitimate in user paths (e.g. "Sthiven's
        // project") and posix_single_quote handles them safely. The
        // round-trip is covered above; this test pins the contract
        // that assert_safe does not over-reject them.
        assert!(
            assert_safe("/c/Users/Foo's project/adaptive-guard.sh", "test").is_ok()
        );
    }

    #[test]
    fn detect_not_installed_when_empty() {
        let v: Value = serde_json::from_str(r#"{}"#).unwrap();
        assert!(!detect_installed(&v));
    }

    #[test]
    fn timestamp_format_is_15_chars() {
        let t = timestamp_utc().expect("system clock should be after epoch");
        assert_eq!(t.len(), 15); // YYYYMMDD-HHMMSS
        assert_eq!(t.chars().nth(8), Some('-'));
    }

    #[test]
    fn days_to_ymd_known_dates() {
        // 2024-02-29 (leap day) is day 19782 since 1970-01-01
        assert_eq!(days_to_ymd(19782), (2024, 2, 29));
        // 1970-01-01 itself
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        // 2000-01-01 (century leap year)
        assert_eq!(days_to_ymd(10957), (2000, 1, 1));
    }

    #[test]
    fn apply_install_on_empty_object() {
        let mut v: Value = serde_json::from_str("{}").unwrap();
        apply_install_to_value(
            &mut v,
            "'/usr/bin/bash' '/repo/hooks/adaptive-guard.sh'".to_string(),
            "/repo/config/default.json".to_string(),
        )
        .unwrap();
        assert!(detect_installed(&v));
        assert_eq!(
            v["env"]["ADAPTIVE_GUARD_CONFIG"].as_str(),
            Some("/repo/config/default.json")
        );
    }

    #[test]
    fn apply_install_preserves_third_party_pretooluse() {
        let mut v: Value = serde_json::from_str(
            r#"{"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"command":"third-party"}]}]}}"#,
        )
        .unwrap();
        apply_install_to_value(
            &mut v,
            "'/usr/bin/bash' '/repo/hooks/adaptive-guard.sh'".to_string(),
            "/repo/config/default.json".to_string(),
        )
        .unwrap();
        assert!(detect_installed(&v));
        let pre = v["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 1);
        assert_eq!(pre[0]["hooks"][0]["command"].as_str(), Some("third-party"));
    }

    #[test]
    fn round_trip_install_then_uninstall_preserves_third_party() {
        let original = r#"{"env":{"FOO":"bar"},"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"command":"third-party"}]}]}}"#;
        let mut v: Value = serde_json::from_str(original).unwrap();
        apply_install_to_value(
            &mut v,
            "'/usr/bin/bash' '/repo/hooks/adaptive-guard.sh'".to_string(),
            "/repo/config/default.json".to_string(),
        )
        .unwrap();
        let changed = apply_uninstall_from_value(&mut v);
        assert!(changed);
        assert!(!detect_installed(&v));
        // Pre-existing keys untouched.
        assert_eq!(v["env"]["FOO"].as_str(), Some("bar"));
        assert!(v["env"].get("ADAPTIVE_GUARD_CONFIG").is_none());
        let pre = v["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 1);
        // Empty `Stop` from our removal is cleaned up — no orphan empty arrays.
        assert!(v["hooks"].get("Stop").is_none(),
            "empty Stop should be removed after uninstall, got: {}", v);
    }

    #[test]
    fn round_trip_on_empty_object_returns_to_empty() {
        // Install + uninstall on `{}` must produce `{}` again — no orphan
        // `{"hooks":{"Stop":[]},"env":{}}` left behind.
        let mut v: Value = serde_json::from_str("{}").unwrap();
        apply_install_to_value(
            &mut v,
            "'/usr/bin/bash' '/repo/adaptive-guard.sh'".to_string(),
            "/repo/config/default.json".to_string(),
        )
        .unwrap();
        let changed = apply_uninstall_from_value(&mut v);
        assert!(changed);
        let obj = v.as_object().expect("root is object");
        assert!(obj.is_empty(),
            "round-trip on empty object should produce empty object, got: {}", v);
    }

    #[test]
    fn install_preserves_top_level_key_order() {
        // The `preserve_order` feature on serde_json exists precisely to
        // keep the user's settings.json layout stable across install/uninstall.
        // This regression test fails if serde_json is rebuilt without the
        // feature OR the install logic accidentally reorders keys.
        let original = r#"{"theme":"dark","permissions":{"x":true},"language":"es"}"#;
        let mut v: Value = serde_json::from_str(original).unwrap();
        apply_install_to_value(
            &mut v,
            "'/usr/bin/bash' '/repo/adaptive-guard.sh'".to_string(),
            "/repo/config/default.json".to_string(),
        )
        .unwrap();
        let serialized = serde_json::to_string(&v).unwrap();
        let theme = serialized.find(r#""theme""#).expect("theme present");
        let perms = serialized.find(r#""permissions""#).expect("permissions present");
        let lang = serialized.find(r#""language""#).expect("language present");
        assert!(theme < perms, "theme must precede permissions: {}", serialized);
        assert!(perms < lang, "permissions must precede language: {}", serialized);
    }

    #[test]
    fn apply_uninstall_preserves_unrelated_stop_hooks_in_same_group() {
        let mut v: Value = serde_json::from_str(
            r#"{"hooks":{"Stop":[{"hooks":[{"id":"adaptive-guard","type":"command","command":"x"},{"id":"other","command":"unrelated"}]}]}}"#,
        )
        .unwrap();
        let changed = apply_uninstall_from_value(&mut v);
        assert!(changed);
        let stop = v["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        let group_hooks = stop[0]["hooks"].as_array().unwrap();
        assert_eq!(group_hooks.len(), 1);
        assert_eq!(group_hooks[0]["id"].as_str(), Some("other"));
    }

    #[test]
    fn apply_uninstall_no_op_on_truly_empty() {
        // No containers at all → genuine no-op.
        let mut v: Value = serde_json::from_str("{}").unwrap();
        assert!(!apply_uninstall_from_value(&mut v));
    }

    #[test]
    fn apply_uninstall_no_op_when_only_third_party() {
        // Settings has third-party hooks only — uninstall must touch nothing.
        let mut v: Value = serde_json::from_str(
            r#"{"hooks":{"Stop":[{"hooks":[{"id":"other","command":"x"}]}]},"env":{"FOO":"bar"}}"#,
        )
        .unwrap();
        assert!(!apply_uninstall_from_value(&mut v));
        // Confirm structure intact.
        assert_eq!(v["env"]["FOO"].as_str(), Some("bar"));
        assert_eq!(v["hooks"]["Stop"][0]["hooks"][0]["id"].as_str(), Some("other"));
    }

    #[test]
    fn apply_uninstall_cleans_up_orphan_empty_containers() {
        // Settings has only an empty Stop array (orphan from a previous bug
        // or manual edit) — uninstall should clean it up cascadingly.
        let mut v: Value = serde_json::from_str(r#"{"hooks":{"Stop":[]}}"#).unwrap();
        assert!(apply_uninstall_from_value(&mut v));
        assert!(v.as_object().unwrap().is_empty());
    }

    #[test]
    fn apply_install_rejects_non_object_root() {
        let mut v: Value = serde_json::from_str(r#"["array root"]"#).unwrap();
        let r = apply_install_to_value(&mut v, "x".to_string(), "y".to_string());
        assert!(r.is_err());
    }
}
