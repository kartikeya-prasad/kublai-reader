use crate::db::AppDatabase;
use crate::reader::extractor;
use tauri::State;

#[tauri::command]
pub async fn parse_article(
    article_id: i64,
    db: State<'_, AppDatabase>,
) -> Result<String, String> {
    // Get article URL
    let url: String = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT url FROM articles WHERE id = ?1",
            [article_id],
            |row| row.get(0),
        ).map_err(|e| format!("Article not found: {}", e))?
    };

    // Fetch the page HTML
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) KublaiReader/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let html = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch article: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Extract readable content
    let extracted = tokio::task::spawn_blocking(move || {
        extractor::extract(&html, &url)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    // Store parsed content
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE articles SET parsed_content = ?1 WHERE id = ?2",
            rusqlite::params![extracted.content, article_id],
        ).map_err(|e| e.to_string())?;
    }

    Ok(extracted.content)
}
