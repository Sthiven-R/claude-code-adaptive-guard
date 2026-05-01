<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { HookStatus, InstallResult } from "../types";
  import { lang, t, type Lang } from "../i18n";
  import { themeMode, type ThemeMode } from "../stores/theme";
  import Icon from "./Icon.svelte";

  let { hook, version, onClose, onChanged }: {
    hook: HookStatus;
    version: string;
    onClose: () => void;
    onChanged: () => void;
  } = $props();

  function setLang(l: Lang) {
    lang.set(l);
  }

  function setTheme(m: ThemeMode) {
    themeMode.set(m);
  }

  let busy = $state(false);
  let result = $state<InstallResult | null>(null);
  let confirmingUninstall = $state(false);

  async function doInstall() {
    busy = true;
    result = null;
    try {
      result = await invoke<InstallResult>("hook_install");
      if (result.ok) onChanged();
    } catch (e) {
      result = {
        ok: false,
        message: e instanceof Error ? e.message : String(e),
        backup_path: null,
      };
    } finally {
      busy = false;
    }
  }

  async function doUninstall() {
    busy = true;
    result = null;
    confirmingUninstall = false;
    try {
      result = await invoke<InstallResult>("hook_uninstall");
      if (result.ok) onChanged();
    } catch (e) {
      result = {
        ok: false,
        message: e instanceof Error ? e.message : String(e),
        backup_path: null,
      };
    } finally {
      busy = false;
    }
  }

  function handleBackdropKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<!--
  Backdrop is the click-target for dismiss. The a11y plugin warns about
  event listeners on a non-interactive element, but `role="dialog"` +
  `tabindex` + Escape handler is the standard accessible modal pattern.
  Keyboard users dismiss via Escape; mouse users click outside.
-->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="backdrop"
  role="dialog"
  aria-modal="true"
  aria-labelledby="settings-title"
  tabindex="-1"
  onclick={onClose}
  onkeydown={handleBackdropKey}
