use crate::db::models::Folder;
use crate::db::AppDatabase;
use tauri::State;

#[tauri::command]
pub async fn create_folder(
    name: String,
    parent_id: Option<i64>,
    db: State<'_, AppDatabase>,
) -> Result<Folder, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM folders WHERE parent_id IS ?1",
        [parent_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO folders (name, parent_id, position) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, parent_id, position],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, name, parent_id, position, created_at FROM folders WHERE id = ?1",
        [id],
        |row| Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            position: row.get(3)?,
            created_at: row.get(4)?,
        }),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_folder(folder_id: i64, db: State<'_, AppDatabase>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    // Move feeds in this folder to uncategorized before deleting
    conn.execute("UPDATE feeds SET folder_id = NULL WHERE folder_id = ?1", [folder_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM folders WHERE id = ?1", [folder_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn rename_folder(
    folder_id: i64,
    name: String,
    db: State<'_, AppDatabase>,
) -> Result<Folder, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE folders SET name = ?1 WHERE id = ?2", rusqlite::params![name, folder_id])
        .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, parent_id, position, created_at FROM folders WHERE id = ?1",
        [folder_id],
        |row| Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            position: row.get(3)?,
            created_at: row.get(4)?,
        }),
    ).map_err(|e| e.to_string())
}
