// Tiny i18n layer. Two locales (en/es), localStorage persistence,
// reactive store integrated with Svelte. No external dep.
//
// Usage from components:
//   import { t, lang, fmt } from "../lib/i18n";
//   <button>{$t.welcome.install_button}</button>
//   <span>{fmt($t.app.load_more, { n: 50 })}</span>
//   <button onclick={() => $lang = "es"}>Español</button>
//
// Adding a new key: add it to `en` first. TypeScript will then force
// you to add it to `es` (or any future locale) via the `Dict` type.

import { derived, writable } from "svelte/store";

// Re-export `fmt` for backward compatibility — its real home is
// `strings.ts` (no locale dependency). New consumers should import from
// `./strings` directly.
export { fmt } from "./strings";

export type Lang = "en" | "es";

const STORAGE_KEY = "adaptive-guard.language";

function loadInitial(): Lang {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "en" || saved === "es") return saved;
  } catch {
    /* localStorage unavailable (private mode, sandbox, etc.) */
  }
  // Detect from browser/OS locale. Anything starting with `es` (es, es-VE,
  // es-419, es-ES, etc.) gets Spanish; everything else falls back to English.
  if (typeof navigator !== "undefined" && navigator.language) {
    if (navigator.language.toLowerCase().startsWith("es")) return "es";
  }
  return "en";
}

export const lang = writable<Lang>(loadInitial());

lang.subscribe((v) => {
  try {
    localStorage.setItem(STORAGE_KEY, v);
  } catch {
    /* best-effort persistence */
  }
});

// English is the source of truth; the type of `es` is derived from it,
// so missing keys become a compile error.
const en = {
  app: {
    error: "Error:",
    no_telemetry_yet: "No telemetry file yet.",
    hook_installed_no_decisions:
      "The hook is installed but has not recorded any decisions on this machine yet. Use Claude Code — this window will populate automatically.",
    hook_not_installed_lead:
      "The guard has not recorded any decisions on this machine. Install the Stop hook from the",
    hook_not_installed_link: "settings panel",
    hook_not_installed_or: "or run",
    hook_not_installed_then: ", then use Claude Code.",
    looking_at: "Looking at:",
    recent_decisions: "Recent decisions",
    shown: "shown",
    of_loaded: "of {n} loaded",
    refresh: "Refresh",
    loading: "Loading...",
    no_decisions_yet: "No decisions to display yet.",
    no_match_filters: "No decisions match the current filters.",
    load_more: "Load more ({n} more)",
    minimize_to_tray: "Close window → minimize to tray",
    loaded_n_of_total: "Loaded {n} of {total}",
    settings_label: "Settings",
  },
  welcome: {
    badge: "first run",
    lede:
      "Quality control layer for Claude Code. The hook scores every prompt and forces a deeper response when the original is too shallow. Decisions land here in real time.",
    step1_title: "Install the Stop hook",
    step1_body_lead: "Writes a single entry to",
    step1_body_tail:
      ". A timestamped backup is saved first. Reversible from the gear menu.",
    step2_title: "Restart Claude Code",
    step2_body:
      "Hooks load at startup. Quit and reopen so the new configuration takes effect.",
    step3_title: "Use Claude Code",
    step3_body:
      "Send any prompt. The first decision will appear here within a second of the response finishing. The window can stay minimized in the system tray.",
    install_button: "Install hook",
    installing: "Installing…",
    installed: "Installed.",
    install_failed: "Failed.",
    backup: "Backup:",
    cannot_install: "Cannot install yet.",
    cannot_install_hint_lead: "Run",
    cannot_install_hint_tail: "from the repo first, then reload this window.",
    skip_question: "Already installed via CLI?",
    skip: "Skip welcome",
  },
  settings: {
    title: "Settings",
    close: "Close",
    hook_status: "Hook status",
    installed: "Installed",
    not_installed: "Not installed",
    install_hook: "Install hook",
    installing: "Installing…",
    uninstall: "Uninstall",
    confirm_uninstall_lead: "This removes the",
    confirm_uninstall_entry: "entry from",
    confirm_uninstall_tail:
      ". A backup is created. Other hooks are preserved.",
    confirm_yes: "Yes, remove",
    cancel: "Cancel",
    removing: "Removing…",
    paths_section: "Paths",
    repo: "Repo",
    cli_config: "CLI config",
    settings_path: "Settings",
    not_configured: "(not configured)",
    cannot_install: "Cannot install yet.",
    ok: "OK.",
    failed: "Failed.",
    backup: "Backup:",
    language_section: "Language",
    language_english: "English",
    language_spanish: "Español",
    theme_section: "Theme",
    theme_dark: "Dark",
    theme_light: "Light",
    theme_auto: "Auto",
  },
  stats: {
    profile: "profile:",
    no_decisions:
      "No decisions recorded yet. Run Claude Code with the hook installed, then come back here.",
    total: "Total",
    blocks: "Blocks",
    deep_allowed: "Deep allowed",
    simple_skipped: "Simple (skipped)",
    tokens_in_out: "Tokens in / out (~)",
    chars_estimate: "chars/4 estimate",
    chars_estimate_hint: "chars/4 heuristic; diverges for non-Latin scripts",
    since: "Since",
    last: "Last:",
    block_tag: "block",
    deep_tag: "deep",
    avg_complexity: "avg complexity",
    depth: "depth",
    missing: "missing",
    loading: "Loading…",
  },
  histogram: {
    complexity_distribution: "Complexity distribution",
    depth_distribution: "Depth distribution",
    no_data: "no data yet",
    records: "records",
    below_threshold: "below threshold",
    at_or_above: "at or above",
  },
  filter: {
    decision: "Decision",
    block: "Block",
    deep: "Deep",
    simple: "Simple",
    time: "Time",
    all_time: "All time",
    last_7_days: "Last 7 days",
    today: "Today",
    last_hour: "Last hour",
    session: "Session",
    session_placeholder: "session id prefix…",
    clear: "Clear",
  },
  live: {
    on: "LIVE",
    off: "PAUSED",
    on_tooltip: "Live monitoring (watching telemetry file)",
    off_tooltip: "Watcher paused",
  },
  decision: {
    block: "BLOCK",
    deep: "DEEP",
    skip: "SKIP",
    session: "Session",
    profile: "Profile",
    prompt: "Prompt",
    response: "Response",
    chars: "chars",
    thresholds: "Thresholds",
    complexity_breakdown: "Complexity breakdown",
    depth_breakdown: "Depth breakdown",
    missing_aspects_head: "Missing aspects flagged by guard",
    missing_n: "{n} missing",
  },
  breakdown: {
    not_evaluated: "not evaluated",
    structural: "structural:",
    semantic: "semantic:",
    blend: "blend:",
    semantic_short: "sem",
    structural_short: "struct",
    axes_section: "Axes",
    signals_section: "Signals detected",
    pts: "pts",
  },
  time: {
    just_now: "just now",
    seconds_ago: "{n}s ago",
    minutes_ago: "{n}m ago",
    hours_ago: "{n}h ago",
    days_ago: "{n}d ago",
    yesterday: "yesterday",
    today: "today",
  },
};

