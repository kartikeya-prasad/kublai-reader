<script lang="ts">
  import { onMount } from "svelte";

  interface MenuItem {
    label: string;
    icon?: string;
    action: () => void;
    danger?: boolean;
    disabled?: boolean;
    separator?: boolean;
  }

  interface Props {
    items: MenuItem[];
    x: number;
    y: number;
    onclose: () => void;
  }

  let { items, x, y, onclose }: Props = $props();

  let menuEl = $state<HTMLDivElement | null>(null);
  let menuWidth = $state(0);
  let menuHeight = $state(0);

  let adjustedX = $derived(menuWidth > 0 && x + menuWidth > window.innerWidth ? window.innerWidth - menuWidth - 4 : x);
  let adjustedY = $derived(menuHeight > 0 && y + menuHeight > window.innerHeight ? window.innerHeight - menuHeight - 4 : y);

  $effect(() => {
    if (menuEl) {
      const rect = menuEl.getBoundingClientRect();
      menuWidth = rect.width;
      menuHeight = rect.height;
    }
  });

  function handleAction(item: MenuItem) {
    if (item.disabled) return;
    item.action();
    onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }

  onMount(() => {
    // Close on click outside
    function handleClickOutside(e: MouseEvent) {
      if (menuEl && !menuEl.contains(e.target as Node)) {
        onclose();
      }
    }
    // Use capture to intercept all clicks
    document.addEventListener("mousedown", handleClickOutside, true);
    document.addEventListener("keydown", handleKeydown);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside, true);
      document.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<div
  class="context-menu"
  bind:this={menuEl}
  style="left: {adjustedX}px; top: {adjustedY}px;"
  role="menu"
>
  {#each items as item}
    {#if item.separator}
      <div class="menu-separator"></div>
    {:else}
      <button
        class="menu-item"
        class:danger={item.danger}
        class:disabled={item.disabled}
        onclick={() => handleAction(item)}
        role="menuitem"
        disabled={item.disabled}
      >
        {#if item.icon}
          <span class="menu-icon">{@html item.icon}</span>
        {/if}
        <span class="menu-label">{item.label}</span>
      </button>
    {/if}
  {/each}
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 9999;
    min-width: 180px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg), 0 8px 32px rgba(0,0,0,0.12);
    padding: 4px;
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 10px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s ease;
  }

  .menu-item:hover:not(.disabled) {
    background: var(--color-bg-hover);
  }

  .menu-item.danger {
    color: var(--color-danger);
  }

  .menu-item.danger:hover {
    background: rgba(239, 68, 68, 0.1);
  }

  .menu-item.disabled {
    opacity: 0.4;
    cursor: default;
  }

  .menu-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    opacity: 0.7;
    color: inherit;
  }

  .menu-label {
    flex: 1;
  }

  .menu-separator {
    height: 1px;
    background: var(--color-divider);
    margin: 4px 0;
  }
</style>
