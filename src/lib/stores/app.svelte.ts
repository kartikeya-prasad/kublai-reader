import type {
  FeedWithCount,
  Article,
  ArticleListItem,
  ArticlePage,
  Tag,
  FeedTree,
  FeedTreeFolder,
  ArticleFilter,
  ArticleView,
  ReaderSettings,
} from "$lib/types";
import {
  getFeeds,
  getArticles,
  getArticle,
  markRead,
  markUnread,
  markAllRead,
  toggleStar,
  toggleReadLater,
  refreshFeed,
  refreshAllFeeds,
  searchArticles,
  getTags,
  getFavicon,
  summarizeArticle,
} from "$lib/utils/tauri";

// ===== State =====

let feedTree = $state<FeedTree>({
  folders: [],
  uncategorized: [],
  total_unread: 0,
});

let selectedFeed = $state<FeedWithCount | null>(null);
let selectedFolder = $state<FeedTreeFolder | null>(null);
let selectedFilter = $state<ArticleFilter>("all");
let selectedArticle = $state<Article | null>(null);
let selectedArticleListItem = $state<ArticleListItem | null>(null);

let articlePage = $state<ArticlePage>({
  articles: [],
  total: 0,
  page: 0,
  page_size: 50,
});

let isLoadingFeeds = $state(false);
let isLoadingArticles = $state(false);
let isLoadingArticle = $state(false);
let isRefreshing = $state(false);

let articleView = $state<ArticleView>("card");

let readerSettings = $state<ReaderSettings>({
  font_family: "Segoe UI Variable",
  font_size: 17,
  line_height: 1.7,
  content_width: 720,
});

let tags = $state<Tag[]>([]);
let searchQuery = $state("");
let isSearchActive = $state(false);
let searchResults = $state<ArticleListItem[]>([]);

let showAddFeedDialog = $state(false);
let showSettingsDialog = $state(false);

let selectedArticleIndex = $state(-1);
let faviconCache = $state(new Map<number, string>());

// AI summary state
let aiSummary = $state<string | null>(null);
let isLoadingSummary = $state(false);
let summaryError = $state<string | null>(null);

// ===== Derived state =====

let currentPaneTitle = $derived.by(() => {
  if (isSearchActive && searchQuery) return `Search: "${searchQuery}"`;
  if (selectedFeed) return selectedFeed.title;
  if (selectedFolder) return selectedFolder.folder.name;
  if (selectedFilter === "unread") return "Unread";
  if (selectedFilter === "starred") return "Starred";
  if (selectedFilter === "read_later") return "Read Later";
  if (typeof selectedFilter === "object" && "tagged" in selectedFilter) {
    const tag = tags.find((t) => t.id === (selectedFilter as { tagged: number }).tagged);
    return tag ? tag.name : "Tagged";
  }
  return "All Articles";
});

let displayedArticles = $derived(isSearchActive ? searchResults : articlePage.articles);

// ===== Functions =====

export async function loadFeedTree(): Promise<void> {
  isLoadingFeeds = true;
  try {
    const tree = await getFeeds();
    feedTree = tree;
  } catch (e) {
    console.error("Failed to load feed tree:", e);
  } finally {
    isLoadingFeeds = false;
  }
}

export async function loadArticles(page = 0, append = false): Promise<void> {
  isLoadingArticles = true;
  try {
    const filterStr = getFilterString(selectedFilter);
    const result = await getArticles({
      feedId: selectedFeed?.id ?? null,
      folderId: selectedFolder?.folder.id ?? null,
      filter: filterStr,
      page,
      pageSize: 50,
    });

    if (append) {
      articlePage = {
        ...result,
        articles: [...articlePage.articles, ...result.articles],
      };
    } else {
      articlePage = result;
    }
  } catch (e) {
    console.error("Failed to load articles:", e);
  } finally {
    isLoadingArticles = false;
  }
}

export async function loadNextPage(): Promise<void> {
  const nextPage = articlePage.page + 1;
  const totalPages = Math.ceil(articlePage.total / articlePage.page_size);
  if (nextPage < totalPages && !isLoadingArticles) {
    await loadArticles(nextPage, true);
  }
}

