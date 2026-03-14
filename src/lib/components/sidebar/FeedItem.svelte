<script lang="ts">
  import type { FeedWithCount } from "$lib/types";
  import ContextMenu from "../common/ContextMenu.svelte";
  import {
    getState,
    selectFeed,
    refreshOneFeed,
    markFeedAllRead,
    loadFeedTree,
  } from "$lib/stores/app.svelte";
  import { deleteFeed } from "$lib/utils/tauri";

  interface Props {
    feed: FeedWithCount;
    ondelete?: (feedId: number) => void;
    onedit?: (feed: FeedWithCount) => void;
  }

  let { feed, ondelete, onedit }: Props = $props();

  const appState = getState();

  let showMenu = $state(false);
  let menuX = $state(0);
  let menuY = $state(0);

  let isSelected = $derived(appState.selectedFeed?.id === feed.id);

  function getFaviconLetter(title: string): string {
    return title.trim().charAt(0).toUpperCase() || "F";
  }

  function getFaviconColor(title: string): string {
    const colors = [
      "#6366f1", "#8b5cf6", "#ec4899", "#f59e0b",
      "#10b981", "#0ea5e9", "#ef4444", "#14b8a6",
    ];
    let hash = 0;
    for (let i = 0; i < title.length; i++) {
      hash = title.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length];
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    menuX = e.clientX;
    menuY = e.clientY;
    showMenu = true;
  }

  async function handleDelete() {
    try {
      await deleteFeed(feed.id);
      await loadFeedTree();
      ondelete?.(feed.id);
    } catch (e) {
      console.error("Failed to delete feed:", e);
    }
  }

  const menuItems = $derived([
    {
      label: "Refresh",
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>`,
      action: () => refreshOneFeed(feed.id),
    },
    {
      label: "Mark All as Read",
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>`,
      action: () => markFeedAllRead(feed.id),
    },
    { separator: true, label: "", action: () => {} },
    {
      label: "Edit Feed",
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>`,
      action: () => onedit?.(feed),
    },
    {
      label: "Delete Feed",
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/></svg>`,
      action: handleDelete,
      danger: true,
    },
  ]);

  const faviconBg = $derived(getFaviconColor(feed.title));
  const faviconLetter = $derived(getFaviconLetter(feed.title));
</script>

<button
  class="feed-item"
  class:selected={isSelected}
  onclick={() => selectFeed(feed)}
  oncontextmenu={handleContextMenu}
  title={feed.title}
>
  <!-- Feed Icon -->
  <div class="feed-icon">
    {#if feed.icon_url}
      <img
        src={feed.icon_url}
        alt=""
        width="16"
        height="16"
        onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
      />
    {:else}
      <span class="feed-letter" style="background: {faviconBg}">
        {faviconLetter}
      </span>
    {/if}
  </div>

  <span class="feed-title">{feed.title}</span>

  {#if feed.unread_count > 0}
    <span class="unread-badge" class:selected={isSelected}>
      {feed.unread_count > 999 ? "999+" : feed.unread_count}
    </span>
  {/if}
</button>

{#if showMenu}
  <ContextMenu
    items={menuItems}
    x={menuX}
    y={menuY}
    onclose={() => showMenu = false}
  />
{/if}

<style>
  .feed-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.12s ease, color 0.12s ease;
    min-width: 0;
  }

  .feed-item:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .feed-item.selected {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .feed-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    border-radius: 4px;
    overflow: hidden;
  }

  .feed-icon img {
    width: 16px;
    height: 16px;
    object-fit: contain;
    border-radius: 3px;
  }

  .feed-letter {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 700;
    color: white;
    line-height: 1;
  }

  .feed-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .unread-badge {
    flex-shrink: 0;
    font-size: 10px;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 10px;
    background: var(--color-bg-active);
    color: var(--color-text-tertiary);
    min-width: 18px;
    text-align: center;
    line-height: 16px;
  }

  .unread-badge.selected {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }
</style>
