use tauri::{AppHandle, Emitter, Manager};
use std::time::Duration;

/// Background feed refresh scheduler.
/// Checks feeds periodically and refreshes those that are due.
pub async fn run(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Check every 5 minutes

    loop {
        interval.tick().await;

        let db = match app.try_state::<crate::db::AppDatabase>() {
            Some(db) => db,
            None => continue,
        };

        // Get feeds that need refreshing
        let feeds_to_refresh: Vec<(i64, String)> = {
            let Ok(conn) = db.conn.lock() else { continue };
            let Ok(mut stmt) = conn.prepare(
                "SELECT id, url FROM feeds
                 WHERE last_fetched IS NULL
                    OR (strftime('%s', 'now') - strftime('%s', last_fetched)) > update_interval
                 ORDER BY last_fetched ASC
                 LIMIT 10"
            ) else { continue };
            let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            }) else { continue };
            let result: Vec<(i64, String)> = rows.filter_map(|r| r.ok()).collect();
            result
        };

        for (feed_id, _url) in feeds_to_refresh {
            let db = match app.try_state::<crate::db::AppDatabase>() {
                Some(db) => db,
                None => continue,
            };

            let feed_info = {
                let conn = match db.conn.lock() {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                match conn.query_row(
                    "SELECT url, etag, last_modified FROM feeds WHERE id = ?1",
                    [feed_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?)),
                ) {
                    Ok(info) => info,
                    Err(_) => continue,
                }
            };

            let (feed_url, etag, last_modified) = feed_info;

            match crate::feed::fetcher::fetch_feed(&feed_url, etag.as_deref(), last_modified.as_deref()).await {
                Ok(Some((parsed, new_etag, new_last_modified))) => {
                    let conn = match db.conn.lock() {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let _ = conn.execute(
                        "UPDATE feeds SET etag = ?1, last_modified = ?2, last_fetched = datetime('now') WHERE id = ?3",
                        rusqlite::params![new_etag, new_last_modified, feed_id],
                    );

                    let mut new_count = 0i64;
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

                        if let Ok(n) = conn.execute(
                            "INSERT OR IGNORE INTO articles (feed_id, guid, title, url, author, summary, content, thumbnail_url, published_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            rusqlite::params![feed_id, guid, title, url, author, summary, content, thumbnail, published],
                        ) {
                            new_count += n as i64;
                        }
                    }

                    if new_count > 0 {
                        let _ = app.emit("feed-updated", serde_json::json!({
                            "feed_id": feed_id,
                            "new_articles": new_count,
                        }));
                    }

                    log::info!("Refreshed feed {} ({} new articles)", feed_id, new_count);
                }
                Ok(None) => {
                    // Not modified, update last_fetched timestamp
                    if let Ok(conn) = db.conn.lock() {
                        let _ = conn.execute(
                            "UPDATE feeds SET last_fetched = datetime('now') WHERE id = ?1",
                            [feed_id],
                        );
                    }
                }
                Err(e) => {
                    log::warn!("Failed to refresh feed {}: {}", feed_id, e);
                }
            }
        }
    }
}
