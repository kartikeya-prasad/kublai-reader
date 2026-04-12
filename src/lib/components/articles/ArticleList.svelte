<script lang="ts">
  import { onMount } from "svelte";
  import type { ArticleListItem } from "$lib/types";
  import ContextMenu from "../common/ContextMenu.svelte";
  import {
    getState,
    selectArticle,
    markArticleRead,
    markArticleUnread,
    toggleArticleStar,
    toggleArticleReadLater,
    loadNextPage,
    setArticleView,
    refreshAll,
    loadArticles,
    doSearch,
    clearSearch,
  } from "$lib/stores/app.svelte";
  import { markAboveRead, parseArticle } from "$lib/utils/tauri";
  import { open as openUrl } from "@tauri-apps/plugin-shell";

  const appState = getState();

  let listEl = $state<HTMLDivElement | null>(null);
  let sentinelEl = $state<HTMLDivElement | null>(null);

  // Context menu state
  let showMenu = $state(false);
  let menuX = $state(0);
  let menuY = $state(0);
  let menuArticle = $state<ArticleListItem | null>(null);

  // IntersectionObserver for auto-mark-read (scroll off top)
  let readObserver: IntersectionObserver | null = null;
  // IntersectionObserver for infinite scroll
  let scrollObserver: IntersectionObserver | null = null;

  // Track which articles have been seen (scrolled off top)
  let seenSet = new Set<number>();

  $effect(() => {
    // Reset seen set when articles change
    const articles = appState.displayedArticles;
    if (!articles.length) {
      seenSet.clear();
    }
  });

  onMount(() => {
    setupScrollObserver();
    setupReadObserver();
    return () => {
      scrollObserver?.disconnect();
      readObserver?.disconnect();
    };
  });

  $effect(() => {
    // Re-setup read observer when articles change
    const articles = appState.displayedArticles;
    if (listEl && articles.length > 0) {
      setTimeout(() => setupReadObserver(), 50);
    }
  });

  function setupScrollObserver() {
    if (!sentinelEl) return;
    scrollObserver?.disconnect();
    scrollObserver = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting && !appState.isLoadingArticles) {
            loadNextPage();
          }
        }
      },
      { root: listEl, threshold: 0.1 }
    );
    scrollObserver.observe(sentinelEl);
  }

  function setupReadObserver() {
    readObserver?.disconnect();
    if (!listEl) return;

    readObserver = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          const articleId = parseInt(
            (entry.target as HTMLElement).dataset.articleId ?? "0"
          );
          if (!articleId) continue;

          // If item was visible and now scrolled OUT from the top
          if (!entry.isIntersecting && entry.boundingClientRect.top < 0) {
            const article = appState.displayedArticles.find((a) => a.id === articleId);
            if (article && !article.is_read && !seenSet.has(articleId)) {
              seenSet.add(articleId);
              markArticleRead(articleId);
            }
          }
        }
      },
      {
        root: listEl,
        threshold: 0,
        rootMargin: "0px 0px 0px 0px",
      }
    );

    // Observe all article elements
    listEl.querySelectorAll("[data-article-id]").forEach((el) => {
      readObserver!.observe(el);
    });
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return "";
    try {
      const date = new Date(dateStr);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffHrs = diffMs / (1000 * 60 * 60);
      const diffDays = diffMs / (1000 * 60 * 60 * 24);

      if (diffHrs < 1) {
        const mins = Math.floor(diffMs / (1000 * 60));
        return mins <= 0 ? "just now" : `${mins}m ago`;
      }
      if (diffHrs < 24) return `${Math.floor(diffHrs)}h ago`;
      if (diffDays < 7) return `${Math.floor(diffDays)}d ago`;
      return date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
    } catch {
      return "";
    }
  }

  function openContextMenu(e: MouseEvent, article: ArticleListItem) {
    e.preventDefault();
    e.stopPropagation();
    menuArticle = article;
    menuX = e.clientX;
    menuY = e.clientY;
    showMenu = true;
  }

  async function openInBrowser(url: string | null) {
    if (!url) return;
    try {
      await openUrl(url);
    } catch {
      window.open(url, "_blank");
    }
  }

  let searchValue = $state('');
  let searchInputEl = $state<HTMLInputElement | null>(null);

  async function handleSearchInput() {
    if (searchValue.trim()) {
      await doSearch(searchValue);
    } else {
      await clearSearch();
      await loadArticles(0, false);
    }
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      searchValue = '';
      clearSearch();
      loadArticles(0, false);
      searchInputEl?.blur();
    }
  }

  let menuItems = $derived.by(() => {
    if (!menuArticle) return [];
    const a = menuArticle;
    return [
      {
        label: a.is_read ? "Mark as Unread" : "Mark as Read",
        icon: a.is_read
          ? `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/></svg>`
          : `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>`,
        action: () =>
          a.is_read ? markArticleUnread(a.id) : markArticleRead(a.id),
      },
      {
        label: a.is_starred ? "Unstar" : "Star",
        icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="${a.is_starred ? 'currentColor' : 'none'}" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>`,
        action: () => toggleArticleStar(a.id),
      },
      {
        label: a.is_read_later ? "Remove from Read Later" : "Save for Later",
        icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="${a.is_read_later ? 'currentColor' : 'none'}" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/></svg>`,
        action: () => toggleArticleReadLater(a.id),
      },
      { separator: true, label: "", action: () => {} },
      {
        label: "Mark Above as Read",
        icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 11 12 6 7 11"/><polyline points="17 18 12 13 7 18"/></svg>`,
        action: async () => {
          try {
            await markAboveRead(a.feed_id, a.id);
            await loadArticles(0, false);
          } catch (e) {
            console.error("markAboveRead failed:", e);
          }
        },
      },
      {
        label: "Open in Browser",
        icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>`,
        action: () => openInBrowser(a.url),
        disabled: !a.url,
      },
      {
        label: 'Copy Link',
        icon: `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>`,
        action: () => { if (a.url) navigator.clipboard.writeText(a.url); },
        disabled: !a.url,
      },
      {
        label: a.has_parsed_content ? 'Remove Extracted Text' : 'Extract Full Text',
        icon: `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>`,
        action: async () => {
          if (!a.has_parsed_content) {
            try { await parseArticle(a.id); } catch {}
          }
        },
      },
      { separator: true, label: '', action: () => {} },
    ];
  });
</script>

<div class="article-list-wrap">
  <!-- Header -->
  <div class="list-header">
    <div class="header-left">
      <h2 class="pane-title">{appState.currentPaneTitle}</h2>
      {#if !appState.isSearchActive}
        <span class="article-count">
          {appState.articlePage.total > 0 ? appState.articlePage.total : ""}
        </span>
      {/if}
    </div>
    <div class="header-actions">
      <button
        class="icon-btn"
        class:active={appState.articleView !== "card"}
        onclick={() => setArticleView(appState.articleView === "card" ? "compact" : appState.articleView === "compact" ? "magazine" : "card")}
        title={appState.articleView === "card" ? "Switch to compact view" : appState.articleView === "compact" ? "Switch to magazine view" : "Switch to card view"}
        aria-label="Toggle view"
      >
        {#if appState.articleView === "card"}
          <!-- card icon: list -->
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="8" y1="6" x2="21" y2="6"/>
            <line x1="8" y1="12" x2="21" y2="12"/>
            <line x1="8" y1="18" x2="21" y2="18"/>
            <line x1="3" y1="6" x2="3.01" y2="6"/>
            <line x1="3" y1="12" x2="3.01" y2="12"/>
            <line x1="3" y1="18" x2="3.01" y2="18"/>
          </svg>
        {:else if appState.articleView === "compact"}
          <!-- compact icon: tighter list -->
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="3" y1="5" x2="21" y2="5"/>
            <line x1="3" y1="10" x2="21" y2="10"/>
            <line x1="3" y1="15" x2="21" y2="15"/>
            <line x1="3" y1="20" x2="21" y2="20"/>
          </svg>
        {:else}
          <!-- magazine icon: grid -->
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7"/>
            <rect x="14" y="3" width="7" height="7"/>
            <rect x="3" y="14" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/>
          </svg>
        {/if}
      </button>
      <button
        class="icon-btn"
        class:spinning={appState.isRefreshing}
        onclick={refreshAll}
        title="Refresh all feeds"
        aria-label="Refresh"
        disabled={appState.isRefreshing}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10"/>
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Search Bar -->
  <div class="search-bar">
    <svg class="search-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
    </svg>
    <input
      bind:this={searchInputEl}
      bind:value={searchValue}
      class="article-search-input"
      type="text"
      placeholder="Search articles…"
      oninput={handleSearchInput}
      onkeydown={handleSearchKeydown}
    />
    {#if searchValue}
      <button class="search-clear" onclick={() => { searchValue = ''; clearSearch(); loadArticles(0,false); }}>×</button>
    {/if}
  </div>

  <!-- Article List -->
  <div
    class="article-list"
    bind:this={listEl}
    class:compact={appState.articleView === "compact"}
  >
    {#if appState.isLoadingArticles && appState.displayedArticles.length === 0}
      <!-- Skeleton loading -->
      <div class="skeleton-list">
        {#each Array(6) as _, i}
          <div class="skeleton-card" class:compact={appState.articleView === "compact"} style="animation-delay: {i * 60}ms">
            {#if appState.articleView === "card"}
              <div class="sk-thumb"></div>
              <div class="sk-body">
                <div class="sk-line wide"></div>
                <div class="sk-line medium"></div>
                <div class="sk-line narrow"></div>
              </div>
            {:else}
              <div class="sk-line wide"></div>
              <div class="sk-line narrow"></div>
            {/if}
          </div>
        {/each}
      </div>
    {:else if appState.displayedArticles.length === 0}
      <!-- Empty state -->
      <div class="empty-state">
        {#if appState.isSearchActive}
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
          <p>No results for "{appState.searchQuery}"</p>
        {:else}
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
            <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
          </svg>
          <p>No articles here yet</p>
          <span>Refresh feeds to load new articles</span>
        {/if}
      </div>
    {:else}
      <!-- Article items: card and compact views -->
      {#if appState.articleView !== "magazine"}
        {#each appState.displayedArticles as article (article.id)}
          {#if appState.articleView === "card"}
            <!-- Card View -->
            <button
              class="article-card"
              class:unread={!article.is_read}
              class:selected={appState.selectedArticleListItem?.id === article.id}
              data-article-id={article.id}
              onclick={() => selectArticle(article)}
              oncontextmenu={(e) => openContextMenu(e, article)}
            >
              {#if !article.is_read}
                <div class="unread-indicator"></div>
              {/if}

              <div class="card-body">
                <div class="card-meta">
                  <span class="feed-name">{article.feed_title}</span>
                  <span class="article-date">{formatDate(article.published_at)}</span>
                </div>
                <h3 class="article-title">{article.title}</h3>
                {#if article.summary}
                  <p class="article-excerpt">{article.summary}</p>
                {/if}
                <div class="card-footer">
                  {#if article.author}
                    <span class="author">{article.author}</span>
                  {/if}
                  <div class="card-indicators">
                    {#if article.has_parsed_content}
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="reader-icon" aria-label="Full text extracted">
                        <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
                        <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
                      </svg>
                    {/if}
                    {#if article.is_starred}
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="1.5" class="star-icon">
                        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
                      </svg>
                    {/if}
                    {#if article.is_read_later}
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="1.5" class="later-icon">
                        <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>
                      </svg>
                    {/if}
                  </div>
                </div>
              </div>

              {#if article.thumbnail_url}
                <div class="card-thumbnail">
                  <img
                    src={article.thumbnail_url}
                    alt=""
                    loading="lazy"
                    onerror={(e) => { (e.currentTarget as HTMLImageElement).parentElement!.style.display = 'none'; }}
                  />
                </div>
              {/if}
            </button>
          {:else}
            <!-- Compact View -->
            <button
              class="article-compact"
              class:unread={!article.is_read}
              class:selected={appState.selectedArticleListItem?.id === article.id}
              data-article-id={article.id}
              onclick={() => selectArticle(article)}
              oncontextmenu={(e) => openContextMenu(e, article)}
            >
              {#if !article.is_read}
                <div class="unread-dot"></div>
              {:else}
                <div class="read-dot"></div>
              {/if}
              <div class="compact-body">
                <span class="compact-title">{article.title}</span>
                <div class="compact-meta">
                  <span class="feed-name">{article.feed_title}</span>
                  <span class="bullet">·</span>
                  <span class="article-date">{formatDate(article.published_at)}</span>
                  {#if article.has_parsed_content}
                    <span class="bullet">·</span>
                    <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="reader-icon" aria-label="Full text extracted">
                      <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
                      <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
                    </svg>
                  {/if}
                  {#if article.is_starred}
                    <span class="bullet">·</span>
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor" class="star-icon">
                      <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
                    </svg>
                  {/if}
                </div>
              </div>
            </button>
          {/if}
        {/each}

        <!-- Load more sentinel -->
        <div bind:this={sentinelEl} class="scroll-sentinel"></div>
      {:else}
        <!-- Magazine View -->
        <div class="magazine-grid">
          {#each appState.displayedArticles as article (article.id)}
            <button
              class="magazine-card"
              class:selected={appState.selectedArticleListItem?.id === article.id}
              data-article-id={article.id}
              onclick={() => selectArticle(article)}
              oncontextmenu={(e) => openContextMenu(e, article)}
            >
              {#if article.thumbnail_url}
                <div class="mag-thumbnail">
                  <img src={article.thumbnail_url} alt="" loading="lazy" />
                </div>
              {:else}
                <div class="mag-thumbnail mag-no-thumb">
                  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="m3 9 4-4 4 4 4-6 6 6"/><circle cx="8.5" cy="8.5" r="1.5"/></svg>
                </div>
              {/if}
              <div class="mag-body">
                <div class="mag-feed">{article.feed_title}</div>
                <div class="mag-title" class:unread={!article.is_read}>{article.title}</div>
                <div class="mag-meta">{formatDate(article.published_at)}</div>
              </div>
            </button>
          {/each}
        </div>

        <!-- Load more sentinel for magazine -->
        <div bind:this={sentinelEl} class="scroll-sentinel"></div>
      {/if}

      {#if appState.isLoadingArticles}
        <div class="load-more-indicator">
          <div class="spinner"></div>
        </div>
      {/if}
    {/if}
  </div>
</div>

{#if showMenu && menuArticle}
  <ContextMenu
    items={menuItems}
    x={menuX}
    y={menuY}
    onclose={() => { showMenu = false; menuArticle = null; }}
  />
{/if}

<style>
  .article-list-wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border-bottom: 1px solid var(--color-divider);
    flex-shrink: 0;
    gap: 8px;
  }

  .header-left {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
  }

  .pane-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .article-count {
    font-size: 11px;
    color: var(--color-text-tertiary);
    font-weight: 400;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .icon-btn {
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

  .icon-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .icon-btn.active {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .icon-btn.spinning svg {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--color-divider);
    background: var(--color-bg-base);
    flex-shrink: 0;
  }

  .search-icon {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .article-search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 12px;
    color: var(--color-text-primary);
    outline: none;
    font-family: inherit;
  }

  .article-search-input::placeholder {
    color: var(--color-text-tertiary);
  }

  .search-clear {
    background: none;
    border: none;
    color: var(--color-text-tertiary);
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    line-height: 1;
  }

  .search-clear:hover { color: var(--color-text-primary); }

  .article-list {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }

  /* ===== Card View ===== */
  .article-card {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 12px 14px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--color-divider);
    cursor: pointer;
    text-align: left;
    transition: background 0.1s ease;
    position: relative;
  }

  .article-card:hover {
    background: var(--color-bg-hover);
  }

  .article-card.selected {
    background: var(--color-accent-soft);
  }

  .unread-indicator {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--color-unread-dot);
    border-radius: 0 2px 2px 0;
  }

  .card-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .feed-name {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-accent);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 120px;
  }

  .article-date {
    font-size: 11px;
    color: var(--color-text-tertiary);
    margin-left: auto;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .article-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    line-height: 1.4;
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .article-card.unread .article-title {
    font-weight: 600;
  }

  .article-excerpt {
    font-size: 12px;
    color: var(--color-text-secondary);
    line-height: 1.5;
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 2px;
  }

  .author {
    font-size: 11px;
    color: var(--color-text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-indicators {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .star-icon {
    color: var(--color-star);
  }

  .later-icon {
    color: var(--color-accent);
  }

  .reader-icon {
    color: var(--color-accent);
    opacity: 0.7;
  }

  .card-thumbnail {
    width: 64px;
    height: 64px;
    flex-shrink: 0;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--color-bg-hover);
  }

  .card-thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* ===== Compact View ===== */
  .article-compact {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 14px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--color-divider);
    cursor: pointer;
    text-align: left;
    transition: background 0.1s ease;
  }

  .article-compact:hover {
    background: var(--color-bg-hover);
  }

  .article-compact.selected {
    background: var(--color-accent-soft);
  }

  .unread-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--color-unread-dot);
    flex-shrink: 0;
  }

  .read-dot {
    width: 7px;
    height: 7px;
    flex-shrink: 0;
  }

  .compact-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .compact-title {
    font-size: 13px;
    font-weight: 400;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .article-compact.unread .compact-title {
    font-weight: 600;
  }

  .compact-meta {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--color-text-tertiary);
  }

  .bullet {
    opacity: 0.5;
  }

  /* ===== States ===== */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 200px;
    color: var(--color-text-tertiary);
    gap: 8px;
    padding: 40px 20px;
    opacity: 0.7;
  }

  .empty-state svg {
    opacity: 0.4;
  }

  .empty-state p {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .empty-state span {
    font-size: 12px;
  }

  /* Skeleton */
  .skeleton-list {
    display: flex;
    flex-direction: column;
  }

  .skeleton-card {
    display: flex;
    gap: 10px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--color-divider);
    animation: pulse 1.5s ease infinite;
  }

  .skeleton-card.compact {
    padding: 8px 14px;
    align-items: center;
  }

  .sk-thumb {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-sm);
    background: var(--color-bg-hover);
    flex-shrink: 0;
  }

  .sk-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    justify-content: center;
  }

  .sk-line {
    height: 12px;
    border-radius: 6px;
    background: var(--color-bg-hover);
  }

  .sk-line.wide { width: 90%; }
  .sk-line.medium { width: 70%; }
  .sk-line.narrow { width: 50%; }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .scroll-sentinel {
    height: 1px;
    width: 100%;
  }

  .load-more-indicator {
    display: flex;
    justify-content: center;
    padding: 16px;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  /* ===== Magazine View ===== */
  .magazine-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 10px;
    padding: 10px;
    overflow-y: auto;
  }

  .magazine-card {
    display: flex;
    flex-direction: column;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    cursor: pointer;
    text-align: left;
    transition: box-shadow 0.15s ease, transform 0.1s ease;
    width: 100%;
  }

  .magazine-card:hover {
    box-shadow: var(--shadow-md);
    transform: translateY(-1px);
  }

  .magazine-card.selected {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px var(--color-accent-soft);
  }

  .mag-thumbnail {
    width: 100%;
    aspect-ratio: 16/9;
    overflow: hidden;
    background: var(--color-bg-hover);
  }

  .mag-thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .mag-no-thumb {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .mag-body {
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .mag-feed {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-accent);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mag-title {
    font-size: 12px;
    font-weight: 400;
    color: var(--color-text-secondary);
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.4;
  }

  .mag-title.unread {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .mag-meta {
    font-size: 10px;
    color: var(--color-text-tertiary);
  }
</style>
