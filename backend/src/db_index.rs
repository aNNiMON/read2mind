use core::slice;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rusqlite::{Connection, ToSql, params};

use crate::{error::AppError, model::item::Item};

pub type DbIndex = Arc<Mutex<Connection>>;

#[derive(Debug, Clone, Default)]
pub struct ItemsFilter {
    pub kind: Option<String>,
    pub status: Option<String>,
    pub date: Option<String>,
    pub keyword: Option<String>,
    pub author: Option<String>,
    pub include_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub limit: u32,
    pub offset: u32,
}

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
        CREATE INDEX IF NOT EXISTS idx_items_title ON items(title);
        CREATE INDEX IF NOT EXISTS idx_items_author ON items(author);
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
            params![
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

        tx.execute("DELETE FROM tags WHERE path = ?1", params![item.path])
            .map_err(|e| AppError::DbError(format!("db index delete tags {}: {}", item.path, e)))?;

        for tag in &item.tags {
            tx.execute(
                "INSERT INTO tags (path, tag) VALUES (?1, ?2)",
                params![item.path, tag],
            )
            .map_err(|e| AppError::DbError(format!("db index add tag {}: {}", tag, e)))?;
        }
    }
    tx.commit()
        .map_err(|e| AppError::DbError(format!("db index commit: {}", e)))?;
    Ok(())
}

pub fn load_items(db: &DbIndex, filter: &ItemsFilter) -> Result<(Vec<Item>, usize), AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::DbError(format!("db index lock: {}", e)))?;

    let mut filter_sql = " WHERE 1=1".to_owned();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(kind) = &filter.kind {
        filter_sql.push_str(" AND kind = ?");
        params.push(Box::new(kind));
    }
    if let Some(status) = &filter.status {
        filter_sql.push_str(" AND status = ?");
        params.push(Box::new(status));
    }
    if let Some(date) = &filter.date {
        filter_sql.push_str(" AND date(created_at) = ?");
        params.push(Box::new(date));
    }
    if let Some(keyword) = &filter.keyword {
        filter_sql.push_str(" AND title LIKE ?");
        params.push(Box::new(format!("%{}%", keyword)));
    }
    if let Some(author) = &filter.author {
        filter_sql.push_str(" AND author = ?");
        params.push(Box::new(author));
    }

    if !filter.include_tags.is_empty() {
        let placeholders = std::iter::repeat_n("?", filter.include_tags.len())
            .collect::<Vec<_>>()
            .join(", ");
        filter_sql.push_str(&format!(
            " AND path IN (SELECT path FROM tags t WHERE tag IN ({}))",
            placeholders,
        ));
        params.extend(
            filter
                .include_tags
                .iter()
                .map(|t| Box::new(t) as Box<dyn ToSql>),
        );
    }

    if !filter.exclude_tags.is_empty() {
        let placeholders = std::iter::repeat_n("?", filter.exclude_tags.len())
            .collect::<Vec<_>>()
            .join(", ");
        filter_sql.push_str(&format!(
            " AND path NOT IN (SELECT path FROM tags t WHERE tag IN ({}))",
            placeholders,
        ));
        params.extend(
            filter
                .exclude_tags
                .iter()
                .map(|t| Box::new(t) as Box<dyn ToSql>),
        );
    }

    let count_sql = format!("SELECT COUNT(*) FROM items {}", filter_sql);
    let count_params: Vec<&dyn ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let total: i32 = conn
        .query_row(&count_sql, &count_params[..], |row| row.get(0))
        .map_err(|e| AppError::DbError(format!("db index count items: {}", e)))?;
    if total <= 0 {
        return Ok((Vec::new(), 0));
    }

    let items_sql = format!(
        r#"
            SELECT path, kind, title, url, author, status, created_at, updated_at
            FROM items {}
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
        "#,
        filter_sql
    );
    params.push(Box::new(filter.limit));
    params.push(Box::new(filter.offset));

    let mut stmt = conn
        .prepare(&items_sql)
        .map_err(|e| AppError::DbError(format!("db index prepare items: {}", e)))?;
    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let items = stmt
        .query_map(param_refs.as_slice(), |row| {
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
    Ok((items, total as usize))
}

pub fn delete_item(db: &DbIndex, path: &str) -> Result<(), AppError> {
    let mut conn = db
        .lock()
        .map_err(|e| AppError::DbError(format!("db index lock: {}", e)))?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DbError(format!("db index transaction: {}", e)))?;
    tx.execute("DELETE FROM tags WHERE path = ?1", params![path])
        .map_err(|e| AppError::DbError(format!("db index delete tags {}: {}", path, e)))?;
    tx.execute("DELETE FROM items WHERE path = ?1", params![path])
        .map_err(|e| AppError::DbError(format!("db index delete item {}: {}", path, e)))?;
    tx.commit()
        .map_err(|e| AppError::DbError(format!("db index commit: {}", e)))?;
    Ok(())
}

pub fn load_tags_by_freq(db: &DbIndex) -> Result<HashMap<String, usize>, AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::DbError(format!("db index lock: {}", e)))?;
    let mut stmt = conn
        .prepare("SELECT tag, count(*) FROM tags GROUP BY tag")
        .map_err(|e| AppError::DbError(format!("db index prepare tags: {}", e)))?;
    let tags = stmt
        .query_map([], |row| {
            let tag: String = row.get(0)?;
            let count: isize = row.get(1)?;
            Ok((tag, count as usize))
        })
        .map_err(|e| AppError::DbError(format!("db index query tags: {}", e)))?
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(|e| AppError::DbError(format!("db index tags row: {}", e)))?;
    Ok(tags)
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
