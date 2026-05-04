<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { HookStatus, InstallResult } from "../types";
  import { t } from "../i18n";
  import Logo from "./Logo.svelte";

  let { status, onInstalled, onSkip }: {
    status: HookStatus;
    onInstalled: () => void;
    onSkip: () => void;
  } = $props();

  // Platform-aware hint for the recovery instruction. We use navigator.platform
  // because the welcome screen renders before any backend round-trip — a quick
  // heuristic is enough; the user only sees this when the CLI config is
  // missing, which is itself rare.
  const setupGlobalHint = (() => {
    const isWin = typeof navigator !== "undefined" &&
      /win/i.test(navigator.platform || navigator.userAgent || "");
    return isWin ? "scripts\\setup-global.bat" : "./scripts/setup-global.sh";
  })();

  let installing = $state(false);
  let result = $state<InstallResult | null>(null);

  async function install() {
    installing = true;
    result = null;
    try {
      const r = await invoke<InstallResult>("hook_install");
      result = r;
      if (r.ok) {
        // Tell the parent to refresh hook status and tear down the
        // welcome screen.
        onInstalled();
      }
    } catch (e) {
      result = {
        ok: false,
        message: e instanceof Error ? e.message : String(e),
        backup_path: null,
      };
    } finally {
      installing = false;
    }
  }
</script>

<section class="welcome">
  <header>
    <Logo variant="full" size={28} />
    <span class="tag">{$t.welcome.badge}</span>
  </header>

  <p class="lede">
    {$t.welcome.lede}
  </p>

  <ol class="steps">
    <li class="step active">
      <div class="num">1</div>
      <div class="body">
        <h2>{$t.welcome.step1_title}</h2>
        <p>
          {$t.welcome.step1_body_lead}
          <code>~/.claude/settings.json</code>{$t.welcome.step1_body_tail}
        </p>

        {#if status.error}
          <div class="error">
            <strong>{$t.welcome.cannot_install}</strong>
            <code>{status.error}</code>
            <div class="hint">
              {$t.welcome.cannot_install_hint_lead}
              <code>{setupGlobalHint}</code>
              {$t.welcome.cannot_install_hint_tail}
            </div>
          </div>
        {:else}
          <button class="primary" onclick={install} disabled={installing}>
            {installing ? $t.welcome.installing : $t.welcome.install_button}
          </button>
          {#if result}
            <div class="result" class:ok={result.ok} class:err={!result.ok}>
              <strong>{result.ok ? $t.welcome.installed : $t.welcome.install_failed}</strong>
              <span>{result.message}</span>
              {#if result.backup_path}
                <div class="backup">
                  {$t.welcome.backup} <code>{result.backup_path}</code>
                </div>
              {/if}
            </div>
          {/if}
        {/if}
      </div>
    </li>

    <li class="step">
      <div class="num">2</div>
      <div class="body">
        <h2>{$t.welcome.step2_title}</h2>
        <p>{$t.welcome.step2_body}</p>
      </div>
    </li>

    <li class="step">
      <div class="num">3</div>
      <div class="body">
        <h2>{$t.welcome.step3_title}</h2>
        <p>{$t.welcome.step3_body}</p>
      </div>
    </li>
  </ol>

  <footer>
    <span>{$t.welcome.skip_question}</span>
    <button class="link" onclick={onSkip}>{$t.welcome.skip}</button>
  </footer>
</section>

<style>
  .welcome {
    max-width: 720px;
    margin: 40px auto;
    padding: 28px 32px 24px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--elevation-1);
  }

  header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 18px;
  }
  .tag {
    margin-left: auto;
    font-family: var(--font-mono);
    font-size: var(--text-mono-micro);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
    color: var(--color-accent);
    border: 1px solid var(--color-accent-dim);
    padding: 2px 8px;
    border-radius: var(--radius-xs);
  }

  .lede {
    color: var(--color-ink-dim);
    font-size: var(--text-mono-body);
    line-height: var(--leading-relaxed);
    margin: 0 0 24px;
  }

  ol.steps {
    list-style: none;
    padding: 0;
    margin: 0 0 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .step {
    display: flex;
    gap: 16px;
    padding: 14px 16px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-base);
    transition: border-color var(--duration-base) var(--ease-standard),
                background var(--duration-base) var(--ease-standard);
  }
  .step.active {
    border-color: var(--color-accent);
    background: var(--color-accent-dim);
  }
  .step .num {
    flex: 0 0 28px;
    height: 28px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--color-border);
    color: var(--color-ink-faint);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-size: var(--text-small);
    font-weight: var(--weight-semibold);
  }
  .step.active .num {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }
  .step .body {
    flex: 1;
    min-width: 0;
  }
  .step h2 {
    margin: 0 0 4px;
    font-size: var(--text-mono-body);
    font-weight: var(--weight-semibold);
    color: var(--color-ink);
  }
  .step p {
    margin: 0 0 10px;
    font-size: var(--text-small);
    line-height: var(--leading-normal);
    color: var(--color-ink-dim);
  }
  .step code {
    background: var(--color-bg-elevated);
    padding: 1px 6px;
    border-radius: var(--radius-xs);
    font-size: var(--text-micro);
    color: var(--color-accent);
  }

  button.primary {
    background: var(--color-accent);
    border: none;
    color: var(--color-ink-on-accent);
    font-weight: var(--weight-semibold);
    font-size: var(--text-mono-body);
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity var(--duration-fast) var(--ease-standard);
  }
  button.primary:hover:not(:disabled) {
    opacity: 0.85;
  }
  button.primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .result {
    margin-top: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    font-size: var(--text-small);
    line-height: var(--leading-normal);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .result.ok {
    background: var(--color-success-soft);
    border: 1px solid var(--color-success);
    color: var(--color-ink);
  }
  .result.err {
    background: var(--color-danger-soft);
    border: 1px solid var(--color-danger);
    color: var(--color-ink);
  }
  .result strong {
    color: var(--color-ink);
  }
  .result .backup {
    font-family: var(--font-mono);
    font-size: var(--text-mono-micro);
    color: var(--color-ink-faint);
    word-break: break-all;
  }
  .result code {
    background: none;
    padding: 0;
    color: var(--color-ink-faint);
  }

  .error {
    margin-top: 8px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-danger-soft);
    border: 1px solid var(--color-danger);
    color: var(--color-ink);
    font-size: var(--text-small);
    line-height: var(--leading-normal);
  }
  .error strong {
    color: var(--color-danger);
    display: block;
    margin-bottom: 4px;
  }
  .error code {
    background: var(--color-bg-elevated);
    padding: 1px 6px;
    border-radius: var(--radius-xs);
    font-size: var(--text-micro);
    color: var(--color-ink-dim);
    word-break: break-all;
  }
  .error .hint {
    margin-top: 6px;
    color: var(--color-ink-dim);
    font-size: var(--text-micro);
  }

  footer {
    display: flex;
    justify-content: center;
    gap: 8px;
    padding-top: 18px;
    border-top: 1px solid var(--color-border);
    font-size: var(--text-micro);
    color: var(--color-ink-faint);
    font-family: var(--font-mono);
  }

  button.link {
    background: none;
    border: none;
    color: var(--color-accent);
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: var(--text-micro);
    padding: 0;
    text-decoration: underline;
    transition: opacity var(--duration-fast) var(--ease-standard);
  }
  button.link:hover {
    opacity: 0.8;
  }
</style>
