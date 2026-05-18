/// 分时历史数据仓库
use crate::models::stock::StockCode;
use crate::models::timeshare::TimesharePoint;
use rusqlite::{params, Connection};

/// 保存分时数据点
pub fn save_timeshare_point(conn: &Connection, point: &TimesharePoint) -> anyhow::Result<()> {
    let code = point.code.for_api();
    conn.execute(
        "INSERT INTO timeshare (code, trade_time, price, volume, avg_price, cum_volume)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            code,
            point.trade_time,
            point.price.to_string(),
            point.volume,
            point.avg_price.map(|d| d.to_string()),
            point.cum_volume,
        ],
    )?;
    Ok(())
}