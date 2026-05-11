/// 自选股管理
use crate::models::stock::StockCode;
use rusqlite::{params, Connection};

/// 自选股记录
#[derive(Debug, Clone)]
pub struct WatchItem {
    pub code: String,
    pub name: String,
}

/// 添加自选股
pub fn add_watch(conn: &Connection, code: &StockCode, name: &str) -> anyhow::Result<bool> {
    let affected = conn.execute(
        "INSERT OR REPLACE INTO watchlist (code, name) VALUES (?1, ?2)",
        params![code.for_api(), name],
    )?;
    Ok(affected > 0)
}

/// 删除自选股
pub fn remove_watch(conn: &Connection, code: &StockCode) -> anyhow::Result<bool> {
    let affected = conn.execute("DELETE FROM watchlist WHERE code = ?1", params![code.for_api()])?;
    Ok(affected > 0)
}

/// 查询所有自选股
pub fn list_watch(conn: &Connection) -> anyhow::Result<Vec<WatchItem>> {
    let mut stmt = conn.prepare("SELECT code, name FROM watchlist ORDER BY code")?;
    let rows = stmt.query_map([], |row| {
        Ok(WatchItem {
            code: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}
