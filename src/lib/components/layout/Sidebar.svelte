<script lang="ts">
  import { onMount } from "svelte";
  import FeedItem from "../sidebar/FeedItem.svelte";
  import FolderItem from "../sidebar/FolderItem.svelte";
  import {
    getState,
    loadFeedTree,
    selectFilter,
    openAddFeedDialog,
    openSettingsDialog,
  } from "$lib/stores/app.svelte";

  const appState = getState();

  onMount(() => {
    loadFeedTree();
  });

  // Smart filter unread counts
  let totalUnread = $derived(appState.feedTree.total_unread);

  // Count starred: can't easily without querying, so show count only for unread
  // We'll keep starred/read_later badges as 0 for now unless we track them
</script>

<div class="sidebar">
  <div class="sidebar-header">
    <span class="sidebar-title">Feeds</span>
    <div class="sidebar-actions">
      <button
        class="sidebar-btn"
        onclick={openSettingsDialog}
        aria-label="Settings"
        title="Settings"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </button>
      <button
        class="sidebar-btn add-btn"
        onclick={openAddFeedDialog}
        aria-label="Add feed"
        title="Add Feed"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Smart Filters -->
  <div class="filter-section">
    <button
      class="filter-item"
      class:active={appState.selectedFilter === "all" && !appState.selectedFeed && !appState.selectedFolder}
      onclick={() => selectFilter("all")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
        <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
      </svg>
      <span>All Articles</span>
      {#if totalUnread > 0}
        <span class="badge">{totalUnread > 999 ? "999+" : totalUnread}</span>
      {/if}
    </button>

    <button
      class="filter-item"
      class:active={appState.selectedFilter === "unread" && !appState.selectedFeed && !appState.selectedFolder}
      onclick={() => selectFilter("unread")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <span>Unread</span>
      {#if totalUnread > 0}
        <span class="badge">{totalUnread > 999 ? "999+" : totalUnread}</span>
      {/if}
    </button>

    <button
      class="filter-item"
      class:active={appState.selectedFilter === "starred" && !appState.selectedFeed && !appState.selectedFolder}
      onclick={() => selectFilter("starred")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
      </svg>
      <span>Starred</span>
    </button>

    <button
      class="filter-item"
      class:active={appState.selectedFilter === "read_later" && !appState.selectedFeed && !appState.selectedFolder}
      onclick={() => selectFilter("read_later")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>
      </svg>
      <span>Read Later</span>
    </button>
  </div>

  <div class="sidebar-divider"></div>

  <!-- Feed List -->
  <div class="feed-list">
    {#if appState.isLoadingFeeds}
      <div class="loading-feeds">
        {#each Array(3) as _, i}
          <div class="feed-skeleton" style="animation-delay: {i * 80}ms"></div>
        {/each}
      </div>
    {:else if appState.feedTree.folders.length === 0 && appState.feedTree.uncategorized.length === 0}
      <div class="sidebar-empty">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="opacity:0.3; margin-bottom: 8px;">
          <path d="M4 11a9 9 0 0 1 9 9"/>
          <path d="M4 4a16 16 0 0 1 16 16"/>
          <circle cx="5" cy="19" r="1"/>
        </svg>
        <p>No feeds yet.</p>
        <button class="empty-add-btn" onclick={openAddFeedDialog}>
          + Add your first feed
        </button>
      </div>
    {:else}
      <div class="feed-tree">
        <!-- Folders -->
        {#each appState.feedTree.folders as folder}
          <FolderItem {folder} />
        {/each}

        <!-- Uncategorized feeds -->
        {#if appState.feedTree.folders.length > 0 && appState.feedTree.uncategorized.length > 0}
          <div class="uncategorized-label">Uncategorized</div>
        {/if}
        {#each appState.feedTree.uncategorized as feed}
          <FeedItem {feed} />
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 10px 10px 14px;
    flex-shrink: 0;
  }

  .sidebar-title {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-text-tertiary);
  }

  .sidebar-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .sidebar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: background 0.12s ease, color 0.12s ease;
  }

  .sidebar-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .add-btn:hover {
    color: var(--color-accent);
  }

  .filter-section {
    display: flex;
    flex-direction: column;
    padding: 0 8px;
    gap: 1px;
  }

  .filter-item {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 7px 10px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background 0.12s ease, color 0.12s ease;
  }

  .filter-item:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .filter-item.active {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .badge {
    margin-left: auto;
    font-size: 10px;
    font-weight: 600;
    min-width: 18px;
    text-align: center;
    padding: 1px 4px;
    border-radius: 10px;
    background: var(--color-bg-active);
    color: var(--color-text-tertiary);
  }

  .filter-item.active .badge {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .sidebar-divider {
    height: 1px;
    background: var(--color-divider);
    margin: 8px 14px;
    flex-shrink: 0;
  }

  .feed-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px 8px;
  }

  .feed-tree {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .uncategorized-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-text-tertiary);
    padding: 8px 10px 4px;
  }

  .sidebar-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 24px 16px;
    color: var(--color-text-tertiary);
    font-size: 12px;
    text-align: center;
    gap: 4px;
  }

  .sidebar-empty p {
    margin: 0;
  }

  .empty-add-btn {
    margin-top: 8px;
    padding: 6px 12px;
    background: var(--color-accent-soft);
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-accent);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.12s ease;
  }

  .empty-add-btn:hover {
    background: var(--color-accent);
    color: white;
  }

  /* Loading skeleton */
  .loading-feeds {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px 2px;
  }

  .feed-skeleton {
    height: 32px;
    border-radius: var(--radius-sm);
    background: var(--color-bg-hover);
    animation: pulse 1.5s ease infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
</style>
