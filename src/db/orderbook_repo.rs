/// 盘口快照仓库
use crate::models::order_book::OrderBookSnapshot;
use crate::models::stock::StockCode;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use std::str::FromStr;

/// 保存盘口快照
pub fn save_order_book(conn: &Connection, ob: &OrderBookSnapshot) -> anyhow::Result<()> {
    let code = ob.code.for_api();
    let bid_prices: Vec<String> = ob.bid_prices.iter().map(|p| p.to_string()).collect();
    let bid_volumes: Vec<i64> = ob.bid_volumes.to_vec();
    let ask_prices: Vec<String> = ob.ask_prices.iter().map(|p| p.to_string()).collect();
    let ask_volumes: Vec<i64> = ob.ask_volumes.to_vec();

    conn.execute(
        "INSERT INTO order_book_snapshots (code, snap_time, bid_prices, bid_volumes, ask_prices, ask_volumes, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            code,
            ob.snap_time,
            serde_json::to_string(&bid_prices)?,
            serde_json::to_string(&bid_volumes)?,
            serde_json::to_string(&ask_prices)?,
            serde_json::to_string(&ask_volumes)?,
            ob.source,
        ],
    )?;
    Ok(())
}

/// 查询最新盘口快照
pub fn get_latest_order_book(
    conn: &Connection,
    code: &StockCode,
) -> anyhow::Result<Option<OrderBookSnapshot>> {
    let code_str = code.for_api();
    let mut stmt = conn.prepare(
        "SELECT code, snap_time, bid_prices, bid_volumes, ask_prices, ask_volumes, source
         FROM order_book_snapshots WHERE code = ?1 ORDER BY snap_time DESC LIMIT 1"
    )?;

    let result = stmt.query_row(params![code_str], |row| {
        let code_s: String = row.get(0)?;
        let time_s: String = row.get(1)?;
        let bp_s: String = row.get(2)?;
        let bv_s: String = row.get(3)?;
        let ap_s: String = row.get(4)?;
        let av_s: String = row.get(5)?;
        let src: String = row.get(6)?;
        Ok((code_s, time_s, bp_s, bv_s, ap_s, av_s, src))
    });

    match result {
        Ok((c, t, bp_s, bv_s, ap_s, av_s, src)) => {
            let bid_prices_str: Vec<String> = serde_json::from_str(&bp_s).unwrap_or_default();
            let bid_volumes: Vec<i64> = serde_json::from_str(&bv_s).unwrap_or_default();
            let ask_prices_str: Vec<String> = serde_json::from_str(&ap_s).unwrap_or_default();
            let ask_volumes: Vec<i64> = serde_json::from_str(&av_s).unwrap_or_default();

            let parse_arr = |strs: &[String]| -> [Decimal; 5] {
                let mut arr = [Decimal::ZERO; 5];
                for (i, s) in strs.iter().take(5).enumerate() {
                    arr[i] = Decimal::from_str(s).unwrap_or(Decimal::ZERO);
                }
                arr
            };
            let parse_vol_arr = |vols: &[i64]| -> [i64; 5] {
                let mut arr = [0i64; 5];
                for (i, &v) in vols.iter().take(5).enumerate() {
                    arr[i] = v;
                }
                arr
            };

            let ob_code = StockCode::from_raw(&c).unwrap_or_else(|_| code.clone());
            Ok(Some(OrderBookSnapshot {
                code: ob_code,
                snap_time: t,
                bid_prices: parse_arr(&bid_prices_str),
                bid_volumes: parse_vol_arr(&bid_volumes),
                ask_prices: parse_arr(&ask_prices_str),
                ask_volumes: parse_vol_arr(&ask_volumes),
                source: src,
            }))
        }
        Err(_) => Ok(None),
    }
}
