use core::slice;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::{error::AppError, item::Item};

pub type DbIndex = Arc<Mutex<Connection>>;

/// Opens in-memory append-only index of items.
pub fn open() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()
        .map_err(|e| AppError::DbError(format!("Failed to open index db: {}", e)))?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            path        TEXT PRIMARY KEY,
            kind        TEXT NOT NULL,
            title       TEXT NOT NULL,
            url         TEXT,
            author      TEXT,
            status      TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            updated_at  TEXT
        );
        CREATE TABLE IF NOT EXISTS tags (
            path TEXT NOT NULL,
            tag  TEXT NOT NULL,
            PRIMARY KEY (path, tag)
        );
        CREATE INDEX IF NOT EXISTS idx_items_kind ON items(kind);
        CREATE INDEX IF NOT EXISTS idx_items_status ON items(status);
        CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at);
        CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
        "#,
    )
    .map_err(|e| AppError::DbError(format!("Failed to init index schema: {}", e)))
}

pub fn add_item(db: &DbIndex, item: &Item) -> Result<(), AppError> {
    add_items(db, slice::from_ref(item))
}

pub fn add_items(db: &DbIndex, items: &[Item]) -> Result<(), AppError> {
    let mut conn = db
        .lock()
        .map_err(|e| AppError::DbError(format!("db index lock: {}", e)))?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DbError(format!("db index transaction: {}", e)))?;
    for item in items {
        tx.execute(
            "INSERT INTO items (path, kind, title, url, author, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(path) DO UPDATE SET
            kind=excluded.kind, title=excluded.title, url=excluded.url,
            author=excluded.author, status=excluded.status,
            created_at=excluded.created_at, updated_at=excluded.updated_at",
            rusqlite::params![
                item.path,
                item.kind.to_string(),
                item.title,
                item.url,
                item.author,
                item.status.to_string(),
                item.created_at,
                item.updated_at,
            ],
        )
        .map_err(|e| AppError::DbError(format!("db index add item {}: {}", item.path, e)))?;

        tx.execute(
            "DELETE FROM tags WHERE path = ?1",
            rusqlite::params![item.path],
        )
        .map_err(|e| AppError::DbError(format!("db index delete tags {}: {}", item.path, e)))?;

        for tag in &item.tags {
            tx.execute(
                "INSERT INTO tags (path, tag) VALUES (?1, ?2)",
                rusqlite::params![item.path, tag],
            )
            .map_err(|e| AppError::DbError(format!("db index add tag {}: {}", tag, e)))?;
        }
    }
    tx.commit()
        .map_err(|e| AppError::DbError(format!("db index commit: {}", e)))?;
    Ok(())
}

pub fn load_items(db: &DbIndex) -> Result<Vec<Item>, AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::DbError(format!("db index lock: {}", e)))?;

    let mut stmt = conn
        .prepare("SELECT path, kind, title, url, author, status, created_at, updated_at FROM items")
        .map_err(|e| AppError::DbError(format!("db index prepare items: {}", e)))?;
    let items = stmt
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let kind: String = row.get(1)?;
            let status: String = row.get(5)?;
            let tags = load_tags(&conn, &path).unwrap_or_default();
            Ok(Item {
                path,
                kind: kind.as_str().into(),
                title: row.get(2)?,
                url: row.get(3)?,
                author: row.get(4)?,
                status: status.as_str().into(),
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                tags,
            })
        })
        .map_err(|e| AppError::DbError(format!("db index query items: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DbError(format!("db index items row: {}", e)))?;
    Ok(items)
}

fn load_tags(conn: &Connection, path: &str) -> Result<Vec<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT tag FROM tags WHERE path = ?1 ORDER BY tag")
        .map_err(|e| AppError::DbError(format!("db index prepare tags: {}", e)))?;
    let tags = stmt
        .query_map([path], |row| row.get::<_, String>(0))
        .map_err(|e| AppError::DbError(format!("db index query tags: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DbError(format!("db index tags row: {}", e)))?;
    Ok(tags)
}
