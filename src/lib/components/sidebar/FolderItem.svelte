<script lang="ts">
  import type { FeedTreeFolder } from "$lib/types";
  import FeedItem from "./FeedItem.svelte";
  import FolderItem from "./FolderItem.svelte";
  import ContextMenu from "../common/ContextMenu.svelte";
  import {
    getState,
    selectFolder,
    markFolderAllRead,
    loadFeedTree,
    openAddFeedDialog,
  } from "$lib/stores/app.svelte";
  import { deleteFolder, renameFolder } from "$lib/utils/tauri";

  interface Props {
    folder: FeedTreeFolder;
    onaddFeedInFolder?: (folderId: number) => void;
  }

  let { folder, onaddFeedInFolder }: Props = $props();

  const appState = getState();

  let expanded = $state(true);
  let showMenu = $state(false);
  let menuX = $state(0);
  let menuY = $state(0);
  let isRenaming = $state(false);
  let renameValue = $state(folder.folder.name);

  let isSelected = $derived(appState.selectedFolder?.folder.id === folder.folder.id);

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    menuX = e.clientX;
    menuY = e.clientY;
    showMenu = true;
  }

  async function handleDelete() {
    try {
      await deleteFolder(folder.folder.id);
      await loadFeedTree();
    } catch (e) {
      console.error("Failed to delete folder:", e);
    }
  }

  async function handleRename() {
    if (!renameValue.trim() || renameValue === folder.folder.name) {
      isRenaming = false;
      return;
    }
    try {
      await renameFolder(folder.folder.id, renameValue.trim());
      await loadFeedTree();
    } catch (e) {
      console.error("Failed to rename folder:", e);
    } finally {
      isRenaming = false;
    }
  }

  function startRename() {
    renameValue = folder.folder.name;
    isRenaming = true;
  }

  const menuItems = $derived([
    {
      label: "Mark All as Read",
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>`,
      action: () => markFolderAllRead(folder.folder.id),
    },
    {
      label: "New Feed in Folder",
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>`,
      action: () => onaddFeedInFolder?.(folder.folder.id) ?? openAddFeedDialog(),
    },
    { separator: true, label: "", action: () => {} },
    {
      label: "Rename Folder",
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>`,
      action: startRename,
    },
    {
      label: "Delete Folder",
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/></svg>`,
      action: handleDelete,
      danger: true,
    },
  ]);
</script>

<div class="folder-item-wrap">
  <!-- Folder Header -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="folder-header"
    class:selected={isSelected}
    onclick={() => {
      if (!isRenaming) {
        selectFolder(folder);
      }
    }}
    onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); if (!isRenaming) selectFolder(folder); } }}
    oncontextmenu={handleContextMenu}
    role="button"
    tabindex="0"
  >
    <!-- Expand/collapse toggle -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="folder-chevron"
      class:expanded
      onclick={(e) => { e.stopPropagation(); expanded = !expanded; }}
      onkeydown={(e) => { if (e.key === "Enter") { e.stopPropagation(); expanded = !expanded; } }}
      aria-label={expanded ? "Collapse folder" : "Expand folder"}
      role="button"
      tabindex="-1"
    >
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="9 18 15 12 9 6"/>
      </svg>
    </div>

    <!-- Folder Icon -->
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="folder-icon">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
    </svg>

    {#if isRenaming}
      <input
        class="rename-input"
        bind:value={renameValue}
        onblur={handleRename}
        onkeydown={(e) => {
          if (e.key === "Enter") handleRename();
          if (e.key === "Escape") { isRenaming = false; renameValue = folder.folder.name; }
          e.stopPropagation();
        }}
        onclick={(e) => e.stopPropagation()}
        autofocus
      />
    {:else}
      <span class="folder-name">{folder.folder.name}</span>
    {/if}

    {#if folder.unread_count > 0}
      <span class="unread-badge" class:selected={isSelected}>
        {folder.unread_count > 999 ? "999+" : folder.unread_count}
      </span>
    {/if}
  </div>

  <!-- Feed Children -->
  {#if expanded}
    <div class="folder-feeds">
      {#each folder.feeds as feed}
        <FeedItem {feed} />
      {/each}
      {#if folder.children.length > 0}
        {#each folder.children as child}
          <div class="nested-folder">
            <FolderItem folder={child} {onaddFeedInFolder} />
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

{#if showMenu}
  <ContextMenu
    items={menuItems}
    x={menuX}
    y={menuY}
    onclose={() => showMenu = false}
  />
{/if}

<style>
  .folder-item-wrap {
    display: flex;
    flex-direction: column;
  }

  .folder-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 10px 6px 6px;
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

  .folder-header:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .folder-header.selected {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .folder-chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: inherit;
    flex-shrink: 0;
    border-radius: 3px;
    transition: transform 0.15s ease;
    transform: rotate(0deg);
  }

  .folder-chevron.expanded {
    transform: rotate(90deg);
  }

  .folder-chevron:hover {
    background: var(--color-bg-active);
  }

  .folder-icon {
    flex-shrink: 0;
    opacity: 0.7;
    color: inherit;
  }

  .folder-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
    min-width: 0;
  }

  .rename-input {
    flex: 1;
    background: var(--color-bg-base);
    border: 1px solid var(--color-accent);
    border-radius: 3px;
    color: var(--color-text-primary);
    font-size: 13px;
    padding: 1px 5px;
    outline: none;
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

  .folder-feeds {
    padding-left: 16px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .nested-folder {
    margin-top: 1px;
  }
</style>