export async function selectFeed(feed: FeedWithCount | null): Promise<void> {
  selectedFeed = feed;
  selectedFolder = null;
  selectedArticle = null;
  selectedArticleListItem = null;
  isSearchActive = false;
  await loadArticles(0, false);
}

export async function selectFolder(folder: FeedTreeFolder | null): Promise<void> {
  selectedFolder = folder;
  selectedFeed = null;
  selectedArticle = null;
  selectedArticleListItem = null;
  isSearchActive = false;
  await loadArticles(0, false);
}

export async function selectFilter(filter: ArticleFilter): Promise<void> {
  selectedFilter = filter;
  selectedFeed = null;
  selectedFolder = null;
  selectedArticle = null;
  selectedArticleListItem = null;
  isSearchActive = false;
  await loadArticles(0, false);
}

export async function selectArticle(item: ArticleListItem): Promise<void> {
  selectedArticleListItem = item;
  isLoadingArticle = true;
  // Clear AI summary when switching articles
  aiSummary = null;
  summaryError = null;
  try {
    const article = await getArticle(item.id);
    selectedArticle = article;
    selectedArticleIndex = displayedArticles.findIndex(a => a.id === article.id);
    // Mark as read
    if (!item.is_read) {
      await markArticleRead(item.id);
    }
  } catch (e) {
    console.error("Failed to load article:", e);
  } finally {
    isLoadingArticle = false;
  }
}

export async function markArticleRead(articleId: number): Promise<void> {
  await markRead(articleId);
  // Update in list
  articlePage = {
    ...articlePage,
    articles: articlePage.articles.map((a) =>
      a.id === articleId ? { ...a, is_read: true } : a
    ),
  };
  // Update unread counts
  feedTree = {
    ...feedTree,
    total_unread: Math.max(0, feedTree.total_unread - 1),
  };
}

export async function markArticleUnread(articleId: number): Promise<void> {
  await markUnread(articleId);
  articlePage = {
    ...articlePage,
    articles: articlePage.articles.map((a) =>
      a.id === articleId ? { ...a, is_read: false } : a
    ),
  };
}

export async function toggleArticleStar(articleId: number): Promise<void> {
  const newState = await toggleStar(articleId);
  articlePage = {
    ...articlePage,
    articles: articlePage.articles.map((a) =>
      a.id === articleId ? { ...a, is_starred: newState } : a
    ),
  };
  if (selectedArticle?.id === articleId) {
    selectedArticle = { ...selectedArticle, is_starred: newState };
  }
}

export async function toggleArticleReadLater(articleId: number): Promise<void> {
  const newState = await toggleReadLater(articleId);
  articlePage = {
    ...articlePage,
    articles: articlePage.articles.map((a) =>
      a.id === articleId ? { ...a, is_read_later: newState } : a
    ),
  };
  if (selectedArticle?.id === articleId) {
    selectedArticle = { ...selectedArticle, is_read_later: newState };
  }
}

export async function markFeedAllRead(feedId: number): Promise<void> {
  await markAllRead({ feedId });
  articlePage = {
    ...articlePage,
    articles: articlePage.articles.map((a) =>
      a.feed_id === feedId ? { ...a, is_read: true } : a
    ),
  };
  await loadFeedTree();
}

export async function markFolderAllRead(folderId: number): Promise<void> {
  await markAllRead({ folderId });
  articlePage = {
    ...articlePage,
    articles: articlePage.articles.map((a) => ({ ...a, is_read: true })),
  };
  await loadFeedTree();
}

export async function refreshOneFeed(feedId: number): Promise<void> {
  isRefreshing = true;
  try {
    await refreshFeed(feedId);
    await loadFeedTree();
    if (selectedFeed?.id === feedId) {
      await loadArticles(0, false);
    }
  } catch (e) {
    console.error("Failed to refresh feed:", e);
  } finally {
    isRefreshing = false;
  }
}

export async function refreshAll(): Promise<void> {
  isRefreshing = true;
  try {
    await refreshAllFeeds();
    await loadFeedTree();
    await loadArticles(0, false);
  } catch (e) {
    console.error("Failed to refresh all feeds:", e);
  } finally {
    isRefreshing = false;
  }
}

