use crate::db::models::{Feed, FeedTree, FeedTreeFolder, FeedWithCount, Article};
use crate::db::AppDatabase;
use crate::feed;
use tauri::State;

#[tauri::command]
pub async fn get_favicon(feed_id: i64, db: State<'_, AppDatabase>) -> Result<Option<String>, String> {
    crate::favicon::fetcher::get_favicon_data_uri(feed_id, &db)
}

#[tauri::command]
pub async fn add_feed(
    url: String,
    folder_id: Option<i64>,
    db: State<'_, AppDatabase>,
) -> Result<Feed, String> {
    // Detect and fetch feed
    let (feed_url, parsed) = feed::fetcher::fetch_and_detect(&url)
        .await
        .map_err(|e| format!("Failed to fetch feed: {}", e))?;

    let title = parsed.title.map(|t| t.content).unwrap_or_else(|| "Untitled Feed".to_string());
    let site_url = parsed.links.first().map(|l| l.href.clone());
    let description = parsed.description.map(|d| d.content);

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Insert feed
    conn.execute(
        "INSERT INTO feeds (title, url, site_url, description, folder_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![title, feed_url, site_url, description, folder_id],
    ).map_err(|e| format!("Failed to insert feed: {}", e))?;

    let feed_id = conn.last_insert_rowid();

    // Insert articles from initial fetch
    for entry in &parsed.entries {
        let guid = entry.id.clone();
        let entry_title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
        let entry_url = entry.links.first().map(|l| l.href.clone());
        let author = entry.authors.first().map(|a| a.name.clone());
        let summary = entry.summary.as_ref().map(|s| s.content.clone());
        let content = entry.content.as_ref().and_then(|c| c.body.clone());
        let thumbnail = entry.media.first()
            .and_then(|m| m.thumbnails.first())
            .map(|t| t.image.uri.clone());
        let published = entry.published.or(entry.updated)
            .map(|d| d.to_rfc3339());

        let _ = conn.execute(
            "INSERT OR IGNORE INTO articles (feed_id, guid, title, url, author, summary, content, thumbnail_url, published_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![feed_id, guid, entry_title, entry_url, author, summary, content, thumbnail, published],
        );
    }

    // Return the created feed
    let feed = conn.query_row(
        "SELECT id, folder_id, title, url, site_url, description, icon_url, favicon_id, sync_id, auto_parse, update_interval, etag, last_modified, last_fetched, created_at FROM feeds WHERE id = ?1",
        [feed_id],
        |row| {
            Ok(Feed {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                site_url: row.get(4)?,
                description: row.get(5)?,
                icon_url: row.get(6)?,
                favicon_id: row.get(7)?,
                sync_id: row.get(8)?,
                auto_parse: row.get::<_, i64>(9)? != 0,
                update_interval: row.get(10)?,
                etag: row.get(11)?,
                last_modified: row.get(12)?,
                last_fetched: row.get(13)?,
                created_at: row.get(14)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    Ok(feed)
}

#[tauri::command]
pub async fn get_feeds(db: State<'_, AppDatabase>) -> Result<FeedTree, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Get all folders
    let mut folder_stmt = conn.prepare(
        "SELECT id, name, parent_id, position, created_at FROM folders ORDER BY position"
    ).map_err(|e| e.to_string())?;

    let folders: Vec<crate::db::models::Folder> = folder_stmt.query_map([], |row| {
        Ok(crate::db::models::Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            position: row.get(3)?,
            created_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Get all feeds with unread counts
    let mut feed_stmt = conn.prepare(
        "SELECT f.id, f.folder_id, f.title, f.url, f.site_url, f.description, f.icon_url,
                f.favicon_id, f.sync_id, f.auto_parse, f.update_interval, f.etag, f.last_modified, f.last_fetched, f.created_at,
                COUNT(a.id) as total_count,
                COUNT(CASE WHEN a.is_read = 0 THEN 1 END) as unread_count
         FROM feeds f
         LEFT JOIN articles a ON a.feed_id = f.id
         GROUP BY f.id
         ORDER BY f.title"
    ).map_err(|e| e.to_string())?;

    let feeds_with_counts: Vec<FeedWithCount> = feed_stmt.query_map([], |row| {
        Ok(FeedWithCount {
            feed: Feed {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                site_url: row.get(4)?,
                description: row.get(5)?,
                icon_url: row.get(6)?,
                favicon_id: row.get(7)?,
                sync_id: row.get(8)?,
                auto_parse: row.get::<_, i64>(9)? != 0,
                update_interval: row.get(10)?,
                etag: row.get(11)?,
                last_modified: row.get(12)?,
                last_fetched: row.get(13)?,
                created_at: row.get(14)?,
            },
            total_count: row.get(15)?,
            unread_count: row.get(16)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Build tree
    let uncategorized: Vec<FeedWithCount> = feeds_with_counts.iter()
        .filter(|f| f.feed.folder_id.is_none())
        .cloned()
        .collect();

    let total_unread: i64 = feeds_with_counts.iter().map(|f| f.unread_count).sum();

    fn build_folder_tree(
        folder: &crate::db::models::Folder,
        all_folders: &[crate::db::models::Folder],
        all_feeds: &[FeedWithCount],
    ) -> FeedTreeFolder {
        let feeds: Vec<FeedWithCount> = all_feeds.iter()
            .filter(|f| f.feed.folder_id == Some(folder.id))
            .cloned()
            .collect();

        let children: Vec<FeedTreeFolder> = all_folders.iter()
            .filter(|f| f.parent_id == Some(folder.id))
            .map(|f| build_folder_tree(f, all_folders, all_feeds))
            .collect();

        let unread_count: i64 = feeds.iter().map(|f| f.unread_count).sum::<i64>()
            + children.iter().map(|c| c.unread_count).sum::<i64>();

        FeedTreeFolder {
            folder: folder.clone(),
            feeds,
            children,
            unread_count,
        }
    }

    let tree_folders: Vec<FeedTreeFolder> = folders.iter()
        .filter(|f| f.parent_id.is_none())
        .map(|f| build_folder_tree(f, &folders, &feeds_with_counts))
        .collect();

    Ok(FeedTree {
        folders: tree_folders,
        uncategorized,
        total_unread,
    })
}

#[tauri::command]
pub async fn refresh_feed(
    feed_id: i64,
    db: State<'_, AppDatabase>,
) -> Result<Vec<Article>, String> {
    let (feed_url, etag, last_modified) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT url, etag, last_modified FROM feeds WHERE id = ?1",
            [feed_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?)),
        ).map_err(|e| e.to_string())?
    };

    let result = feed::fetcher::fetch_feed(&feed_url, etag.as_deref(), last_modified.as_deref())
        .await
        .map_err(|e| format!("Failed to fetch feed: {}", e))?;

    let Some((parsed, new_etag, new_last_modified)) = result else {
        // 304 Not Modified
        return Ok(vec![]);
    };

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Update feed metadata
    conn.execute(
        "UPDATE feeds SET etag = ?1, last_modified = ?2, last_fetched = datetime('now') WHERE id = ?3",
        rusqlite::params![new_etag, new_last_modified, feed_id],
    ).map_err(|e| e.to_string())?;

    let mut new_articles = Vec::new();

    for entry in &parsed.entries {
        let guid = entry.id.clone();
        let title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
        let url = entry.links.first().map(|l| l.href.clone());
        let author = entry.authors.first().map(|a| a.name.clone());
        let summary = entry.summary.as_ref().map(|s| s.content.clone());
        let content = entry.content.as_ref().and_then(|c| c.body.clone());
        let thumbnail = entry.media.first()
            .and_then(|m| m.thumbnails.first())
            .map(|t| t.image.uri.clone());
        let published = entry.published.or(entry.updated)
            .map(|d| d.to_rfc3339());

        let inserted = conn.execute(
            "INSERT OR IGNORE INTO articles (feed_id, guid, title, url, author, summary, content, thumbnail_url, published_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![feed_id, guid, title, url, author, summary, content, thumbnail, published],
        ).map_err(|e| e.to_string())?;

        if inserted > 0 {
            let article_id = conn.last_insert_rowid();
            new_articles.push(Article {
                id: article_id,
                feed_id,
                guid,
                title,
                url,
                author,
                summary,
                content,
                parsed_content: None,
                thumbnail_url: thumbnail,
                published_at: published,
                is_read: false,
                is_starred: false,
                is_read_later: false,
                sync_id: None,
                content_cached_at: None,
                ai_summary: None,
                created_at: String::new(),
            });
        }
    }

    Ok(new_articles)
}

#[tauri::command]
pub async fn refresh_all_feeds(db: State<'_, AppDatabase>) -> Result<(), String> {
    let feed_ids: Vec<i64> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id FROM feeds").map_err(|e| e.to_string())?;
        let ids: Vec<i64> = stmt.query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        ids
    };

    // Note: In a real implementation, we'd use tokio tasks with a semaphore.
    // For now, refresh sequentially since we hold a Mutex on the connection.
    for feed_id in feed_ids {
        let _ = refresh_feed(feed_id, db.clone()).await;
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_feed(feed_id: i64, db: State<'_, AppDatabase>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM feeds WHERE id = ?1", [feed_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_feed(
    feed_id: i64,
    title: Option<String>,
    folder_id: Option<i64>,
    auto_parse: Option<bool>,
    update_interval: Option<i64>,
    db: State<'_, AppDatabase>,
) -> Result<Feed, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    if let Some(t) = title {
        conn.execute("UPDATE feeds SET title = ?1 WHERE id = ?2", rusqlite::params![t, feed_id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(fid) = folder_id {
        conn.execute("UPDATE feeds SET folder_id = ?1 WHERE id = ?2", rusqlite::params![fid, feed_id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(ap) = auto_parse {
        conn.execute("UPDATE feeds SET auto_parse = ?1 WHERE id = ?2", rusqlite::params![ap as i64, feed_id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(ui) = update_interval {
        conn.execute("UPDATE feeds SET update_interval = ?1 WHERE id = ?2", rusqlite::params![ui, feed_id])
            .map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT id, folder_id, title, url, site_url, description, icon_url, favicon_id, sync_id, auto_parse, update_interval, etag, last_modified, last_fetched, created_at FROM feeds WHERE id = ?1",
        [feed_id],
        |row| {
            Ok(Feed {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                site_url: row.get(4)?,
                description: row.get(5)?,
                icon_url: row.get(6)?,
                favicon_id: row.get(7)?,
                sync_id: row.get(8)?,
                auto_parse: row.get::<_, i64>(9)? != 0,
                update_interval: row.get(10)?,
                etag: row.get(11)?,
                last_modified: row.get(12)?,
                last_fetched: row.get(13)?,
                created_at: row.get(14)?,
            })
        },
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_opml(xml: String, db: State<'_, AppDatabase>) -> Result<Vec<Feed>, String> {
    let items = feed::opml::parse_opml(&xml).map_err(|e| e.to_string())?;
    let mut feeds = Vec::new();
    let mut folder_cache: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    for item in items {
        // Resolve folder_name -> folder_id, creating folder if needed
        let folder_id = match item.folder_name {
            Some(ref name) => {
                if let Some(&id) = folder_cache.get(name) {
                    Some(id)
                } else {
                    let id = {
                        let conn = db.conn.lock().map_err(|e| e.to_string())?;
                        // Check if folder already exists
                        let existing: Option<i64> = conn.query_row(
                            "SELECT id FROM folders WHERE name = ?1 AND parent_id IS NULL",
                            [name],
                            |row| row.get(0),
                        ).ok();
                        if let Some(eid) = existing {
                            eid
                        } else {
                            let pos: i64 = conn.query_row(
                                "SELECT COALESCE(MAX(position), -1) + 1 FROM folders WHERE parent_id IS NULL",
                                [],
                                |row| row.get(0),
                            ).unwrap_or(0);
                            conn.execute(
                                "INSERT INTO folders (name, position) VALUES (?1, ?2)",
                                rusqlite::params![name, pos],
                            ).map_err(|e| e.to_string())?;
                            conn.last_insert_rowid()
                        }
                    };
                    folder_cache.insert(name.clone(), id);
                    Some(id)
                }
            }
            None => None,
        };

        match add_feed(item.xml_url, folder_id, db.clone()).await {
            Ok(feed) => feeds.push(feed),
            Err(e) => log::warn!("Failed to import feed {}: {}", item.title, e),
        }
    }

    Ok(feeds)
}

#[tauri::command]
pub async fn export_opml(db: State<'_, AppDatabase>) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT f.title, f.url, f.site_url, fo.name as folder_name
         FROM feeds f
         LEFT JOIN folders fo ON f.folder_id = fo.id
         ORDER BY fo.name, f.title"
    ).map_err(|e| e.to_string())?;

    let rows: Vec<(String, String, Option<String>, Option<String>)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(feed::opml::generate_opml(&rows))
}
