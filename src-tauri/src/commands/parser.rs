use crate::db::AppDatabase;
use crate::reader;
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

    // Fetch and extract using shared function
    let content = reader::fetch_and_extract(&url)
        .await
        .map_err(|e| format!("Failed to extract article: {}", e))?;

    // Store parsed content
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE articles SET parsed_content = ?1 WHERE id = ?2",
            rusqlite::params![content, article_id],
        ).map_err(|e| e.to_string())?;
    }

    Ok(content)
}