export async function doSearch(query: string): Promise<void> {
  searchQuery = query;
  if (!query.trim()) {
    isSearchActive = false;
    searchResults = [];
    return;
  }
  isSearchActive = true;
  isLoadingArticles = true;
  try {
    const results = await searchArticles(query, {
      feedId: selectedFeed?.id ?? null,
      folderId: selectedFolder?.folder.id ?? null,
    });
    searchResults = results;
  } catch (e) {
    console.error("Search failed:", e);
    searchResults = [];
  } finally {
    isLoadingArticles = false;
  }
}

export async function clearSearch(): Promise<void> {
  searchQuery = "";
  isSearchActive = false;
  searchResults = [];
}

export async function loadTags(): Promise<void> {
  try {
    tags = await getTags();
  } catch (e) {
    console.error("Failed to load tags:", e);
  }
}

export function setArticleView(view: ArticleView): void {
  articleView = view;
}

export function updateReaderSettings(settings: Partial<ReaderSettings>): void {
  readerSettings = { ...readerSettings, ...settings };
}

export function updateSelectedArticleParsedContent(content: string): void {
  if (selectedArticle) {
    selectedArticle = { ...selectedArticle, parsed_content: content };
  }
}

export function openAddFeedDialog(): void {
  showAddFeedDialog = true;
}

export function closeAddFeedDialog(): void {
  showAddFeedDialog = false;
}

export function openSettingsDialog(): void {
  showSettingsDialog = true;
}

export function closeSettingsDialog(): void {
  showSettingsDialog = false;
}

export function navigateArticle(delta: number): void {
  const articles = displayedArticles;
  if (!articles.length) return;
  const newIndex = Math.max(0, Math.min(articles.length - 1, selectedArticleIndex + delta));
  if (newIndex !== selectedArticleIndex) {
    selectArticle(articles[newIndex]);
  }
}

export async function fetchAiSummary(articleId: number): Promise<void> {
  if (isLoadingSummary) return;
  aiSummary = null;
  summaryError = null;
  isLoadingSummary = true;
  try {
    const summary = await summarizeArticle(articleId);
    aiSummary = summary;
  } catch (e: unknown) {
    summaryError = e instanceof Error ? e.message : String(e);
  } finally {
    isLoadingSummary = false;
  }
}

export function clearAiSummary(): void {
  aiSummary = null;
  summaryError = null;
}

export async function getFaviconCached(feedId: number): Promise<string | null> {
  if (faviconCache.has(feedId)) return faviconCache.get(feedId) ?? null;
  const uri = await getFavicon(feedId);
  if (uri) faviconCache.set(feedId, uri);
  return uri ?? null;
}

// ===== Helpers =====

function getFilterString(filter: ArticleFilter): string {
  if (filter === "all") return "all";
  if (filter === "unread") return "unread";
  if (filter === "starred") return "starred";
  if (filter === "read_later") return "read_later";
  if (typeof filter === "object" && "tagged" in filter) return "all";
  return "all";
}

// ===== Exported getters =====
// Svelte 5 module-level state is reactive when accessed in components,
// but we export getter functions to allow clean imports.

export function getState() {
  return {
    get feedTree() { return feedTree; },
    get selectedFeed() { return selectedFeed; },
    get selectedFolder() { return selectedFolder; },
    get selectedFilter() { return selectedFilter; },
    get selectedArticle() { return selectedArticle; },
    get selectedArticleListItem() { return selectedArticleListItem; },
    get articlePage() { return articlePage; },
    get isLoadingFeeds() { return isLoadingFeeds; },
    get isLoadingArticles() { return isLoadingArticles; },
    get isLoadingArticle() { return isLoadingArticle; },
    get isRefreshing() { return isRefreshing; },
    get articleView() { return articleView; },
    get readerSettings() { return readerSettings; },
    get tags() { return tags; },
    get searchQuery() { return searchQuery; },
    get isSearchActive() { return isSearchActive; },
    get displayedArticles() { return displayedArticles; },
    get currentPaneTitle() { return currentPaneTitle; },
    get showAddFeedDialog() { return showAddFeedDialog; },
    get showSettingsDialog() { return showSettingsDialog; },
    get selectedArticleIndex() { return selectedArticleIndex; },
    get faviconCache() { return faviconCache; },
    get aiSummary() { return aiSummary; },
    get isLoadingSummary() { return isLoadingSummary; },
    get summaryError() { return summaryError; },
  };
}
