/// 实时行情快照仓库
use crate::models::quote::Quote;
use crate::models::stock::StockCode;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use std::str::FromStr;

/// 保存行情快照
pub fn save_quote_snapshot(conn: &Connection, quote: &Quote, source: &str) -> anyhow::Result<()> {
    let code = quote.code.for_api();
    conn.execute(
        "INSERT INTO quote_snapshots (code, name, price, prev_close, open, high, low, volume, turnover, bid, ask, bid_vol, ask_vol, change_val, change_pct, high_limit, low_limit, inner_vol, outer_vol, open_vol, quote_time, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            code,
            quote.name,
            quote.price.to_string(),
            quote.prev_close.to_string(),
            quote.open.to_string(),
            quote.high.to_string(),
            quote.low.to_string(),
            quote.volume,
            quote.turnover.to_string(),
            quote.bid.to_string(),
            quote.ask.to_string(),
            quote.bid_vol,
            quote.ask_vol,
            quote.change.to_string(),
            quote.change_pct.to_string(),
            quote.high_limit.map(|d| d.to_string()),
            quote.low_limit.map(|d| d.to_string()),
            quote.inner_vol,
            quote.outer_vol,
            quote.open_vol,
            quote.time,
            source,
        ],
    )?;
    Ok(())
}

/// 查询最新行情快照
pub fn get_latest_snapshot(
    conn: &Connection,
    code: &StockCode,
) -> anyhow::Result<Option<Quote>> {
    let code_str = code.for_api();
    let mut stmt = conn.prepare(
        "SELECT code, name, price, prev_close, open, high, low, volume, turnover, bid, ask, bid_vol, ask_vol, change_val, change_pct, high_limit, low_limit, inner_vol, outer_vol, open_vol, quote_time
         FROM quote_snapshots WHERE code = ?1 ORDER BY created_at DESC LIMIT 1"
    )?;

    let result = stmt.query_row(params![code_str], |row| {
        Ok(Quote {
            code: StockCode::from_raw(&row.get::<_, String>(0)?).unwrap_or_else(|_| code.clone()),
            name: row.get(1)?,
            price: Decimal::from_str(&row.get::<_, String>(2)?).unwrap_or(Decimal::ZERO),
            prev_close: Decimal::from_str(&row.get::<_, String>(3)?).unwrap_or(Decimal::ZERO),
            open: Decimal::from_str(&row.get::<_, String>(4)?).unwrap_or(Decimal::ZERO),
            high: Decimal::from_str(&row.get::<_, String>(5)?).unwrap_or(Decimal::ZERO),
            low: Decimal::from_str(&row.get::<_, String>(6)?).unwrap_or(Decimal::ZERO),
            volume: row.get(7)?,
            turnover: Decimal::from_str(&row.get::<_, String>(8)?).unwrap_or(Decimal::ZERO),
            bid: Decimal::from_str(&row.get::<_, String>(9)?).unwrap_or(Decimal::ZERO),
            ask: Decimal::from_str(&row.get::<_, String>(10)?).unwrap_or(Decimal::ZERO),
            bid_vol: row.get(11)?,
            ask_vol: row.get(12)?,
            change: Decimal::from_str(&row.get::<_, String>(13)?).unwrap_or(Decimal::ZERO),
            change_pct: Decimal::from_str(&row.get::<_, String>(14)?).unwrap_or(Decimal::ZERO),
            high_limit: row.get::<_, Option<String>>(15)?.and_then(|s| Decimal::from_str(&s).ok()),
            low_limit: row.get::<_, Option<String>>(16)?.and_then(|s| Decimal::from_str(&s).ok()),
            inner_vol: row.get(17)?,
            outer_vol: row.get(18)?,
            open_vol: row.get(19)?,
            time: row.get(20)?,
        })
    });

    match result {
        Ok(q) => Ok(Some(q)),
        Err(_) => Ok(None),
    }
}
