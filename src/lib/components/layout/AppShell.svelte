<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "./Sidebar.svelte";
  import ArticleList from "../articles/ArticleList.svelte";
  import ArticleReader from "../reader/ArticleReader.svelte";
  import AddFeedDialog from "../dialogs/AddFeedDialog.svelte";
  import SettingsDialog from "../dialogs/SettingsDialog.svelte";
  import {
    getState,
    loadFeedTree,
    loadArticles,
    loadTags,
    clearSearch,
    navigateArticle,
    toggleArticleStar,
    markArticleRead,
    markArticleUnread,
    toggleArticleReadLater,
    refreshAll,
    markFeedAllRead,
    markFolderAllRead,
    openSettingsDialog,
    closeSettingsDialog,
    closeAddFeedDialog,
    updateSelectedArticleParsedContent,
  } from "$lib/stores/app.svelte";
  import { parseArticle } from "$lib/utils/tauri";

  const appState = getState();

  let sidebarWidth = $state(250);
  let articleListWidth = $state(360);
  let resizingSidebar = $state(false);
  let resizingArticleList = $state(false);

  onMount(async () => {
    await loadFeedTree();
    await loadArticles(0, false);
    await loadTags();
  });

  // Keyboard shortcuts
  function handleKeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement).tagName;
    const inInput = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";

    // Escape always works
    if (e.key === "Escape") {
      if (inInput) return;
      if (appState.isSearchActive) clearSearch();
      if (appState.showSettingsDialog) closeSettingsDialog();
      if (appState.showAddFeedDialog) closeAddFeedDialog();
      return;
    }

    if (inInput) return;

    switch (e.key) {
      case "ArrowUp":
        e.preventDefault();
        navigateArticle(-1);
        break;
      case "ArrowDown":
        e.preventDefault();
        navigateArticle(1);
        break;
      case "r":
      case "R":
        if (appState.selectedArticle) {
          if (appState.selectedArticle.is_read) {
            markArticleUnread(appState.selectedArticle.id);
          } else {
            markArticleRead(appState.selectedArticle.id);
          }
        }
        break;
      case "s":
      case "S":
        if (appState.selectedArticle) toggleArticleStar(appState.selectedArticle.id);
        break;
      case "b":
      case "B":
        if (appState.selectedArticle) toggleArticleReadLater(appState.selectedArticle.id);
        break;
      case "F5":
        e.preventDefault();
        refreshAll();
        break;
      case "Delete":
        if (appState.selectedArticle) {
          markArticleRead(appState.selectedArticle.id);
          navigateArticle(1);
        }
        break;
    }

    // W: toggle extraction
    if ((e.key === "w" || e.key === "W") && appState.selectedArticle) {
      handleToggleExtraction();
    }

    // Ctrl+Shift+A: mark all read
    if (e.ctrlKey && e.shiftKey && e.key === "A") {
      if (appState.selectedFeed) markFeedAllRead(appState.selectedFeed.id);
      else if (appState.selectedFolder) markFolderAllRead(appState.selectedFolder.folder.id);
    }

    // Ctrl+F: focus search
    if (e.ctrlKey && e.key === "f") {
      e.preventDefault();
      const searchInput = document.querySelector<HTMLInputElement>('.article-search-input');
      searchInput?.focus();
    }

    // Ctrl+,: open settings
    if (e.ctrlKey && e.key === ",") {
      e.preventDefault();
      openSettingsDialog();
    }
  }

  async function handleToggleExtraction() {
    if (!appState.selectedArticle) return;
    try {
      const parsed = await parseArticle(appState.selectedArticle.id);
      updateSelectedArticleParsedContent(parsed);
    } catch {}
  }

  function startResizeSidebar(e: MouseEvent) {
    e.preventDefault();
    resizingSidebar = true;
    const onMove = (ev: MouseEvent) => {
      sidebarWidth = Math.max(180, Math.min(400, ev.clientX));
    };
    const onUp = () => {
      resizingSidebar = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  function startResizeArticleList(e: MouseEvent) {
    e.preventDefault();
    resizingArticleList = true;
    const startX = e.clientX;
    const startWidth = articleListWidth;
    const onMove = (ev: MouseEvent) => {
      articleListWidth = Math.max(280, Math.min(600, startWidth + ev.clientX - startX));
    };
    const onUp = () => {
      resizingArticleList = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-shell">
  <div class="main-content">
    <!-- Left Pane: Feed Sidebar -->
    <div class="pane sidebar-pane" style="width: {sidebarWidth}px">
      <Sidebar />
    </div>

    <!-- Resize Handle: Sidebar -->
    <div
      class="resize-handle"
      class:active={resizingSidebar}
      onmousedown={startResizeSidebar}
      role="separator"
      aria-orientation="vertical"
      tabindex="-1"
    ></div>

    <!-- Middle Pane: Article List -->
    <div class="pane article-list-pane" style="width: {articleListWidth}px">
      <ArticleList />
    </div>

    <!-- Resize Handle: Article List -->
    <div
      class="resize-handle"
      class:active={resizingArticleList}
      onmousedown={startResizeArticleList}
      role="separator"
      aria-orientation="vertical"
      tabindex="-1"
    ></div>

    <!-- Right Pane: Article Reader -->
    <div class="pane reader-pane">
      <ArticleReader />
    </div>
  </div>

  <!-- Dialogs -->
  {#if appState.showAddFeedDialog}
    <AddFeedDialog />
  {/if}

  {#if appState.showSettingsDialog}
    <SettingsDialog />
  {/if}
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-base);
    overflow: hidden;
  }

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .pane {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .sidebar-pane {
    background: var(--color-bg-sidebar);
    border-right: 1px solid var(--color-divider);
    flex-shrink: 0;
  }

  .article-list-pane {
    background: var(--color-bg-base);
    border-right: 1px solid var(--color-divider);
    flex-shrink: 0;
  }

  .reader-pane {
    flex: 1;
    background: var(--color-bg-elevated);
    min-width: 0;
  }

  .resize-handle {
    width: 4px;
    cursor: col-resize;
    background: transparent;
    flex-shrink: 0;
    position: relative;
    z-index: 10;
    transition: background 0.15s ease;
  }

  .resize-handle:hover,
  .resize-handle.active {
    background: var(--color-accent);
    opacity: 0.5;
  }
</style>
