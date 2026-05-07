<script lang="ts">
  import { explainerExpanded } from "../stores/explainer";
  import { t } from "../i18n";
  import Icon from "./Icon.svelte";

  /*
   * "How to read this dashboard" panel.
   *
   * Renders at the top of the window. First-time visitors land on the
   * expanded state and see a four-paragraph explainer of what the app
   * is, what the three outcome tags (RETRY / PASS / TRIVIAL) mean, and
   * the privacy stance. Once they hit "Hide" the chevron collapses
   * the body but the trigger stays visible so re-opening is one click.
   *
   * Strings live in i18n.ts under the `how_to_read` namespace so EN
   * and ES are both covered and any future locale falls into the same
   * compile-time check that catches missing keys.
   */

  function toggle(): void {
    explainerExpanded.update((v) => !v);
  }
</script>

<section class="howto" class:expanded={$explainerExpanded}>
  <button
    class="trigger"
    onclick={toggle}
    aria-expanded={$explainerExpanded}
    aria-controls="howto-body"
  >
    <span class="chev" class:expanded={$explainerExpanded} aria-hidden="true">
      <Icon name="chevron" size={12} />
    </span>
    <span class="trigger-text">
      {$explainerExpanded ? $t.how_to_read.title : $t.how_to_read.show}
    </span>
    {#if $explainerExpanded}
      <span class="hide-hint">{$t.how_to_read.hide}</span>
    {/if}
  </button>

  {#if $explainerExpanded}
    <div id="howto-body" class="body">
      <p>{$t.how_to_read.intro}</p>

      <ul class="bullets">
        <li>{$t.how_to_read.point_complexity}</li>
        <li>{$t.how_to_read.point_depth}</li>
      </ul>

      <p>{$t.how_to_read.outcome_intro}</p>

      <ul class="outcomes">
        <li>
          <span class="tag retry">{$t.decision.block}</span>
          <span>{$t.how_to_read.outcome_retry}</span>
        </li>
        <li>
          <span class="tag pass">{$t.decision.deep}</span>
          <span>{$t.how_to_read.outcome_pass}</span>
        </li>
        <li>
          <span class="tag trivial">{$t.decision.skip}</span>
          <span>{$t.how_to_read.outcome_trivial}</span>
        </li>
      </ul>

      <p class="privacy">{$t.how_to_read.privacy}</p>
    </div>
  {/if}
</section>

<style>
  .howto {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    margin-bottom: var(--space-4);
    overflow: hidden;
    transition: border-color var(--duration-base) var(--ease-standard);
  }

  .howto.expanded {
    border-color: var(--color-accent-dim);
  }

  .trigger {
    width: 100%;
    background: transparent;
    border: none;
    color: var(--color-ink);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 12px 18px;
    text-align: left;
    cursor: pointer;
    font-size: var(--text-mono-body);
    font-family: var(--font-mono);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    transition: background var(--duration-fast) var(--ease-standard);
  }

  .trigger:hover {
    background: var(--color-bg-hover);
  }

  .chev {
    color: var(--color-ink-faint);
    display: inline-flex;
    align-items: center;
    transition: transform var(--duration-base) var(--ease-out-quart),
                color var(--duration-fast) var(--ease-standard);
  }

  .chev.expanded {
    transform: rotate(90deg);
    color: var(--color-accent);
  }

  .trigger-text {
    flex: 1;
  }

  .hide-hint {
    font-size: var(--text-mono-micro);
    color: var(--color-ink-faint);
    font-weight: var(--weight-regular);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
  }

  .body {
    padding: 4px 22px 18px;
    border-top: 1px solid var(--color-border);
    font-size: var(--text-mono-body);
    color: var(--color-ink-dim);
    line-height: var(--leading-relaxed);
    animation: slide-down var(--duration-base) var(--ease-out-quart);
  }

  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .body p {
    margin: 12px 0 8px;
  }

  .body p:first-child {
    margin-top: 14px;
  }

  .body .bullets,
  .body .outcomes {
    list-style: none;
    padding: 0;
    margin: 4px 0 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .body .bullets li {
    padding-left: 18px;
    position: relative;
  }

  .body .bullets li::before {
    content: "·";
    position: absolute;
    left: 6px;
    color: var(--color-accent);
    font-weight: var(--weight-bold);
  }

  .body .outcomes li {
    display: grid;
    grid-template-columns: 88px 1fr;
    gap: 12px;
    align-items: start;
    color: var(--color-ink);
  }

  .tag {
    display: inline-block;
    padding: 2px 8px;
    border-radius: var(--radius-xs);
    border: 1px solid;
    font-family: var(--font-mono);
    font-size: var(--text-mono-micro);
    letter-spacing: var(--tracking-widest);
    font-weight: var(--weight-semibold);
    text-align: center;
  }

  .tag.retry {
    background: var(--color-danger-soft);
    color: var(--color-danger);
    border-color: var(--color-danger);
  }

  .tag.pass {
    background: var(--color-success-soft);
    color: var(--color-success);
    border-color: var(--color-success);
  }

  .tag.trivial {
    background: var(--color-ink-soft);
    color: var(--color-ink-dim);
    border-color: var(--color-border);
  }

  .privacy {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px dashed var(--color-border);
    font-size: var(--text-small);
    color: var(--color-ink-faint);
  }
</style>
