<script lang="ts">
  import { t } from "../i18n";

  let { active }: { active: boolean } = $props();
</script>

<span
  class="live"
  class:active
  title={active ? $t.live.on_tooltip : $t.live.off_tooltip}
>
  <span class="dot"></span>
  <span class="text">{active ? $t.live.on : $t.live.off}</span>
</span>

<style>
  .live {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border-radius: 10px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.12em;
    color: var(--ink-faint);
    background: var(--bg-hard);
    border: 1px solid var(--border);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ink-faint);
  }

  .live.active {
    color: var(--ok);
    border-color: rgba(134, 239, 172, 0.3);
  }
  .live.active .dot {
    background: var(--ok);
    box-shadow: 0 0 6px var(--ok);
    animation: pulse 1.6s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.5;
      transform: scale(0.85);
    }
  }
</style>
