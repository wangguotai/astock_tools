/// 分时成交仓库
use crate::models::stock::StockCode;
use crate::models::tick_trade::TickTrade;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use std::str::FromStr;

/// 批量保存分时成交
pub fn save_tick_trades(conn: &Connection, ticks: &[TickTrade]) -> anyhow::Result<usize> {
    let mut count = 0;
    let tx = conn.unchecked_transaction()?;

    for tick in ticks {
        let code = tick.code.for_api();
        let price = tick.price.to_string();
        // tick_id 为空时允许重复插入，为非空时 UNIQUE 约束避免重复
        let tick_id = tick.tick_id.as_deref();
        tx.execute(
            "INSERT OR IGNORE INTO tick_trades (tick_id, code, trade_time, price, volume, direction, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![tick_id, code, tick.trade_time, price, tick.volume, tick.direction, tick.source],
        )?;
        count += 1;
    }

    tx.commit()?;
    Ok(count)
}

/// 查询分时成交
pub fn query_tick_trades(
    conn: &Connection,
    code: &StockCode,
    limit: Option<usize>,
) -> anyhow::Result<Vec<TickTrade>> {
    let code_str = code.for_api();
    let sql = match limit {
        Some(_) => "SELECT tick_id, code, trade_time, price, volume, direction, source FROM tick_trades WHERE code = ?1 ORDER BY trade_time DESC LIMIT ?2",
        None => "SELECT tick_id, code, trade_time, price, volume, direction, source FROM tick_trades WHERE code = ?1 ORDER BY trade_time DESC",
    };

    let mut stmt = conn.prepare(sql)?;
    let ticks = match limit {
        Some(lim) => {
            let rows = stmt.query_map(params![code_str, lim as i64], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })?;
            collect_ticks(rows, code)?
        }
        None => {
            let rows = stmt.query_map(params![code_str], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })?;
            collect_ticks(rows, code)?
        }
    };

    Ok(ticks)
}

fn collect_ticks(
    rows: rusqlite::MappedRows<impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<(Option<String>, String, String, String, i64, String, String)>>,
    fallback_code: &StockCode,
) -> anyhow::Result<Vec<TickTrade>> {
    let mut ticks = Vec::new();
    for row in rows {
        let (tick_id, c, t, p, v, d, s) = row?;
        let tick_code = StockCode::from_raw(&c).unwrap_or_else(|_| fallback_code.clone());
        ticks.push(TickTrade {
            tick_id,
            code: tick_code,
            trade_time: t,
            price: Decimal::from_str(&p).unwrap_or(Decimal::ZERO),
            volume: v,
            direction: d,
            source: s,
        });
    }
    ticks.reverse();
    Ok(ticks)
}
