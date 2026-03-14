<script lang="ts">
  import { getState, toggleArticleStar, toggleArticleReadLater, updateReaderSettings, updateSelectedArticleParsedContent } from "$lib/stores/app.svelte";
  import { parseArticle, addTagToArticle, removeTagFromArticle, createTag } from "$lib/utils/tauri";
  import { open as openUrl } from "@tauri-apps/plugin-shell";

  const appState = getState();

  let isParsing = $state(false);
  let parseError = $state<string | null>(null);
  let showSettings = $state(false);
  let showTagPanel = $state(false);
  let newTagName = $state("");
  let newTagColor = $state("#6366f1");

  const FONT_OPTIONS = [
    { label: "System UI", value: "Segoe UI Variable, Segoe UI, system-ui, sans-serif" },
    { label: "Georgia", value: "Georgia, serif" },
    { label: "Merriweather", value: "'Merriweather', Georgia, serif" },
    { label: "Source Serif 4", value: "'Source Serif 4', Georgia, serif" },
    { label: "Lora", value: "'Lora', Georgia, serif" },
    { label: "Inter", value: "'Inter', system-ui, sans-serif" },
    { label: "Charter", value: "Charter, Georgia, serif" },
  ];

  const WIDTH_OPTIONS = [
    { label: "Narrow", value: 600 },
    { label: "Normal", value: 720 },
    { label: "Wide", value: 900 },
    { label: "Full", value: 9999 },
  ];

  let article = $derived(appState.selectedArticle);
  let listItem = $derived(appState.selectedArticleListItem);
  let settings = $derived(appState.readerSettings);
  let tags = $derived(appState.tags);

  // The content to display
  let displayContent = $derived(
    article
      ? (article.parsed_content || article.content || article.summary || "")
      : ""
  );

  let contentMaxWidth = $derived(
    settings.content_width >= 9999 ? "100%" : `${settings.content_width}px`
  );

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return "";
    try {
      return new Date(dateStr).toLocaleDateString(undefined, {
        year: "numeric",
        month: "long",
        day: "numeric",
      });
    } catch {
      return dateStr;
    }
  }

  async function handleParseArticle() {
    if (!article) return;
    isParsing = true;
    parseError = null;
    try {
      const content = await parseArticle(article.id);
      updateSelectedArticleParsedContent(content);
    } catch (e) {
      parseError = "Failed to extract article content. The site may block scraping.";
      console.error("Parse error:", e);
    } finally {
      isParsing = false;
    }
  }

  async function openInBrowser() {
    if (!article?.url) return;
    try {
      await openUrl(article.url);
    } catch {
      window.open(article.url, "_blank");
    }
  }

  async function addTag(tagId: number) {
    if (!article) return;
    try {
      await addTagToArticle(article.id, tagId);
      // Reload to reflect changes
    } catch (e) {
      console.error("Failed to add tag:", e);
    }
  }

  async function removeTag(tagId: number) {
    if (!article) return;
    try {
      await removeTagFromArticle(article.id, tagId);
    } catch (e) {
      console.error("Failed to remove tag:", e);
    }
  }

  async function handleCreateTag() {
    if (!newTagName.trim()) return;
    try {
      await createTag(newTagName.trim(), newTagColor);
      newTagName = "";
    } catch (e) {
      console.error("Failed to create tag:", e);
    }
  }

</script>

