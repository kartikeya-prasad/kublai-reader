pub mod greader;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFeed {
    pub id: String,
    pub title: String,
    pub url: String,
    pub site_url: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntry {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub published: Option<i64>,
    pub is_read: bool,
    pub is_starred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub feeds_synced: usize,
    pub entries_synced: usize,
    pub errors: Vec<String>,
}
