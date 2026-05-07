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
//
// UX-writing principle (Sprint 9): every string a non-technical user
// sees must read as plain language, not as identifiers from the code.
// `BLOCK` / `DEEP` / `SKIP` were the internal decision strings; the
// tags now read `RETRY` / `PASS` / `TRIVIAL` because that is what a
// human watching the dashboard actually wants to see. Tooltips
// (the `*_tooltip` keys) carry the technical detail for whoever
// hovers — invisible to the casual viewer, available to the curious.

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
    profile: "preset:",
    profile_tooltip:
      "Active scoring preset. Defines how strict the guard is. Switch via `adaptive-guard install --profile`.",
    no_decisions:
      "No decisions recorded yet. Run Claude Code with the hook installed, then come back here.",
    total: "Decisions seen",
    total_tooltip:
      "Every Claude Code response that flowed through the guard, including the trivial ones.",
    blocks: "Forced retry",
    blocks_tooltip:
      "The guard judged the prompt complex but the response shallow, blocked the stop, and asked Claude to rethink. Each one is a moment of saved frustration.",
    deep_allowed: "Passed",
    deep_allowed_tooltip:
      "Response had enough analytical depth — markdown structure, code blocks, varied phrasing — and was allowed through.",
    simple_skipped: "Trivial",
    simple_skipped_tooltip:
      "Prompt was too simple to deserve a deep response (e.g. \"what time is it?\"). The guard never evaluates the response in this case.",
    tokens_in_out: "Tokens (estimated)",
    tokens_in_out_tooltip:
      "Rough estimate of total tokens that flowed in and out, computed as chars/4. Diverges for non-Latin scripts.",
    chars_estimate: "chars/4 estimate",
    chars_estimate_hint: "chars/4 heuristic; diverges for non-Latin scripts",
    since: "First seen",
    since_tooltip:
      "When the very first decision in this telemetry file was recorded.",
    last: "Latest:",
    block_tag: "RETRY",
    deep_tag: "PASS",
    avg_complexity: "avg prompt complexity",
    depth: "response depth",
    missing: "gaps flagged",
    loading: "Loading…",
  },
  histogram: {
    complexity_distribution: "How complex were your prompts",
    complexity_distribution_tooltip:
      "Distribution of prompt-complexity scores (0-100). The taller the bar, the more prompts landed in that range. The vertical line marks the threshold below which the guard never blocks.",
    depth_distribution: "How deep were the responses",
    depth_distribution_tooltip:
      "Distribution of response-depth scores (0-100). The vertical line marks the threshold above which the guard never blocks, regardless of complexity.",
    no_data: "no data yet",
    records: "decisions",
    below_threshold: "below the bar",
    at_or_above: "at or above the bar",
  },
  filter: {
    decision: "Outcome",
    block: "Retry",
    deep: "Pass",
    simple: "Trivial",
    time: "Period",
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
    on_tooltip:
      "The guard is watching the telemetry file. New decisions appear here within a second of being recorded.",
    off_tooltip:
      "File watcher disabled. Hit Refresh to reload manually, or close and reopen the dashboard.",
  },
  decision: {
    block: "RETRY",
    deep: "PASS",
    skip: "TRIVIAL",
    session: "Session",
    profile: "Preset",
    prompt: "Prompt",
    response: "Response",
    chars: "chars",
    thresholds: "Thresholds",
    complexity_breakdown: "Why we judged the prompt complex",
    depth_breakdown: "Why we judged the response shallow or deep",
    missing_aspects_head: "What was missing",
    missing_n: "{n} gaps",
    score_complexity_short: "c",
    score_complexity_tooltip:
      "Prompt complexity (0-100). How challenging the prompt looked, computed from length, code-density, technical tokens, and structure.",
    score_depth_short: "d",
    score_depth_tooltip:
      "Response depth (0-100). How thorough the response looked, computed from markdown structure, code blocks, and lexical diversity. Dash means the prompt was trivial and the response was not evaluated.",
    context_show: "Show the prompt and response",
    context_hide: "Hide",
    context_loading: "Reading from transcript…",
    context_error_no_pointer:
      "This decision was recorded before the transcript-pointer feature shipped. The original prompt and response cannot be looked up.",
    context_error_missing:
      "The transcript file is no longer on disk — Claude Code may have rotated or deleted it. The decision metadata is preserved, but the original text is gone.",
    context_error_generic:
      "Could not load context. The transcript may be corrupted or the line was deleted.",
    context_prompt_label: "Prompt",
    context_response_label: "Response",
    feedback_section_label: "Was this decision useful?",
    feedback_useful: "Useful",
    feedback_useful_tooltip:
      "The decision agreed with your judgment — block was correct, or pass was correct.",
    feedback_annoying: "Annoying",
    feedback_annoying_tooltip:
      "The decision disagreed with your judgment — it blocked something fine, or let something shallow through.",
    feedback_clear: "Clear",
    feedback_clear_tooltip:
      "Remove your prior feedback for this decision.",
    feedback_note_placeholder: "Optional note: why?",
    feedback_saved: "Saved",
  },
  breakdown: {
    not_evaluated: "not evaluated",
    structural: "structural:",
    semantic: "semantic:",
    blend: "blend:",
    semantic_short: "sem",
    structural_short: "struct",
    axes_section: "Score breakdown",
    signals_section: "What was detected",
    pts: "pts",
    total_tooltip:
      "Final score after blending structural and (optional) semantic layers, clamped to 0-100.",
  },
  how_to_read: {
    title: "How to read this dashboard",
    show: "How to read this dashboard",
    hide: "Hide",
    intro:
      "Adaptive Guard sits between Claude Code and your screen. Every time Claude finishes a response, the guard rates two things on a 0–100 scale:",
    point_complexity:
      "How complex your prompt looked — based on length, technical tokens, code density, and how multi-part it was.",
    point_depth:
      "How thorough the response was — based on markdown structure, code blocks, and varied phrasing.",
    outcome_intro: "Then it picks one of three outcomes:",
    outcome_retry:
      "Complex prompt, shallow response → the guard blocked the stop and asked Claude to rethink.",
    outcome_pass:
      "Response had enough depth → it was allowed through. No interruption.",
    outcome_trivial:
      "Prompt was too simple to bother grading the response → it was allowed through with no evaluation.",
    privacy:
      "Nothing leaves your machine. The guard stores only counters and scores in the telemetry file — never the text of your prompts or Claude's responses.",
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
    profile: "preset:",
    profile_tooltip:
      "Preset de evaluación activo. Define qué tan estricto es el guard. Cambia con `adaptive-guard install --profile`.",
    no_decisions:
      "Aún no se han registrado decisiones. Usa Claude Code con el hook instalado y vuelve aquí.",
    total: "Decisiones registradas",
    total_tooltip:
      "Cada respuesta de Claude Code que pasó por el guard, incluyendo las triviales.",
    blocks: "Forzados a repensar",
    blocks_tooltip:
      "El guard juzgó el prompt complejo pero la respuesta superficial, bloqueó el cierre y le pidió a Claude repensar. Cada uno es un momento de frustración evitada.",
    deep_allowed: "Pasaron",
    deep_allowed_tooltip:
      "La respuesta tuvo suficiente profundidad analítica — estructura markdown, bloques de código, fraseo variado — y se dejó pasar.",
    simple_skipped: "Triviales",
    simple_skipped_tooltip:
      "El prompt era demasiado simple para merecer una respuesta profunda (ej. \"¿qué hora es?\"). El guard no evalúa la respuesta en este caso.",
    tokens_in_out: "Tokens (estimado)",
    tokens_in_out_tooltip:
      "Estimación aproximada del total de tokens que entraron y salieron, calculado como caracteres/4. Diverge para alfabetos no latinos.",
    chars_estimate: "estimación caracteres/4",
    chars_estimate_hint:
      "heurística caracteres/4; diverge para alfabetos no latinos",
    since: "Primera decisión",
    since_tooltip:
      "Cuándo se registró la primera decisión en este archivo de telemetría.",
    last: "Última:",
    block_tag: "REPENSAR",
    deep_tag: "PASA",
    avg_complexity: "complejidad promedio del prompt",
    depth: "profundidad de la respuesta",
    missing: "vacíos detectados",
    loading: "Cargando…",
  },
  histogram: {
    complexity_distribution: "Qué tan complejos fueron tus prompts",
    complexity_distribution_tooltip:
      "Distribución de los puntajes de complejidad del prompt (0-100). Cuanto más alta la barra, más prompts cayeron en ese rango. La línea vertical marca el umbral debajo del cual el guard nunca bloquea.",
    depth_distribution: "Qué tan profundas fueron las respuestas",
    depth_distribution_tooltip:
      "Distribución de los puntajes de profundidad de la respuesta (0-100). La línea vertical marca el umbral por encima del cual el guard nunca bloquea, sin importar la complejidad.",
    no_data: "aún sin datos",
    records: "decisiones",
    below_threshold: "bajo el umbral",
    at_or_above: "al umbral o por encima",
  },
  filter: {
    decision: "Resultado",
    block: "Repensar",
    deep: "Pasa",
    simple: "Trivial",
    time: "Período",
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
    on_tooltip:
      "El guard está observando el archivo de telemetría. Las nuevas decisiones aparecen aquí en menos de un segundo de ser registradas.",
    off_tooltip:
      "Watcher de archivo desactivado. Pulsa Actualizar para recargar manualmente, o cierra y reabre el dashboard.",
  },
  decision: {
    block: "REPENSAR",
    deep: "PASA",
    skip: "TRIVIAL",
    session: "Sesión",
    profile: "Preset",
    prompt: "Prompt",
    response: "Respuesta",
    chars: "caracteres",
    thresholds: "Umbrales",
    complexity_breakdown: "Por qué juzgamos el prompt complejo",
    depth_breakdown: "Por qué juzgamos la respuesta superficial o profunda",
    missing_aspects_head: "Qué faltó",
    missing_n: "{n} vacíos",
    score_complexity_short: "c",
    score_complexity_tooltip:
      "Complejidad del prompt (0-100). Qué tan desafiante se veía el prompt, calculado desde longitud, densidad de código, tokens técnicos y estructura.",
    score_depth_short: "p",
    score_depth_tooltip:
      "Profundidad de la respuesta (0-100). Qué tan completa se veía la respuesta, calculado desde estructura markdown, bloques de código y diversidad léxica. El guion significa que el prompt era trivial y la respuesta no se evaluó.",
    context_show: "Ver el prompt y la respuesta",
    context_hide: "Ocultar",
    context_loading: "Leyendo del transcript…",
    context_error_no_pointer:
      "Esta decisión se registró antes de que existiera el puntero al transcript. El prompt y la respuesta originales no se pueden recuperar.",
    context_error_missing:
      "El archivo del transcript ya no está en disco — Claude Code pudo haberlo rotado o eliminado. Los metadatos de la decisión se conservan, pero el texto original se perdió.",
    context_error_generic:
      "No se pudo cargar el contexto. El transcript puede estar corrupto o la línea fue eliminada.",
    context_prompt_label: "Prompt",
    context_response_label: "Respuesta",
    feedback_section_label: "¿Fue útil esta decisión?",
    feedback_useful: "Útil",
    feedback_useful_tooltip:
      "La decisión coincidió con tu juicio — el bloqueo fue correcto, o el pase fue correcto.",
    feedback_annoying: "Molesta",
    feedback_annoying_tooltip:
      "La decisión chocó con tu juicio — bloqueó algo bueno, o dejó pasar algo superficial.",
    feedback_clear: "Borrar",
    feedback_clear_tooltip:
      "Eliminar tu feedback previo para esta decisión.",
    feedback_note_placeholder: "Nota opcional: ¿por qué?",
    feedback_saved: "Guardado",
  },
  breakdown: {
    not_evaluated: "no evaluado",
    structural: "estructural:",
    semantic: "semántico:",
    blend: "mezcla:",
    semantic_short: "sem",
    structural_short: "estruct",
    axes_section: "Desglose del puntaje",
    signals_section: "Qué se detectó",
    pts: "pts",
    total_tooltip:
      "Puntaje final tras mezclar las capas estructural y (opcional) semántica, acotado a 0-100.",
  },
  how_to_read: {
    title: "Cómo leer este tablero",
    show: "Cómo leer este tablero",
    hide: "Ocultar",
    intro:
      "Adaptive Guard se sienta entre Claude Code y tu pantalla. Cada vez que Claude termina una respuesta, el guard califica dos cosas en escala 0–100:",
    point_complexity:
      "Qué tan complejo se veía tu prompt — basado en longitud, tokens técnicos, densidad de código y cuántas partes tenía.",
    point_depth:
      "Qué tan completa fue la respuesta — basado en estructura markdown, bloques de código y fraseo variado.",
    outcome_intro: "Después escoge uno de tres resultados:",
    outcome_retry:
      "Prompt complejo, respuesta superficial → el guard bloqueó el cierre y le pidió a Claude repensar.",
    outcome_pass:
      "La respuesta tuvo suficiente profundidad → se dejó pasar. Sin interrupción.",
    outcome_trivial:
      "El prompt era demasiado simple para molestarse en evaluar la respuesta → pasó sin evaluación.",
    privacy:
      "Nada sale de tu máquina. El guard sólo guarda contadores y puntajes en el archivo de telemetría — nunca el texto de tus prompts ni de las respuestas de Claude.",
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
