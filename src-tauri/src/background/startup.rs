use crate::db::AppDatabase;
use crate::reader;
use tauri::AppHandle;
use tauri::Manager;

/// On app startup, extract full text for any articles belonging to auto_parse feeds
/// that don't yet have parsed_content. Runs sequentially with short delays to avoid
/// hammering websites. Capped at 50 articles per startup.
pub async fn extract_pending_articles(handle: AppHandle) {
    let db_state = handle.state::<AppDatabase>();

    // Query articles needing extraction
    let pending: Vec<(i64, String)> = {
        let conn = match db_state.conn.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut stmt = match conn.prepare(
            "SELECT a.id, a.url
             FROM articles a
             JOIN feeds f ON f.id = a.feed_id
             WHERE f.auto_parse = 1
               AND a.parsed_content IS NULL
               AND a.url IS NOT NULL
               AND a.url != ''
             ORDER BY a.published_at DESC
             LIMIT 50"
        ) {
            Ok(s) => s,
            Err(_) => return,
        };

        stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    };

    if pending.is_empty() {
        return;
    }

    log::info!("startup: extracting {} pending articles", pending.len());

    for (article_id, url) in pending {
        match reader::fetch_and_extract(&url).await {
            Ok(content) => {
                if let Ok(conn) = db_state.conn.lock() {
                    let _ = conn.execute(
                        "UPDATE articles SET parsed_content = ?1 WHERE id = ?2",
                        rusqlite::params![content, article_id],
                    );
                }
            }
            Err(e) => {
                log::warn!("startup: failed to extract article {}: {}", article_id, e);
            }
        }
        // Small delay between requests to be polite to servers
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    log::info!("startup: article extraction complete");
}