<div class="reader-pane">
  {#if !article}
    <!-- Empty state -->
    <div class="reader-empty">
      <div class="reader-empty-icon">
        <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
          <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
        </svg>
      </div>
      <p class="reader-empty-title">Kublai Reader</p>
      <p class="reader-empty-sub">Select an article to start reading</p>
    </div>
  {:else}
    <!-- Toolbar -->
    <div class="reader-toolbar">
      <div class="toolbar-left">
        {#if listItem?.feed_title}
          <span class="toolbar-feed">{listItem.feed_title}</span>
        {/if}
      </div>
      <div class="toolbar-right">
        <!-- Star -->
        <button
          class="tool-btn"
          class:active={article.is_starred}
          onclick={() => toggleArticleStar(article!.id)}
          title={article.is_starred ? "Unstar" : "Star article"}
          aria-label={article.is_starred ? "Unstar" : "Star"}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill={article.is_starred ? "currentColor" : "none"} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
          </svg>
        </button>

        <!-- Read Later -->
        <button
          class="tool-btn"
          class:active={article.is_read_later}
          onclick={() => toggleArticleReadLater(article!.id)}
          title={article.is_read_later ? "Remove from Read Later" : "Save for Later"}
          aria-label="Read Later"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill={article.is_read_later ? "currentColor" : "none"} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>
          </svg>
        </button>

        <!-- Tags -->
        <button
          class="tool-btn"
          class:active={showTagPanel}
          onclick={() => showTagPanel = !showTagPanel}
          title="Tags"
          aria-label="Tags"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/>
            <line x1="7" y1="7" x2="7.01" y2="7"/>
          </svg>
        </button>

        <!-- Parse Article -->
        <button
          class="tool-btn"
          class:loading={isParsing}
          onclick={handleParseArticle}
          title="Extract full article content"
          aria-label="Parse article"
          disabled={isParsing}
        >
          {#if isParsing}
            <div class="btn-spinner"></div>
          {:else}
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="16" y1="13" x2="8" y2="13"/>
              <line x1="16" y1="17" x2="8" y2="17"/>
              <polyline points="10 9 9 9 8 9"/>
            </svg>
          {/if}
        </button>

        <!-- Open in Browser -->
        <button
          class="tool-btn"
          onclick={openInBrowser}
          title="Open in browser"
          aria-label="Open in browser"
          disabled={!article.url}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
            <polyline points="15 3 21 3 21 9"/>
            <line x1="10" y1="14" x2="21" y2="3"/>
          </svg>
        </button>

        <div class="toolbar-divider"></div>

        <!-- Reader Settings -->
        <button
          class="tool-btn"
          class:active={showSettings}
          onclick={() => showSettings = !showSettings}
          title="Reader settings"
          aria-label="Settings"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="3" y1="12" x2="21" y2="12"/>
            <line x1="3" y1="6" x2="21" y2="6"/>
            <line x1="3" y1="18" x2="21" y2="18"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Reader Settings Panel -->
    {#if showSettings}
      <div class="settings-panel">
        <div class="settings-row">
          <span class="settings-label">Font</span>
          <div class="settings-options font-options">
            {#each FONT_OPTIONS as opt}
              <button
                class="option-pill"
                class:active={settings.font_family === opt.value}
                onclick={() => updateReaderSettings({ font_family: opt.value })}
                style="font-family: {opt.value}"
              >
                {opt.label}
              </button>
            {/each}
          </div>
        </div>
        <div class="settings-row">
          <span class="settings-label">Size</span>
          <div class="settings-controls">
            <button class="size-btn" onclick={() => updateReaderSettings({ font_size: Math.max(12, settings.font_size - 1) })}>A−</button>
            <span class="size-val">{settings.font_size}px</span>
            <button class="size-btn" onclick={() => updateReaderSettings({ font_size: Math.min(28, settings.font_size + 1) })}>A+</button>
          </div>
          <span class="settings-label">Line</span>
          <div class="settings-controls">
            <button class="size-btn" onclick={() => updateReaderSettings({ line_height: Math.max(1.2, +(settings.line_height - 0.1).toFixed(1)) })}>−</button>
            <span class="size-val">{settings.line_height.toFixed(1)}</span>
            <button class="size-btn" onclick={() => updateReaderSettings({ line_height: Math.min(2.5, +(settings.line_height + 0.1).toFixed(1)) })}>+</button>
          </div>
        </div>
        <div class="settings-row">
          <span class="settings-label">Width</span>
          <div class="settings-options">
            {#each WIDTH_OPTIONS as opt}
              <button
                class="option-pill"
                class:active={settings.content_width === opt.value}
                onclick={() => updateReaderSettings({ content_width: opt.value })}
              >
                {opt.label}
              </button>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Tag Panel -->
    {#if showTagPanel}
      <div class="tag-panel">
        <div class="tag-panel-header">
          <span>Tags</span>
        </div>
        {#if tags.length === 0}
          <p class="tag-empty">No tags yet</p>
        {:else}
          <div class="tag-list">
            {#each tags as tag}
              <button
                class="tag-chip"
                class:active={listItem?.tag_ids?.includes(tag.id)}
                onclick={() => {
                  if (listItem?.tag_ids?.includes(tag.id)) {
                    removeTag(tag.id);
                  } else {
                    addTag(tag.id);
                  }
                }}
                style="--tag-color: {tag.color}"
              >
                <span class="tag-dot"></span>
                {tag.name}
              </button>
            {/each}
          </div>
        {/if}
        <div class="new-tag-row">
          <input
            type="color"
            bind:value={newTagColor}
            class="color-picker"
            aria-label="Tag color"
          />
          <input
            type="text"
            bind:value={newTagName}
            placeholder="New tag..."
            class="tag-input"
            onkeydown={(e) => e.key === "Enter" && handleCreateTag()}
          />
          <button class="tag-add-btn" onclick={handleCreateTag} disabled={!newTagName.trim()}>
            Add
          </button>
        </div>
      </div>
    {/if}

    {#if parseError}
      <div class="parse-error">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
        {parseError}
        <button onclick={() => parseError = null} class="error-close">×</button>
      </div>
    {/if}

    <!-- Article Content -->
    <div class="reader-scroll">
      <div
        class="reader-content"
        style="max-width: {contentMaxWidth}; font-family: {settings.font_family}; font-size: {settings.font_size}px; line-height: {settings.line_height};"
      >
        <!-- Article Header -->
        <div class="article-header">
          {#if listItem?.feed_title}
            <div class="article-source">{listItem.feed_title}</div>
          {/if}
          <h1 class="article-title">{article.title}</h1>
          <div class="article-meta">
            {#if article.author}
              <span class="meta-author">By {article.author}</span>
            {/if}
            {#if article.published_at}
              <span class="meta-sep">·</span>
              <time class="meta-date" datetime={article.published_at}>
                {formatDate(article.published_at)}
              </time>
            {/if}
            {#if article.url}
              <span class="meta-sep">·</span>
              <a class="meta-link" href={article.url} onclick={(e) => { e.preventDefault(); openInBrowser(); }}>
                Read Original
              </a>
            {/if}
          </div>
        </div>

        <!-- Content -->
        {#if displayContent}
          <div class="article-body">{@html displayContent}</div>
        {:else}
          <div class="no-content">
            <p>No content available.</p>
            <button class="parse-prompt-btn" onclick={handleParseArticle} disabled={isParsing}>
              {isParsing ? "Extracting..." : "Try extracting article content"}
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .reader-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* ===== Empty State ===== */
  .reader-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-tertiary);
    gap: 8px;
  }

  .reader-empty-icon {
    opacity: 0.25;
    margin-bottom: 8px;
    color: var(--color-text-secondary);
  }

  .reader-empty-title {
    font-size: 22px;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0;
    opacity: 0.6;
  }

  .reader-empty-sub {
    font-size: 13px;
    margin: 0;
    opacity: 0.5;
  }

  /* ===== Toolbar ===== */
  .reader-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-bottom: 1px solid var(--color-divider);
    flex-shrink: 0;
    gap: 8px;
    min-height: 42px;
  }

  .toolbar-left {
    flex: 1;
    min-width: 0;
  }

  .toolbar-feed {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.3px;
    color: var(--color-accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .tool-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: background 0.12s ease, color 0.12s ease;
    position: relative;
  }

  .tool-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .tool-btn.active {
    color: var(--color-accent);
    background: var(--color-accent-soft);
  }

  .tool-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .tool-btn:disabled:hover {
    background: none;
  }

  .toolbar-divider {
    width: 1px;
    height: 20px;
    background: var(--color-divider);
    margin: 0 4px;
  }

  .btn-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  /* ===== Settings Panel ===== */
  .settings-panel {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-divider);
    background: var(--color-bg-surface);
    flex-shrink: 0;
  }

  .settings-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 0;
    flex-wrap: wrap;
  }

  .settings-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--color-text-tertiary);
    min-width: 36px;
    flex-shrink: 0;
  }

  .settings-options {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .font-options {
    flex-wrap: wrap;
  }

  .option-pill {
    padding: 3px 10px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: 20px;
    font-size: 12px;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .option-pill:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .option-pill.active {
    background: var(--color-accent-soft);
    border-color: var(--color-accent);
    color: var(--color-accent);
    font-weight: 500;
  }

  .settings-controls {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: 20px;
    padding: 2px 8px;
  }

  .size-btn {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 12px;
    padding: 2px 4px;
    border-radius: 4px;
    transition: background 0.1s;
  }

  .size-btn:hover {
    background: var(--color-bg-active);
    color: var(--color-text-primary);
  }

  .size-val {
    font-size: 12px;
    color: var(--color-text-primary);
    min-width: 36px;
    text-align: center;
  }

  /* ===== Tag Panel ===== */
  .tag-panel {
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-divider);
    background: var(--color-bg-surface);
    flex-shrink: 0;
  }

  .tag-panel-header {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--color-text-tertiary);
    margin-bottom: 8px;
  }

  .tag-empty {
    font-size: 12px;
    color: var(--color-text-tertiary);
    margin: 0 0 8px;
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-bottom: 8px;
  }

  .tag-chip {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 3px 10px;
    border-radius: 20px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.12s;
  }

  .tag-chip:hover {
    border-color: var(--tag-color);
    color: var(--tag-color);
  }

  .tag-chip.active {
    background: color-mix(in srgb, var(--tag-color) 15%, transparent);
    border-color: var(--tag-color);
    color: var(--tag-color);
  }

  .tag-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--tag-color);
  }

  .new-tag-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .color-picker {
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    padding: 0;
    background: none;
  }

  .tag-input {
    flex: 1;
    height: 28px;
    padding: 0 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-base);
    color: var(--color-text-primary);
    font-size: 12px;
    outline: none;
  }

  .tag-input:focus {
    border-color: var(--color-accent);
  }

  .tag-add-btn {
    height: 28px;
    padding: 0 10px;
    background: var(--color-accent);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    font-size: 12px;
    cursor: pointer;
  }

  .tag-add-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  /* ===== Parse Error ===== */
  .parse-error {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    background: rgba(239, 68, 68, 0.1);
    border-bottom: 1px solid rgba(239, 68, 68, 0.2);
    font-size: 12px;
    color: var(--color-danger);
    flex-shrink: 0;
  }

  .error-close {
    margin-left: auto;
    background: none;
    border: none;
    color: var(--color-danger);
    font-size: 16px;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }

  /* ===== Reader Scroll ===== */
  .reader-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }

  .reader-content {
    margin: 0 auto;
    padding: 32px 40px 80px;
  }

  /* ===== Article Header ===== */
  .article-header {
    margin-bottom: 28px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--color-divider);
  }

  .article-source {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-accent);
    margin-bottom: 10px;
  }

  .article-title {
    font-size: 1.6em;
    font-weight: 700;
    line-height: 1.3;
    color: var(--color-text-primary);
    margin: 0 0 14px;
  }

  .article-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--color-text-tertiary);
    flex-wrap: wrap;
  }

  .meta-author {
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .meta-sep {
    opacity: 0.5;
  }

  .meta-link {
    color: var(--color-accent);
    text-decoration: none;
    font-weight: 500;
  }

  .meta-link:hover {
    text-decoration: underline;
  }

  /* ===== Article Body Prose Styles ===== */
  .article-body :global(p) {
    margin: 0 0 1em;
    color: var(--color-text-primary);
  }

  .article-body :global(h1),
  .article-body :global(h2),
  .article-body :global(h3),
  .article-body :global(h4),
  .article-body :global(h5),
  .article-body :global(h6) {
    color: var(--color-text-primary);
    font-weight: 700;
    line-height: 1.3;
    margin: 1.5em 0 0.5em;
  }

  .article-body :global(h1) { font-size: 1.5em; }
  .article-body :global(h2) { font-size: 1.3em; }
  .article-body :global(h3) { font-size: 1.15em; }

  .article-body :global(a) {
    color: var(--color-accent);
    text-decoration: none;
  }

  .article-body :global(a:hover) {
    text-decoration: underline;
  }

  .article-body :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: var(--radius-md);
    margin: 1em 0;
    display: block;
  }

  .article-body :global(blockquote) {
    margin: 1.2em 0;
    padding: 0.8em 1.2em;
    border-left: 3px solid var(--color-accent);
    background: var(--color-accent-soft);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    color: var(--color-text-secondary);
    font-style: italic;
  }

  .article-body :global(code) {
    font-family: "Cascadia Code", "Consolas", "Courier New", monospace;
    font-size: 0.87em;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    padding: 1px 5px;
    color: var(--color-text-primary);
  }

  .article-body :global(pre) {
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 1em 1.2em;
    overflow-x: auto;
    font-size: 0.87em;
    margin: 1.2em 0;
  }

  .article-body :global(pre code) {
    background: none;
    border: none;
    padding: 0;
    font-size: 1em;
  }

  .article-body :global(ul),
  .article-body :global(ol) {
    padding-left: 1.6em;
    margin: 0.8em 0;
  }

  .article-body :global(li) {
    margin-bottom: 0.3em;
  }

  .article-body :global(hr) {
    border: none;
    border-top: 1px solid var(--color-divider);
    margin: 2em 0;
  }

  .article-body :global(figure) {
    margin: 1.2em 0;
  }

  .article-body :global(figcaption) {
    font-size: 0.85em;
    color: var(--color-text-tertiary);
    text-align: center;
    margin-top: 0.4em;
    font-style: italic;
  }

  .article-body :global(table) {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9em;
    margin: 1em 0;
  }

  .article-body :global(th),
  .article-body :global(td) {
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    text-align: left;
  }

  .article-body :global(th) {
    background: var(--color-bg-surface);
    font-weight: 600;
  }

  /* ===== No Content ===== */
  .no-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 40px 0;
    color: var(--color-text-tertiary);
    text-align: center;
  }

  .no-content p {
    margin: 0;
    font-size: 14px;
  }

  .parse-prompt-btn {
    padding: 8px 16px;
    background: var(--color-accent-soft);
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-md);
    color: var(--color-accent);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s;
  }

  .parse-prompt-btn:hover:not(:disabled) {
    background: var(--color-accent);
    color: white;
  }

  .parse-prompt-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