export type Dict = typeof en;

// Spanish neutro venezolano: tuteo, sin voseo, con tildes correctas.
// Términos técnicos (hook, backup, CLI, prompt, session) se preservan
// porque son los mismos que aparecen en el código y la documentación.
//
// `satisfies Dict` (not `: Dict`) — preserves narrow literal types so
// typos in existing keys are caught structurally, not just missing keys.
const es = {
  app: {
    error: "Error:",
    no_telemetry_yet: "Aún no hay archivo de telemetría.",
    hook_installed_no_decisions:
      "El hook está instalado pero todavía no ha registrado ninguna decisión en esta máquina. Usa Claude Code y esta ventana se actualizará automáticamente.",
    hook_not_installed_lead:
      "El guardia no ha registrado decisiones en esta máquina. Instala el hook Stop desde el",
    hook_not_installed_link: "panel de configuración",
    hook_not_installed_or: "o ejecuta",
    hook_not_installed_then: ", luego usa Claude Code.",
    looking_at: "Buscando en:",
    recent_decisions: "Decisiones recientes",
    shown: "mostradas",
    of_loaded: "de {n} cargadas",
    refresh: "Actualizar",
    loading: "Cargando...",
    no_decisions_yet: "Aún no hay decisiones para mostrar.",
    no_match_filters: "Ninguna decisión coincide con los filtros actuales.",
    load_more: "Cargar más ({n} más)",
    minimize_to_tray: "Cerrar ventana → minimizar a la bandeja",
    loaded_n_of_total: "{n} de {total} cargadas",
    settings_label: "Configuración",
  },
  welcome: {
    badge: "primera vez",
    lede:
      "Capa de control de calidad para Claude Code. El hook puntúa cada prompt y fuerza una respuesta más profunda cuando la original es superficial. Las decisiones aparecen aquí en tiempo real.",
    step1_title: "Instalar el hook Stop",
    step1_body_lead: "Agrega una sola entrada a",
    step1_body_tail:
      ". Se guarda primero un backup con marca de tiempo. Reversible desde el menú de configuración.",
    step2_title: "Reiniciar Claude Code",
    step2_body:
      "Los hooks se cargan al arrancar. Cierra y vuelve a abrir Claude Code para que la nueva configuración tome efecto.",
    step3_title: "Usa Claude Code",
    step3_body:
      "Envía cualquier prompt. La primera decisión aparecerá aquí en menos de un segundo después de que termine la respuesta. La ventana puede quedarse minimizada en la bandeja del sistema.",
    install_button: "Instalar hook",
    installing: "Instalando…",
    installed: "Instalado.",
    install_failed: "Falló.",
    backup: "Backup:",
    cannot_install: "Aún no se puede instalar.",
    cannot_install_hint_lead: "Ejecuta",
    cannot_install_hint_tail: "desde el repo primero, luego recarga esta ventana.",
    skip_question: "¿Ya lo instalaste vía CLI?",
    skip: "Saltar bienvenida",
  },
  settings: {
    title: "Configuración",
    close: "Cerrar",
    hook_status: "Estado del hook",
    installed: "Instalado",
    not_installed: "No instalado",
    install_hook: "Instalar hook",
    installing: "Instalando…",
    uninstall: "Desinstalar",
    confirm_uninstall_lead: "Esto elimina la entrada",
    confirm_uninstall_entry: "de",
    confirm_uninstall_tail:
      ". Se crea un backup. Los otros hooks quedan intactos.",
    confirm_yes: "Sí, eliminar",
    cancel: "Cancelar",
    removing: "Eliminando…",
    paths_section: "Rutas",
    repo: "Repo",
    cli_config: "Config CLI",
    settings_path: "Settings",
    not_configured: "(no configurado)",
    cannot_install: "Aún no se puede instalar.",
    ok: "OK.",
    failed: "Falló.",
    backup: "Backup:",
    language_section: "Idioma",
    language_english: "English",
    language_spanish: "Español",
    theme_section: "Tema",
    theme_dark: "Oscuro",
    theme_light: "Claro",
    theme_auto: "Auto",
  },
  stats: {
    profile: "perfil:",
    no_decisions:
      "Aún no se han registrado decisiones. Usa Claude Code con el hook instalado y vuelve aquí.",
    total: "Total",
    blocks: "Bloqueos",
    deep_allowed: "Profundas permitidas",
    simple_skipped: "Simples (saltadas)",
    tokens_in_out: "Tokens entrada / salida (~)",
    chars_estimate: "estimación caracteres/4",
    chars_estimate_hint:
      "heurística caracteres/4; diverge para alfabetos no latinos",
    since: "Desde",
    last: "Última:",
    block_tag: "bloqueo",
    deep_tag: "profunda",
    avg_complexity: "complejidad media",
    depth: "profundidad",
    missing: "faltantes",
    loading: "Cargando…",
  },
  histogram: {
    complexity_distribution: "Distribución de complejidad",
    depth_distribution: "Distribución de profundidad",
    no_data: "aún sin datos",
    records: "registros",
    below_threshold: "bajo el umbral",
    at_or_above: "al umbral o por encima",
  },
  filter: {
    decision: "Decisión",
    block: "Bloqueo",
    deep: "Profunda",
    simple: "Simple",
    time: "Tiempo",
    all_time: "Todo el tiempo",
    last_7_days: "Últimos 7 días",
    today: "Hoy",
    last_hour: "Última hora",
    session: "Sesión",
    session_placeholder: "prefijo de session id…",
    clear: "Limpiar",
  },
  live: {
    on: "EN VIVO",
    off: "PAUSADO",
    on_tooltip: "Monitoreo en vivo (observando archivo de telemetría)",
    off_tooltip: "Watcher pausado",
  },
  decision: {
    block: "BLOQUEO",
    deep: "PROFUNDA",
    skip: "SALTADA",
    session: "Sesión",
    profile: "Perfil",
    prompt: "Prompt",
    response: "Respuesta",
    chars: "caracteres",
    thresholds: "Umbrales",
    complexity_breakdown: "Desglose de complejidad",
    depth_breakdown: "Desglose de profundidad",
    missing_aspects_head: "Aspectos faltantes detectados",
    missing_n: "{n} faltantes",
  },
  breakdown: {
    not_evaluated: "no evaluado",
    structural: "estructural:",
    semantic: "semántico:",
    blend: "mezcla:",
    semantic_short: "sem",
    structural_short: "estruct",
    axes_section: "Ejes",
    signals_section: "Señales detectadas",
    pts: "pts",
  },
  time: {
    just_now: "ahora mismo",
    seconds_ago: "hace {n}s",
    minutes_ago: "hace {n}m",
    hours_ago: "hace {n}h",
    days_ago: "hace {n}d",
    yesterday: "ayer",
    today: "hoy",
  },
} satisfies Dict;

const dicts: Record<Lang, Dict> = { en, es };

/** Reactive translation store. Read with `$t.section.key` in templates. */
export const t = derived(lang, ($lang) => dicts[$lang]);
