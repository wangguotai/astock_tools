/// K线数据仓库 - 本地SQLite缓存
use crate::models::bar::Bar;
use crate::models::stock::StockCode;
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use std::str::FromStr;

/// 批量保存K线数据到数据库
pub fn save_bars(conn: &Connection, bars: &[Bar]) -> anyhow::Result<usize> {
    let mut count = 0;
    let tx = conn.unchecked_transaction()?;

    for bar in bars {
        let code = bar.code.for_api();
        let date = bar.date.to_string();
        let open = bar.open.to_string();
        let high = bar.high.to_string();
        let low = bar.low.to_string();
        let close = bar.close.to_string();
        let volume = bar.volume;
        let turnover = bar.turnover.to_string();

        tx.execute(
            "INSERT OR REPLACE INTO daily_bars (code, date, open, high, low, close, volume, turnover)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![code, date, open, high, low, close, volume, turnover],
        )?;
        count += 1;
    }

    tx.commit()?;
    Ok(count)
}

/// 从数据库查询K线数据
pub fn query_bars(
    conn: &Connection,
    code: &StockCode,
    start: Option<&NaiveDate>,
    end: Option<&NaiveDate>,
) -> anyhow::Result<Vec<Bar>> {
    let code_str = code.for_api();
    let mut sql = String::from("SELECT code, date, open, high, low, close, volume, turnover FROM daily_bars WHERE code = ?1");
    let mut param_idx = 2;

    if start.is_some() {
        sql.push_str(&format!(" AND date >= ?{}", param_idx));
        param_idx += 1;
    }
    if end.is_some() {
        sql.push_str(&format!(" AND date <= ?{}", param_idx));
    }
    sql.push_str(" ORDER BY date ASC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![code_str], |row| {
        let code_str: String = row.get(0)?;
        let date_str: String = row.get(1)?;
        let open_str: String = row.get(2)?;
        let high_str: String = row.get(3)?;
        let low_str: String = row.get(4)?;
        let close_str: String = row.get(5)?;
        let volume: i64 = row.get(6)?;
        let turnover_str: String = row.get(7)?;

        Ok((code_str, date_str, open_str, high_str, low_str, close_str, volume, turnover_str))
    })?;

    let mut bars = Vec::new();
    for row in rows {
        let (c, d, o, h, l, cl, v, t) = row?;
        let code = StockCode::from_raw(&c).unwrap_or_else(|_| code.clone());
        let date = NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok();
        if let Some(date) = date {
            bars.push(Bar {
                code,
                date,
                open: Decimal::from_str(&o).unwrap_or(Decimal::ZERO),
                high: Decimal::from_str(&h).unwrap_or(Decimal::ZERO),
                low: Decimal::from_str(&l).unwrap_or(Decimal::ZERO),
                close: Decimal::from_str(&cl).unwrap_or(Decimal::ZERO),
                volume: v,
                turnover: Decimal::from_str(&t).unwrap_or(Decimal::ZERO),
            });
        }
    }

    Ok(bars)
}

/// 获取数据库中最新的K线日期
pub fn get_latest_date(conn: &Connection, code: &StockCode) -> anyhow::Result<Option<String>> {
    let code_str = code.for_api();
    let mut stmt = conn.prepare("SELECT MAX(date) FROM daily_bars WHERE code = ?1")?;
    let result = stmt.query_row(params![code_str], |row| row.get::<_, Option<String>>(0))?;
    Ok(result)
}
