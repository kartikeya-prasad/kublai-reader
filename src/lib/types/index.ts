// ===== Database Models =====

export interface Folder {
  id: number;
  name: string;
  parent_id: number | null;
  position: number;
  created_at: string;
}

export interface Feed {
  id: number;
  folder_id: number | null;
  title: string;
  url: string;
  site_url: string | null;
  description: string | null;
  icon_url: string | null;
  auto_parse: boolean;
  update_interval: number;
  etag: string | null;
  last_modified: string | null;
  last_fetched: string | null;
  created_at: string;
}

export interface FeedWithCount extends Feed {
  unread_count: number;
  total_count: number;
}

export interface Article {
  id: number;
  feed_id: number;
  guid: string;
  title: string;
  url: string | null;
  author: string | null;
  summary: string | null;
  content: string | null;
  parsed_content: string | null;
  thumbnail_url: string | null;
  published_at: string | null;
  is_read: boolean;
  is_starred: boolean;
  is_read_later: boolean;
  created_at: string;
}

export interface ArticleListItem {
  id: number;
  feed_id: number;
  feed_title: string;
  feed_icon_url: string | null;
  title: string;
  url: string | null;
  author: string | null;
  summary: string | null;
  thumbnail_url: string | null;
  published_at: string | null;
  is_read: boolean;
  is_starred: boolean;
  is_read_later: boolean;
  has_parsed_content: boolean;
  tag_ids: number[];
}

export interface Tag {
  id: number;
  name: string;
  color: string;
  created_at: string;
}

// ===== UI State Types =====

export type ArticleView = "card" | "compact";
export type ThemeMode = "light" | "dark" | "auto";
export type ArticleFilter =
  | "all"
  | "unread"
  | "starred"
  | "read_later"
  | { tagged: number };

export interface ReaderSettings {
  font_family: string;
  font_size: number;
  line_height: number;
  content_width: number;
}

export interface ArticlePage {
  articles: ArticleListItem[];
  total: number;
  page: number;
  page_size: number;
}

// ===== Feed Tree =====

export interface FeedTreeFolder {
  folder: Folder;
  feeds: FeedWithCount[];
  children: FeedTreeFolder[];
  unread_count: number;
}

export interface FeedTree {
  folders: FeedTreeFolder[];
  uncategorized: FeedWithCount[];
  total_unread: number;
}
