import { invoke } from "@tauri-apps/api/core";
import type {
  Feed,
  Article,
  ArticleListItem,
  ArticlePage,
  Folder,
  Tag,
  FeedTree,
  SyncAccount,
  SyncResult,
} from "$lib/types";

// ===== Feed Commands =====

export async function addFeed(
  url: string,
  folderId?: number | null
): Promise<Feed> {
  return invoke("add_feed", { url, folderId });
}

export async function getFeeds(): Promise<FeedTree> {
  return invoke("get_feeds");
}

export async function refreshFeed(feedId: number): Promise<Article[]> {
  return invoke("refresh_feed", { feedId });
}

export async function refreshAllFeeds(): Promise<void> {
  return invoke("refresh_all_feeds");
}

export async function deleteFeed(feedId: number): Promise<void> {
  return invoke("delete_feed", { feedId });
}

export async function updateFeed(
  feedId: number,
  updates: Partial<Pick<Feed, "title" | "folder_id" | "auto_parse" | "update_interval">>
): Promise<Feed> {
  return invoke("update_feed", { feedId, ...updates });
}

export async function importOpml(xml: string): Promise<Feed[]> {
  return invoke("import_opml", { xml });
}

export async function exportOpml(): Promise<string> {
  return invoke("export_opml");
}

// ===== Folder Commands =====

export async function createFolder(
  name: string,
  parentId?: number | null
): Promise<Folder> {
  return invoke("create_folder", { name, parentId });
}

export async function deleteFolder(folderId: number): Promise<void> {
  return invoke("delete_folder", { folderId });
}

export async function renameFolder(
  folderId: number,
  name: string
): Promise<Folder> {
  return invoke("rename_folder", { folderId, name });
}

// ===== Article Commands =====

export async function getArticles(params: {
  feedId?: number | null;
  folderId?: number | null;
  filter?: string;
  page?: number;
  pageSize?: number;
}): Promise<ArticlePage> {
  return invoke("get_articles", params);
}

export async function getArticle(articleId: number): Promise<Article> {
  return invoke("get_article", { articleId });
}

export async function markRead(articleId: number): Promise<void> {
  return invoke("mark_read", { articleId });
}

export async function markUnread(articleId: number): Promise<void> {
  return invoke("mark_unread", { articleId });
}

export async function markAllRead(params: {
  feedId?: number | null;
  folderId?: number | null;
}): Promise<void> {
  return invoke("mark_all_read", params);
}

export async function markAboveRead(
  feedId: number,
  articleId: number
): Promise<void> {
  return invoke("mark_above_read", { feedId, articleId });
}

export async function toggleStar(articleId: number): Promise<boolean> {
  return invoke("toggle_star", { articleId });
}

export async function toggleReadLater(articleId: number): Promise<boolean> {
  return invoke("toggle_read_later", { articleId });
}

// ===== Parser Commands =====

export async function parseArticle(articleId: number): Promise<string> {
  return invoke("parse_article", { articleId });
}

// ===== Tag Commands =====

export async function createTag(
  name: string,
  color: string
): Promise<Tag> {
  return invoke("create_tag", { name, color });
}

export async function getTags(): Promise<Tag[]> {
  return invoke("get_tags");
}

export async function addTagToArticle(
  articleId: number,
  tagId: number
): Promise<void> {
  return invoke("add_tag_to_article", { articleId, tagId });
}

export async function removeTagFromArticle(
  articleId: number,
  tagId: number
): Promise<void> {
  return invoke("remove_tag_from_article", { articleId, tagId });
}

// ===== Search =====

export async function searchArticles(
  query: string,
  params?: {
    feedId?: number | null;
    folderId?: number | null;
    filter?: string;
  }
): Promise<ArticleListItem[]> {
  return invoke("search_articles", { query, ...params });
}

// ===== Favicon =====

export async function getFavicon(feedId: number): Promise<string | null> {
  return invoke<string | null>('get_favicon', { feedId });
}

// ===== Sync =====

export async function getSyncAccounts(): Promise<SyncAccount[]> {
  return invoke<SyncAccount[]>('get_sync_accounts');
}

export async function addSyncAccount(provider: string, serverUrl: string, username: string, password: string): Promise<SyncAccount> {
  return invoke<SyncAccount>('add_sync_account', { provider, serverUrl, username, password });
}

export async function removeSyncAccount(accountId: number): Promise<void> {
  return invoke<void>('remove_sync_account', { accountId });
}

export async function syncNow(accountId: number): Promise<SyncResult> {
  return invoke<SyncResult>('sync_now', { accountId });
}

// ===== AI =====

export async function summarizeArticle(articleId: number): Promise<string> {
  return invoke<string>('summarize_article', { articleId });
}

// ===== Settings =====

export async function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}
