<script lang="ts">
  import { getState, closeAddFeedDialog, loadFeedTree } from "$lib/stores/app.svelte";
  import { addFeed } from "$lib/utils/tauri";

  const appState = getState();

  let url = $state("");
  let selectedFolderId = $state<number | null>(null);
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  // Flatten all folders for the dropdown
  let folderOptions = $derived.by(() => {
    const result: { id: number; name: string; depth: number }[] = [];
    function flatten(folders: typeof appState.feedTree.folders, depth = 0) {
      for (const f of folders) {
        result.push({ id: f.folder.id, name: f.folder.name, depth });
        flatten(f.children, depth + 1);
      }
    }
    flatten(appState.feedTree.folders);
    return result;
  });

  async function handleAdd() {
    const trimmed = url.trim();
    if (!trimmed) {
      error = "Please enter a feed URL.";
      return;
    }

    // Basic URL validation / auto-fix
    let feedUrl = trimmed;
    if (!/^https?:\/\//i.test(feedUrl)) {
      feedUrl = "https://" + feedUrl;
    }

    isLoading = true;
    error = null;

    try {
      await addFeed(feedUrl, selectedFolderId);
      await loadFeedTree();
      closeAddFeedDialog();
      url = "";
      selectedFolderId = null;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : e?.message ?? "Failed to add feed.";
      error = msg.includes("not a valid") || msg.includes("parse")
        ? "Could not find a valid RSS/Atom feed at that URL."
        : msg.includes("network") || msg.includes("connect")
        ? "Could not connect to the server. Check the URL and your internet connection."
        : msg;
    } finally {
      isLoading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") closeAddFeedDialog();
    if (e.key === "Enter" && !isLoading) handleAdd();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) closeAddFeedDialog();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="dialog-backdrop"
  role="dialog"
  aria-modal="true"
  aria-label="Add Feed"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
>
  <div class="dialog">
    <div class="dialog-header">
      <h2 class="dialog-title">Add RSS Feed</h2>
      <button class="dialog-close" onclick={closeAddFeedDialog} aria-label="Close">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <div class="dialog-body">
      <div class="form-group">
        <label class="form-label" for="feed-url">Feed URL</label>
        <div class="url-input-wrap" class:has-error={!!error}>
          <svg class="url-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 11a9 9 0 0 1 9 9"/>
            <path d="M4 4a16 16 0 0 1 16 16"/>
            <circle cx="5" cy="19" r="1"/>
          </svg>
          <input
            id="feed-url"
            type="url"
            class="url-input"
            bind:value={url}
            placeholder="https://example.com/feed.xml"
            disabled={isLoading}
            autofocus
            autocomplete="off"
            spellcheck="false"
          />
          {#if isLoading}
            <div class="input-spinner"></div>
          {/if}
        </div>
        {#if error}
          <p class="error-msg">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
            {error}
          </p>
        {/if}
      </div>

      <div class="form-group">
        <label class="form-label" for="feed-folder">Folder (optional)</label>
        <select
          id="feed-folder"
          class="form-select"
          bind:value={selectedFolderId}
          disabled={isLoading}
        >
          <option value={null}>No folder</option>
          {#each folderOptions as folder}
            <option value={folder.id}>
              {"  ".repeat(folder.depth)}{folder.name}
            </option>
          {/each}
        </select>
      </div>

      <div class="form-hint">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
        You can paste a website URL — Kublai will auto-detect the RSS feed.
      </div>
    </div>

    <div class="dialog-footer">
      <button class="btn-secondary" onclick={closeAddFeedDialog} disabled={isLoading}>
        Cancel
      </button>
      <button class="btn-primary" onclick={handleAdd} disabled={isLoading || !url.trim()}>
        {#if isLoading}
          <div class="btn-spinner"></div>
          Adding...
        {:else}
          Add Feed
        {/if}
      </button>
    </div>
  </div>
</div>

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fadein 0.15s ease;
  }

  @keyframes fadein {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .dialog {
    width: 420px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg), 0 24px 64px rgba(0,0,0,0.2);
    animation: slidein 0.2s ease;
    overflow: hidden;
  }

  @keyframes slidein {
    from { transform: translateY(-12px) scale(0.97); opacity: 0; }
    to { transform: none; opacity: 1; }
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 12px;
    border-bottom: 1px solid var(--color-divider);
  }

  .dialog-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .dialog-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: background 0.1s;
  }

  .dialog-close:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .dialog-body {
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .form-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .url-input-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 10px;
    background: var(--color-bg-base);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: border-color 0.15s;
  }

  .url-input-wrap:focus-within {
    border-color: var(--color-accent);
  }

  .url-input-wrap.has-error {
    border-color: var(--color-danger);
  }

  .url-icon {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .url-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 13px;
    color: var(--color-text-primary);
    padding: 0;
    min-width: 0;
  }

  .url-input::placeholder {
    color: var(--color-text-tertiary);
  }

  .url-input:disabled {
    opacity: 0.6;
  }

  .input-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .error-msg {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--color-danger);
    margin: 0;
  }

  .form-select {
    height: 36px;
    padding: 0 10px;
    background: var(--color-bg-base);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-primary);
    font-size: 13px;
    outline: none;
    cursor: pointer;
    appearance: auto;
  }

  .form-select:focus {
    border-color: var(--color-accent);
  }

  .form-hint {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--color-text-tertiary);
    line-height: 1.5;
  }

  .dialog-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px 16px;
    border-top: 1px solid var(--color-divider);
  }

  .btn-secondary {
    height: 32px;
    padding: 0 14px;
    background: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.12s;
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .btn-primary {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 16px;
    background: var(--color-accent);
    border: none;
    border-radius: var(--radius-md);
    color: white;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.12s;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--color-accent-hover);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .btn-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255,255,255,0.4);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
</style>
