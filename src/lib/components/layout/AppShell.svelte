<script lang="ts">
  import { onMount } from "svelte";
  import Titlebar from "./Titlebar.svelte";
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
    doSearch,
    clearSearch,
  } from "$lib/stores/app.svelte";

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
    // Don't handle if in input/textarea
    const tag = (e.target as HTMLElement).tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

    switch (e.key) {
      case "Escape":
        if (appState.isSearchActive) clearSearch();
        break;
    }
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

  async function handleSearch(query: string) {
    if (query.trim()) {
      await doSearch(query);
    } else {
      await clearSearch();
      await loadArticles(0, false);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-shell">
  <Titlebar onsearch={handleSearch} />

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
    background: var(--color-bg-surface);
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
