use rusqlite::Connection;

const CURRENT_VERSION: i32 = 2;

pub fn run_migrations(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 1 {
        migrate_v1(conn)?;
    }

    if version < 2 {
        migrate_v2(conn)?;
    }

    conn.pragma_update(None, "user_version", CURRENT_VERSION)?;
    Ok(())
}

fn migrate_v2(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS favicons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hash TEXT NOT NULL UNIQUE,
            data BLOB NOT NULL,
            mime_type TEXT NOT NULL DEFAULT 'image/png',
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS sync_accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            server_url TEXT NOT NULL,
            username TEXT NOT NULL,
            auth_token TEXT,
            last_synced TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );"
    )?;

    // SQLite requires separate ALTER TABLE statements
    conn.execute("ALTER TABLE feeds ADD COLUMN favicon_id INTEGER", [])?;
    conn.execute("ALTER TABLE feeds ADD COLUMN sync_id TEXT", [])?;
    conn.execute("ALTER TABLE articles ADD COLUMN sync_id TEXT", [])?;
    conn.execute("ALTER TABLE articles ADD COLUMN content_cached_at TEXT", [])?;

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_feeds_sync ON feeds(sync_id);
         CREATE INDEX IF NOT EXISTS idx_articles_sync ON articles(sync_id);

         INSERT OR IGNORE INTO settings (key, value) VALUES ('cache_max_days', '60');
         INSERT OR IGNORE INTO settings (key, value) VALUES ('cache_max_per_feed', '500');"
    )?;

    Ok(())
}

fn migrate_v1(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            parent_id INTEGER REFERENCES folders(id) ON DELETE CASCADE,
            position INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER REFERENCES folders(id) ON DELETE SET NULL,
            title TEXT NOT NULL,
            url TEXT NOT NULL UNIQUE,
            site_url TEXT,
            description TEXT,
            icon_url TEXT,
            auto_parse INTEGER NOT NULL DEFAULT 0,
            update_interval INTEGER NOT NULL DEFAULT 3600,
            etag TEXT,
            last_modified TEXT,
            last_fetched TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
            guid TEXT NOT NULL,
            title TEXT NOT NULL,
            url TEXT,
            author TEXT,
            summary TEXT,
            content TEXT,
            parsed_content TEXT,
            thumbnail_url TEXT,
            published_at TEXT,
            is_read INTEGER NOT NULL DEFAULT 0,
            is_starred INTEGER NOT NULL DEFAULT 0,
            is_read_later INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(feed_id, guid)
        );

        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#6366f1',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS article_tags (
            article_id INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
            tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (article_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- Performance indexes
        CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id);
        CREATE INDEX IF NOT EXISTS idx_articles_feed_read_date ON articles(feed_id, is_read, published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_articles_starred ON articles(is_starred) WHERE is_starred = 1;
        CREATE INDEX IF NOT EXISTS idx_articles_read_later ON articles(is_read_later) WHERE is_read_later = 1;
        CREATE INDEX IF NOT EXISTS idx_articles_unread ON articles(is_read) WHERE is_read = 0;
        CREATE INDEX IF NOT EXISTS idx_feeds_folder ON feeds(folder_id);
        CREATE INDEX IF NOT EXISTS idx_article_tags_article ON article_tags(article_id);
        CREATE INDEX IF NOT EXISTS idx_article_tags_tag ON article_tags(tag_id);

        -- Full-text search
        CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
            title,
            summary,
            content,
            parsed_content,
            content=articles,
            content_rowid=id
        );

        CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
            INSERT INTO articles_fts(rowid, title, summary, content, parsed_content)
            VALUES (new.id, new.title, new.summary, new.content, new.parsed_content);
        END;

        CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
            INSERT INTO articles_fts(articles_fts, rowid, title, summary, content, parsed_content)
            VALUES ('delete', old.id, old.title, old.summary, old.content, old.parsed_content);
        END;

        CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
            INSERT INTO articles_fts(articles_fts, rowid, title, summary, content, parsed_content)
            VALUES ('delete', old.id, old.title, old.summary, old.content, old.parsed_content);
            INSERT INTO articles_fts(rowid, title, summary, content, parsed_content)
            VALUES (new.id, new.title, new.summary, new.content, new.parsed_content);
        END;

        -- Default settings
        INSERT OR IGNORE INTO settings (key, value) VALUES ('theme_mode', 'auto');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('font_family', 'system-ui');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('font_size', '16');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('line_height', '1.6');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('content_width', '680');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('article_view', 'card');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_mark_read', 'true');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('sidebar_width', '250');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('article_list_width', '360');"
    )?;

    Ok(())
}
