use crate::db::AppDatabase;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

pub async fn fetch_and_store_favicon(
    site_url: &str,
    feed_id: i64,
    db: &AppDatabase,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("Mozilla/5.0 KublaiReader/0.2")
        .build()?;

    let icon_url = discover_favicon_url(&client, site_url).await;

    if let Some(url) = icon_url {
        let resp = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            _ => return Ok(()),
        };
        let bytes = resp.bytes().await?;
        if bytes.is_empty() || bytes.len() > 256_000 {
            return Ok(());
        }

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());

        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM favicons WHERE hash = ?1",
            [&hash],
            |row| row.get(0),
        ).ok();

        let favicon_id = if let Some(id) = existing {
            id
        } else {
            conn.execute(
                "INSERT OR IGNORE INTO favicons (hash, data, mime_type) VALUES (?1, ?2, 'image/png')",
                rusqlite::params![hash, bytes.as_ref()],
            )?;
            conn.last_insert_rowid()
        };

        conn.execute(
            "UPDATE feeds SET favicon_id = ?1 WHERE id = ?2",
            rusqlite::params![favicon_id, feed_id],
        )?;
    }
    Ok(())
}

async fn discover_favicon_url(client: &reqwest::Client, site_url: &str) -> Option<String> {
    // Try HTML parsing first
    if let Ok(resp) = client.get(site_url).send().await {
        if resp.status().is_success() {
            if let Ok(html) = resp.text().await {
                if let Some(icon) = extract_icon_from_html(&html, site_url) {
                    return Some(icon);
                }
            }
        }
    }
    // Fallback: /favicon.ico
    if let Ok(base) = url::Url::parse(site_url) {
        let favicon_url = format!("{}://{}/favicon.ico",
            base.scheme(),
            base.host_str().unwrap_or(""));
        if let Ok(resp) = client.head(&favicon_url).send().await {
            if resp.status().is_success() {
                return Some(favicon_url);
            }
        }
    }
    None
}

fn extract_icon_from_html(html: &str, base_url: &str) -> Option<String> {
    let html_lower = html.to_lowercase();
    let patterns = ["rel=\"icon\"", "rel=\"shortcut icon\"", "rel=\"apple-touch-icon\""];
    for pattern in &patterns {
        if let Some(pos) = html_lower.find(pattern) {
            let start = html_lower[..pos].rfind('<').unwrap_or(0);
            let end = html_lower[pos..].find('>').map(|e| pos + e + 1).unwrap_or(html_lower.len());
            let tag = &html[start..end];
            if let Some(href) = extract_attr(tag, "href") {
                if let Ok(base) = url::Url::parse(base_url) {
                    if let Ok(resolved) = base.join(&href) {
                        return Some(resolved.to_string());
                    }
                }
                if href.starts_with("http") { return Some(href); }
            }
        }
    }
    None
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let search = format!("{}={}", attr, quote);
        if let Some(start) = tag.to_lowercase().find(&search) {
            let vs = start + search.len();
            if let Some(end) = tag[vs..].find(quote) {
                return Some(tag[vs..vs+end].to_string());
            }
        }
    }
    None
}

pub fn get_favicon_data_uri(feed_id: i64, db: &AppDatabase) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let result: Option<(Vec<u8>, String)> = conn.query_row(
        "SELECT fav.data, fav.mime_type FROM favicons fav
         JOIN feeds f ON f.favicon_id = fav.id WHERE f.id = ?1",
        [feed_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).ok();
    Ok(result.map(|(data, mime)| {
        format!("data:{};base64,{}", mime, general_purpose::STANDARD.encode(&data))
    }))
}
