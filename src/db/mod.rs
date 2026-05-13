/// 数据库模块 - SQLite本地缓存
pub mod schema;
pub mod bar_repo;
pub mod trade_repo;
pub mod position_repo;
pub mod watch_repo;
pub mod alert_repo;
pub mod tick_repo;
pub mod orderbook_repo;
pub mod moneyflow_repo;
pub mod quote_snapshot_repo;

use rusqlite::Connection;
use std::path::PathBuf;

/// 获取数据库文件路径 (~/.astock/astock.db)
fn db_path() -> PathBuf {
    let home = dirs_home();
    let dir = home.join(".astock");
    std::fs::create_dir_all(&dir).ok();
    dir.join("astock.db")
}

fn dirs_home() -> PathBuf {
    // 简单获取home目录
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

/// 打开数据库连接并初始化表结构
pub fn open_db() -> anyhow::Result<Connection> {
    let path = db_path();
    let conn = Connection::open(path)?;
    conn.execute_batch(&schema::CREATE_TABLES)?;
    Ok(conn)
}
