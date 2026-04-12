use crate::db::models::{Article, ArticleListItem, ArticlePage};
use crate::db::AppDatabase;
use tauri::State;

#[tauri::command]
pub async fn get_articles(
    feed_id: Option<i64>,
    folder_id: Option<i64>,
    filter: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
    db: State<'_, AppDatabase>,
) -> Result<ArticlePage, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(50);
    let offset = page * page_size;

    let mut where_clauses = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(fid) = feed_id {
        where_clauses.push("a.feed_id = ?".to_string());
        params.push(Box::new(fid));
    }

    if let Some(foid) = folder_id {
        where_clauses.push("f.folder_id = ?".to_string());
        params.push(Box::new(foid));
    }

    match filter.as_deref() {
        Some("unread") => where_clauses.push("a.is_read = 0".to_string()),
        Some("starred") => where_clauses.push("a.is_starred = 1".to_string()),
        Some("read_later") => where_clauses.push("a.is_read_later = 1".to_string()),
        _ => {}
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Get total count
    let count_sql = format!(
        "SELECT COUNT(*) FROM articles a JOIN feeds f ON a.feed_id = f.id {}",
        where_sql
    );
    let total: i64 = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    // Get articles
    let query = format!(
        "SELECT a.id, a.feed_id, f.title, f.icon_url, a.title, a.url, a.author,
                a.summary, a.thumbnail_url, a.published_at, a.is_read, a.is_starred,
                a.is_read_later, (a.parsed_content IS NOT NULL) as has_parsed
         FROM articles a
         JOIN feeds f ON a.feed_id = f.id
         {}
         ORDER BY a.published_at DESC, a.id DESC
         LIMIT ? OFFSET ?",
        where_sql
    );

    params.push(Box::new(page_size));
    params.push(Box::new(offset));

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let articles: Vec<ArticleListItem> = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| {
            let article_id: i64 = row.get(0)?;
            Ok((article_id, ArticleListItem {
                id: article_id,
                feed_id: row.get(1)?,
                feed_title: row.get(2)?,
                feed_icon_url: row.get(3)?,
                title: row.get(4)?,
                url: row.get(5)?,
                author: row.get(6)?,
                summary: row.get(7)?,
                thumbnail_url: row.get(8)?,
                published_at: row.get(9)?,
                is_read: row.get::<_, i64>(10)? != 0,
                is_starred: row.get::<_, i64>(11)? != 0,
                is_read_later: row.get::<_, i64>(12)? != 0,
                has_parsed_content: row.get::<_, i64>(13)? != 0,
                tag_ids: vec![],
            }))
        },
    ).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .map(|(id, mut item)| {
        // Get tag IDs for this article
        if let Ok(mut tag_stmt) = conn.prepare("SELECT tag_id FROM article_tags WHERE article_id = ?1") {
            if let Ok(tags) = tag_stmt.query_map([id], |row| row.get::<_, i64>(0)) {
                item.tag_ids = tags.filter_map(|r| r.ok()).collect();
            }
        }
        item
    })
    .collect();

    Ok(ArticlePage {
        articles,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub async fn get_article(article_id: i64, db: State<'_, AppDatabase>) -> Result<Article, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, feed_id, guid, title, url, author, summary, content, parsed_content,
                thumbnail_url, published_at, is_read, is_starred, is_read_later, sync_id,
                content_cached_at, ai_summary, created_at
         FROM articles WHERE id = ?1",
        [article_id],
        |row| {
            Ok(Article {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                guid: row.get(2)?,
                title: row.get(3)?,
                url: row.get(4)?,
                author: row.get(5)?,
                summary: row.get(6)?,
                content: row.get(7)?,
                parsed_content: row.get(8)?,
                thumbnail_url: row.get(9)?,
                published_at: row.get(10)?,
                is_read: row.get::<_, i64>(11)? != 0,
                is_starred: row.get::<_, i64>(12)? != 0,
                is_read_later: row.get::<_, i64>(13)? != 0,
                sync_id: row.get(14)?,
                content_cached_at: row.get(15)?,
                ai_summary: row.get(16)?,
                created_at: row.get(17)?,
            })
        },
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_read(article_id: i64, db: State<'_, AppDatabase>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE articles SET is_read = 1 WHERE id = ?1", [article_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn mark_unread(article_id: i64, db: State<'_, AppDatabase>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE articles SET is_read = 0 WHERE id = ?1", [article_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn mark_all_read(
    feed_id: Option<i64>,
    folder_id: Option<i64>,
    db: State<'_, AppDatabase>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    if let Some(fid) = feed_id {
        conn.execute("UPDATE articles SET is_read = 1 WHERE feed_id = ?1", [fid])
            .map_err(|e| e.to_string())?;
    } else if let Some(foid) = folder_id {
        conn.execute(
            "UPDATE articles SET is_read = 1 WHERE feed_id IN (SELECT id FROM feeds WHERE folder_id = ?1)",
            [foid],
        ).map_err(|e| e.to_string())?;
    } else {
        conn.execute("UPDATE articles SET is_read = 1", [])
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn mark_above_read(
    feed_id: i64,
    article_id: i64,
    db: State<'_, AppDatabase>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Get the published_at of the reference article
    let published_at: Option<String> = conn.query_row(
        "SELECT published_at FROM articles WHERE id = ?1",
        [article_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if let Some(pub_date) = published_at {
        conn.execute(
            "UPDATE articles SET is_read = 1 WHERE feed_id = ?1 AND published_at >= ?2 AND id != ?3",
            rusqlite::params![feed_id, pub_date, article_id],
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_star(article_id: i64, db: State<'_, AppDatabase>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE articles SET is_starred = CASE WHEN is_starred = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        [article_id],
    ).map_err(|e| e.to_string())?;

    let is_starred: bool = conn.query_row(
        "SELECT is_starred FROM articles WHERE id = ?1",
        [article_id],
        |row| Ok(row.get::<_, i64>(0)? != 0),
    ).map_err(|e| e.to_string())?;

    Ok(is_starred)
}

#[tauri::command]
pub async fn toggle_read_later(article_id: i64, db: State<'_, AppDatabase>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE articles SET is_read_later = CASE WHEN is_read_later = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        [article_id],
    ).map_err(|e| e.to_string())?;

    let is_read_later: bool = conn.query_row(
        "SELECT is_read_later FROM articles WHERE id = ?1",
        [article_id],
        |row| Ok(row.get::<_, i64>(0)? != 0),
    ).map_err(|e| e.to_string())?;

    Ok(is_read_later)
}

#[tauri::command]
pub async fn search_articles(
    query: String,
    feed_id: Option<i64>,
    folder_id: Option<i64>,
    filter: Option<String>,
    db: State<'_, AppDatabase>,
) -> Result<Vec<ArticleListItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut where_clauses = vec!["articles_fts MATCH ?1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(query)];

    if let Some(fid) = feed_id {
        where_clauses.push("a.feed_id = ?".to_string());
        params.push(Box::new(fid));
    }

    if let Some(foid) = folder_id {
        where_clauses.push("f.folder_id = ?".to_string());
        params.push(Box::new(foid));
    }

    match filter.as_deref() {
        Some("unread") => where_clauses.push("a.is_read = 0".to_string()),
        Some("starred") => where_clauses.push("a.is_starred = 1".to_string()),
        Some("read_later") => where_clauses.push("a.is_read_later = 1".to_string()),
        _ => {}
    }

    let sql = format!(
        "SELECT a.id, a.feed_id, f.title, f.icon_url, a.title, a.url, a.author,
                a.summary, a.thumbnail_url, a.published_at, a.is_read, a.is_starred,
                a.is_read_later, (a.parsed_content IS NOT NULL) as has_parsed
         FROM articles_fts
         JOIN articles a ON a.id = articles_fts.rowid
         JOIN feeds f ON a.feed_id = f.id
         WHERE {}
         ORDER BY bm25(articles_fts)
         LIMIT 100",
        where_clauses.join(" AND ")
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let articles: Vec<ArticleListItem> = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| {
            Ok(ArticleListItem {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                feed_title: row.get(2)?,
                feed_icon_url: row.get(3)?,
                title: row.get(4)?,
                url: row.get(5)?,
                author: row.get(6)?,
                summary: row.get(7)?,
                thumbnail_url: row.get(8)?,
                published_at: row.get(9)?,
                is_read: row.get::<_, i64>(10)? != 0,
                is_starred: row.get::<_, i64>(11)? != 0,
                is_read_later: row.get::<_, i64>(12)? != 0,
                has_parsed_content: row.get::<_, i64>(13)? != 0,
                tag_ids: vec![],
            })
        },
    ).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(articles)
}
