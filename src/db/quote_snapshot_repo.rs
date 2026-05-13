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
        "INSERT INTO quote_snapshots (code, name, price, prev_close, open, high, low, volume, turnover, bid, ask, change_val, change_pct, quote_time, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
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
            quote.change.to_string(),
            quote.change_pct.to_string(),
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
        "SELECT code, name, price, prev_close, open, high, low, volume, turnover, bid, ask, change_val, change_pct, quote_time
         FROM quote_snapshots WHERE code = ?1 ORDER BY created_at DESC LIMIT 1"
    )?;

    let result = stmt.query_row(params![code_str], |row| {
        let c: String = row.get(0)?;
        let n: String = row.get(1)?;
        let p: String = row.get(2)?;
        let pc: String = row.get(3)?;
        let o: String = row.get(4)?;
        let h: String = row.get(5)?;
        let l: String = row.get(6)?;
        let v: i64 = row.get(7)?;
        let t: String = row.get(8)?;
        let b: String = row.get(9)?;
        let a: String = row.get(10)?;
        let ch: String = row.get(11)?;
        let cp: String = row.get(12)?;
        let qt: String = row.get(13)?;
        Ok((c, n, p, pc, o, h, l, v, t, b, a, ch, cp, qt))
    });

    match result {
        Ok((c, n, p, pc, o, h, l, v, t, b, a, ch, cp, qt)) => {
            let q_code = StockCode::from_raw(&c).unwrap_or_else(|_| code.clone());
            Ok(Some(Quote {
                code: q_code,
                name: n,
                price: Decimal::from_str(&p).unwrap_or(Decimal::ZERO),
                prev_close: Decimal::from_str(&pc).unwrap_or(Decimal::ZERO),
                open: Decimal::from_str(&o).unwrap_or(Decimal::ZERO),
                high: Decimal::from_str(&h).unwrap_or(Decimal::ZERO),
                low: Decimal::from_str(&l).unwrap_or(Decimal::ZERO),
                volume: v,
                turnover: Decimal::from_str(&t).unwrap_or(Decimal::ZERO),
                bid: Decimal::from_str(&b).unwrap_or(Decimal::ZERO),
                ask: Decimal::from_str(&a).unwrap_or(Decimal::ZERO),
                change: Decimal::from_str(&ch).unwrap_or(Decimal::ZERO),
                change_pct: Decimal::from_str(&cp).unwrap_or(Decimal::ZERO),
                time: qt,
            }))
        }
        Err(_) => Ok(None),
    }
}
