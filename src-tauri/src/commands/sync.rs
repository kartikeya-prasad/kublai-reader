use crate::db::{AppDatabase, models::SyncAccount};
use crate::sync::{SyncResult, greader::GoogleReaderClient};
use tauri::State;

#[tauri::command]
pub async fn get_sync_accounts(db: State<'_, AppDatabase>) -> Result<Vec<SyncAccount>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, provider, server_url, username, auth_token, last_synced, created_at FROM sync_accounts"
    ).map_err(|e| e.to_string())?;
    let accounts = stmt.query_map([], |row| {
        Ok(SyncAccount {
            id: row.get(0)?,
            provider: row.get(1)?,
            server_url: row.get(2)?,
            username: row.get(3)?,
            auth_token: row.get(4)?,
            last_synced: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(accounts)
}

#[tauri::command]
pub async fn add_sync_account(
    provider: String,
    server_url: String,
    username: String,
    password: String,
    db: State<'_, AppDatabase>,
) -> Result<SyncAccount, String> {
    // Test login first
    let mut client = GoogleReaderClient::new(server_url.clone(), username.clone());
    client.login(&password).await?;
    let token = client.auth_token.clone();

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO sync_accounts (provider, server_url, username, auth_token) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![provider, server_url, username, token],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, provider, server_url, username, auth_token, last_synced, created_at FROM sync_accounts WHERE id = ?1",
        [id],
        |row| Ok(SyncAccount {
            id: row.get(0)?,
            provider: row.get(1)?,
            server_url: row.get(2)?,
            username: row.get(3)?,
            auth_token: row.get(4)?,
            last_synced: row.get(5)?,
            created_at: row.get(6)?,
        }),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_sync_account(account_id: i64, db: State<'_, AppDatabase>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM sync_accounts WHERE id = ?1", [account_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn sync_now(account_id: i64, db: State<'_, AppDatabase>) -> Result<SyncResult, String> {
    let (provider, server_url, username, auth_token) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT provider, server_url, username, auth_token FROM sync_accounts WHERE id = ?1",
            [account_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            )),
        ).map_err(|e| e.to_string())?
    };

    let mut client = GoogleReaderClient::new(server_url, username);
    client.auth_token = auth_token;

    let mut result = SyncResult { feeds_synced: 0, entries_synced: 0, errors: vec![] };

    // Get subscriptions and sync feeds
    match client.get_subscriptions().await {
        Ok(remote_feeds) => {
            result.feeds_synced = remote_feeds.len();
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            for feed in &remote_feeds {
                // Check if we already have this feed by URL
                let local: Option<i64> = conn.query_row(
                    "SELECT id FROM feeds WHERE url = ?1",
                    [&feed.url],
                    |row| row.get(0),
                ).ok();
                if let Some(lid) = local {
                    // Update sync_id
                    let _ = conn.execute(
                        "UPDATE feeds SET sync_id = ?1 WHERE id = ?2",
                        rusqlite::params![feed.id, lid],
                    );
                }
            }
        }
        Err(e) => result.errors.push(format!("Subscription sync failed: {}", e)),
    }

    // Get unread IDs and sync read state
    match client.get_unread_ids().await {
        Ok(unread_ids) => {
            result.entries_synced = unread_ids.len();
            // Mark articles as unread if they appear in remote unread list
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            for id in &unread_ids {
                let _ = conn.execute(
                    "UPDATE articles SET is_read = 0 WHERE sync_id = ?1",
                    [id],
                );
            }
        }
        Err(e) => result.errors.push(format!("Unread sync failed: {}", e)),
    }

    // Update last_synced
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let _ = conn.execute(
        "UPDATE sync_accounts SET last_synced = datetime('now') WHERE id = ?1",
        [account_id],
    );

    let _ = provider; // suppress unused warning
    Ok(result)
}
