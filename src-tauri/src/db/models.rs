use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub position: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub folder_id: Option<i64>,
    pub title: String,
    pub url: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub favicon_id: Option<i64>,
    pub sync_id: Option<String>,
    pub auto_parse: bool,
    pub update_interval: i64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_fetched: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedWithCount {
    #[serde(flatten)]
    pub feed: Feed,
    pub unread_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub parsed_content: Option<String>,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub is_read_later: bool,
    pub sync_id: Option<String>,
    pub content_cached_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favicon {
    pub id: i64,
    pub hash: String,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncAccount {
    pub id: i64,
    pub provider: String,
    pub server_url: String,
    pub username: String,
    pub auth_token: Option<String>,
    pub last_synced: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleListItem {
    pub id: i64,
    pub feed_id: i64,
    pub feed_title: String,
    pub feed_icon_url: Option<String>,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub is_read_later: bool,
    pub has_parsed_content: bool,
    pub tag_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlePage {
    pub articles: Vec<ArticleListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedTreeFolder {
    pub folder: Folder,
    pub feeds: Vec<FeedWithCount>,
    pub children: Vec<FeedTreeFolder>,
    pub unread_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedTree {
    pub folders: Vec<FeedTreeFolder>,
    pub uncategorized: Vec<FeedWithCount>,
    pub total_unread: i64,
}