>
  <div
    class="modal"
    role="document"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <header>
      <h2 id="settings-title">{$t.settings.title}</h2>
      <button class="close" onclick={onClose} aria-label={$t.settings.close}>
        <Icon name="close" size={18} />
      </button>
    </header>

    <section>
      <div class="row">
        <span class="label">{$t.settings.hook_status}</span>
        <span class="value">
          <span class="dot" class:on={hook.installed} class:off={!hook.installed}></span>
          {hook.installed ? $t.settings.installed : $t.settings.not_installed}
        </span>
      </div>

      {#if hook.installed}
        {#if confirmingUninstall}
          <div class="confirm">
            <p>
              {$t.settings.confirm_uninstall_lead}
              <code>adaptive-guard</code>
              {$t.settings.confirm_uninstall_entry}
              <code>{hook.settings_path}</code>{$t.settings.confirm_uninstall_tail}
            </p>
            <div class="confirm-actions">
              <button class="danger" onclick={doUninstall} disabled={busy}>
                {busy ? $t.settings.removing : $t.settings.confirm_yes}
              </button>
              <button class="ghost" onclick={() => (confirmingUninstall = false)}>
                {$t.settings.cancel}
              </button>
            </div>
          </div>
        {:else}
          <button
            class="ghost danger-text"
            onclick={() => (confirmingUninstall = true)}
            disabled={busy}
          >
            {$t.settings.uninstall}
          </button>
        {/if}
      {:else}
        <button class="primary" onclick={doInstall} disabled={busy || !!hook.error}>
          {busy ? $t.settings.installing : $t.settings.install_hook}
        </button>
      {/if}

      {#if hook.error}
        <div class="error">
          <strong>{$t.settings.cannot_install}</strong>
          <code>{hook.error}</code>
        </div>
      {/if}

      {#if result}
        <div class="result" class:ok={result.ok} class:err={!result.ok}>
          <strong>{result.ok ? $t.settings.ok : $t.settings.failed}</strong>
          <span>{result.message}</span>
          {#if result.backup_path}
            <div class="backup">{$t.settings.backup} <code>{result.backup_path}</code></div>
          {/if}
        </div>
      {/if}
    </section>

    <section>
      <h3>{$t.settings.language_section}</h3>
      <div class="seg-toggle" role="radiogroup" aria-label={$t.settings.language_section}>
        <button
          class="seg-btn"
          class:active={$lang === "en"}
          role="radio"
          aria-checked={$lang === "en"}
          onclick={() => setLang("en")}
        >
          {$t.settings.language_english}
        </button>
        <button
          class="seg-btn"
          class:active={$lang === "es"}
          role="radio"
          aria-checked={$lang === "es"}
          onclick={() => setLang("es")}
        >
          {$t.settings.language_spanish}
        </button>
      </div>
    </section>

    <section>
      <h3>{$t.settings.theme_section}</h3>
      <div class="seg-toggle" role="radiogroup" aria-label={$t.settings.theme_section}>
        <button
          class="seg-btn"
          class:active={$themeMode === "dark"}
          role="radio"
          aria-checked={$themeMode === "dark"}
          onclick={() => setTheme("dark")}
        >
          {$t.settings.theme_dark}
        </button>
        <button
          class="seg-btn"
          class:active={$themeMode === "light"}
          role="radio"
          aria-checked={$themeMode === "light"}
          onclick={() => setTheme("light")}
        >
          {$t.settings.theme_light}
        </button>
        <button
          class="seg-btn"
          class:active={$themeMode === "auto"}
          role="radio"
          aria-checked={$themeMode === "auto"}
          onclick={() => setTheme("auto")}
        >
          {$t.settings.theme_auto}
        </button>
      </div>
    </section>

    <section>
      <h3>{$t.settings.paths_section}</h3>
      <div class="kv">
        <div class="k">{$t.settings.repo}</div>
        <div class="v"><code>{hook.repo_root ?? $t.settings.not_configured}</code></div>
      </div>
      <div class="kv">
        <div class="k">{$t.settings.cli_config}</div>
        <div class="v"><code>{hook.config_path}</code></div>
      </div>
      <div class="kv">
        <div class="k">{$t.settings.settings_path}</div>
        <div class="v"><code>{hook.settings_path}</code></div>
      </div>
    </section>

    <footer>
      <span>adaptive-guard{version ? ` v${version}` : ""}</span>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 20px;
  }
  .modal {
    background: var(--bg-soft);
    border: 1px solid var(--border);
    border-radius: 12px;
    width: 100%;
    max-width: 560px;
    max-height: 88vh;
    overflow-y: auto;
    padding: 18px 22px 16px;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 14px;
  }
  h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--ink);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .close {
    background: none;
    border: none;
    color: var(--ink-faint);
    font-size: 22px;
    line-height: 1;
    cursor: pointer;
    padding: 0 6px;
  }
  .close:hover {
    color: var(--ink);
  }

  section {
    padding: 8px 0 12px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 12px;
  }
  section:last-of-type {
    border-bottom: none;
  }
  h3 {
    margin: 0 0 10px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--ink-faint);
    font-weight: 600;
  }

  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    font-size: 12px;
  }
  .row .label {
    color: var(--ink-dim);
  }
  .row .value {
    font-family: var(--mono);
    color: var(--ink);
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ink-faint);
  }
  .dot.on {
    background: var(--ok);
    box-shadow: 0 0 6px var(--ok);
  }
  .dot.off {
    background: var(--ink-faint);
  }

  button {
    font-family: inherit;
    font-size: 12px;
    padding: 6px 14px;
    border-radius: 6px;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button.primary {
    background: var(--accent);
    border: none;
    color: var(--bg);
    font-weight: 600;
  }
  button.primary:hover:not(:disabled) {
    opacity: 0.85;
  }
  button.ghost {
    background: var(--bg-hard);
    border: 1px solid var(--border);
    color: var(--ink);
  }
  button.ghost:hover:not(:disabled) {
    border-color: var(--ink-dim);
  }
  button.ghost.danger-text {
    color: var(--danger);
  }
  button.ghost.danger-text:hover:not(:disabled) {
    border-color: var(--danger);
  }
  button.danger {
    background: var(--danger);
    border: none;
    color: var(--bg);
    font-weight: 600;
  }
  button.danger:hover:not(:disabled) {
    opacity: 0.85;
  }

  .confirm {
    background: rgba(248, 113, 113, 0.06);
    border: 1px solid var(--danger);
    border-radius: 6px;
    padding: 10px 12px;
    font-size: 12px;
  }
  .confirm p {
    margin: 0 0 10px;
    color: var(--ink);
    line-height: 1.5;
  }
  .confirm code {
    background: var(--bg-hard);
    padding: 1px 5px;
    border-radius: 3px;
    color: var(--accent);
    font-size: 11px;
    word-break: break-all;
  }
  .confirm-actions {
    display: flex;
    gap: 8px;
  }

  .kv {
    display: grid;
    grid-template-columns: 80px 1fr;
    gap: 12px;
    align-items: baseline;
    font-size: 11px;
    margin-bottom: 6px;
  }
  .kv .k {
    color: var(--ink-faint);
    font-family: var(--mono);
  }
  .kv .v code {
    color: var(--ink);
    background: none;
    padding: 0;
    font-family: var(--mono);
    word-break: break-all;
  }

  .error {
    margin-top: 10px;
    padding: 10px 12px;
    border-radius: 6px;
    background: rgba(248, 113, 113, 0.06);
    border: 1px solid var(--danger);
    font-size: 12px;
    line-height: 1.5;
  }
  .error strong {
    color: var(--danger);
    display: block;
    margin-bottom: 4px;
  }
  .error code {
    background: var(--bg-hard);
    padding: 1px 5px;
    border-radius: 3px;
    color: var(--ink-dim);
    font-size: 11px;
    word-break: break-all;
  }

  .result {
    margin-top: 10px;
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 12px;
    line-height: 1.5;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .result.ok {
    background: rgba(134, 239, 172, 0.06);
    border: 1px solid var(--ok);
    color: var(--ink);
  }
  .result.err {
    background: rgba(248, 113, 113, 0.06);
    border: 1px solid var(--danger);
    color: var(--ink);
  }
  .result strong {
    color: var(--ink);
  }
  .result .backup {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--ink-faint);
    word-break: break-all;
  }
  .result code {
    background: none;
    padding: 0;
    color: var(--ink-faint);
  }

  .seg-toggle {
    display: inline-flex;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    font-size: var(--text-small);
  }
  .seg-btn {
    background: var(--color-bg-base);
    border: none;
    color: var(--color-ink-dim);
    padding: 6px 14px;
    border-radius: 0;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-standard),
                color var(--duration-fast) var(--ease-standard);
    font-family: var(--font-mono);
  }
  .seg-btn:not(:last-child) {
    border-right: 1px solid var(--color-border);
  }
  .seg-btn:hover:not(.active):not(:disabled) {
    color: var(--color-ink);
    background: var(--color-bg-elevated);
  }
  .seg-btn.active {
    background: var(--color-accent);
    color: var(--color-ink-on-accent);
    font-weight: var(--weight-semibold);
  }

  footer {
    text-align: center;
    color: var(--ink-faint);
    font-family: var(--mono);
    font-size: 10px;
    padding-top: 8px;
  }
</style>
