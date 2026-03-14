<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import ThemeToggle from "../common/ThemeToggle.svelte";

  interface Props {
    onsearch?: (query: string) => void;
  }

  let { onsearch }: Props = $props();

  let searchValue = $state("");

  async function minimize() {
    await getCurrentWindow().minimize();
  }

  async function toggleMaximize() {
    await getCurrentWindow().toggleMaximize();
  }

  async function close() {
    await getCurrentWindow().close();
  }

  function handleSearchInput(e: Event) {
    searchValue = (e.target as HTMLInputElement).value;
    onsearch?.(searchValue);
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      searchValue = "";
      onsearch?.("");
      (e.target as HTMLInputElement).blur();
    }
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="titlebar-left" data-tauri-drag-region>
    <div class="app-logo">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H19a1 1 0 0 1 1 1v18a1 1 0 0 1-1 1H6.5a1 1 0 0 1 0-5H20"/>
      </svg>
    </div>
    <span class="app-title" data-tauri-drag-region>Kublai Reader</span>
  </div>

  <div class="titlebar-center" data-tauri-drag-region>
    <div class="search-container">
      <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search articles..."
        aria-label="Search articles"
        bind:value={searchValue}
        oninput={handleSearchInput}
        onkeydown={handleSearchKeydown}
      />
      <kbd class="search-kbd">Ctrl+F</kbd>
    </div>
  </div>

  <div class="titlebar-right">
    <ThemeToggle />

    <div class="window-controls">
      <button class="win-btn" onclick={minimize} aria-label="Minimize">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect y="5" width="12" height="1" fill="currentColor"/>
        </svg>
      </button>
      <button class="win-btn" onclick={toggleMaximize} aria-label="Maximize">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect x="1" y="1" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1"/>
        </svg>
      </button>
      <button class="win-btn close-btn" onclick={close} aria-label="Close">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.2"/>
          <line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
    </div>
  </div>
</div>

<style>
  .titlebar {
    height: var(--titlebar-height);
    display: flex;
    align-items: center;
    padding: 0 8px;
    gap: 8px;
    background: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-divider);
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 140px;
  }

  .app-logo {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--color-accent);
  }

  .app-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.3px;
  }

  .titlebar-center {
    flex: 1;
    display: flex;
    justify-content: center;
    max-width: 400px;
    margin: 0 auto;
  }

  .search-container {
    display: flex;
    align-items: center;
    width: 100%;
    height: 28px;
    padding: 0 10px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    gap: 6px;
  }

  .search-container:focus-within {
    border-color: var(--color-accent);
    background: var(--color-bg-elevated);
  }

  .search-icon {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 12px;
    color: var(--color-text-primary);
    padding: 0;
  }

  .search-input::placeholder {
    color: var(--color-text-tertiary);
  }

  .search-kbd {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    color: var(--color-text-tertiary);
    font-family: inherit;
  }

  .titlebar-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .window-controls {
    display: flex;
    align-items: center;
    margin-left: 4px;
  }

  .win-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 28px;
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .win-btn:hover {
    background: var(--color-bg-hover);
  }

  .close-btn:hover {
    background: var(--color-danger);
    color: white;
  }
</style>
