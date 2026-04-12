<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let isMaximized = $state(false);

  $effect(() => {
    const win = getCurrentWindow();
    win.isMaximized().then((v) => (isMaximized = v));
    const unlisten = win.onResized(async () => {
      isMaximized = await win.isMaximized();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  function minimize() {
    getCurrentWindow().minimize();
  }

  function toggleMaximize() {
    getCurrentWindow().toggleMaximize();
  }

  function close() {
    getCurrentWindow().close();
  }
</script>

<div class="window-controls" data-tauri-drag-region-exclude>
  <button class="wc-btn" onclick={minimize} title="Minimize" aria-label="Minimize">
    <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
      <rect width="10" height="1"/>
    </svg>
  </button>
  <button class="wc-btn" onclick={toggleMaximize} title={isMaximized ? "Restore" : "Maximize"} aria-label={isMaximized ? "Restore" : "Maximize"}>
    {#if isMaximized}
      <!-- Restore icon: two overlapping squares -->
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
        <rect x="0" y="2" width="8" height="8"/>
        <path d="M2 2V0h8v8H8"/>
      </svg>
    {:else}
      <!-- Maximize icon: empty square -->
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
        <rect x="0.5" y="0.5" width="9" height="9"/>
      </svg>
    {/if}
  </button>
  <button class="wc-btn close-btn" onclick={close} title="Close" aria-label="Close">
    <svg width="10" height="10" viewBox="0 0 10 10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round">
      <line x1="0" y1="0" x2="10" y2="10"/>
      <line x1="10" y1="0" x2="0" y2="10"/>
    </svg>
  </button>
</div>

<style>
  .window-controls {
    position: fixed;
    top: 0;
    right: 0;
    height: 40px;
    display: flex;
    align-items: center;
    z-index: 9999;
    /* No background — immersive, blends with whatever pane is behind */
  }

  .wc-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 40px;
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: background 0.1s ease, color 0.1s ease;
  }

  .wc-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .close-btn:hover {
    background: #C42B1C;
    color: white;
  }
</style>
