/// 资金流向仓库
use crate::models::money_flow::MoneyFlow;
use crate::models::stock::StockCode;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use std::str::FromStr;

/// 保存资金流向 (INSERT OR REPLACE)
pub fn save_money_flow(conn: &Connection, mf: &MoneyFlow) -> anyhow::Result<()> {
    let code = mf.code.for_api();
    conn.execute(
        "INSERT OR REPLACE INTO money_flow (code, trade_date, main_inflow, main_outflow, main_net, retail_inflow, retail_outflow, retail_net, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            code,
            mf.trade_date,
            mf.main_inflow.to_string(),
            mf.main_outflow.to_string(),
            mf.main_net.to_string(),
            mf.retail_inflow.to_string(),
            mf.retail_outflow.to_string(),
            mf.retail_net.to_string(),
            mf.source,
        ],
    )?;
    Ok(())
}

/// 查询资金流向
pub fn query_money_flow(
    conn: &Connection,
    code: &StockCode,
    limit: Option<usize>,
) -> anyhow::Result<Vec<MoneyFlow>> {
    let code_str = code.for_api();
    let sql = match limit {
        Some(_) => "SELECT code, trade_date, main_inflow, main_outflow, main_net, retail_inflow, retail_outflow, retail_net, source FROM money_flow WHERE code = ?1 ORDER BY trade_date DESC LIMIT ?2",
        None => "SELECT code, trade_date, main_inflow, main_outflow, main_net, retail_inflow, retail_outflow, retail_net, source FROM money_flow WHERE code = ?1 ORDER BY trade_date DESC",
    };

    let mut stmt = conn.prepare(sql)?;
    let flows = match limit {
        Some(lim) => {
            let rows = stmt.query_map(params![code_str, lim as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })?;
            collect_flows(rows, code)?
        }
        None => {
            let rows = stmt.query_map(params![code_str], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })?;
            collect_flows(rows, code)?
        }
    };

    Ok(flows)
}

fn collect_flows(
    rows: rusqlite::MappedRows<impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<(String, String, String, String, String, String, String, String, String)>>,
    fallback_code: &StockCode,
) -> anyhow::Result<Vec<MoneyFlow>> {
    let mut flows = Vec::new();
    for row in rows {
        let (c, d, mi, mo, mn, ri, ro, rn, s) = row?;
        let mf_code = StockCode::from_raw(&c).unwrap_or_else(|_| fallback_code.clone());
        flows.push(MoneyFlow {
            code: mf_code,
            trade_date: d,
            main_inflow: Decimal::from_str(&mi).unwrap_or(Decimal::ZERO),
            main_outflow: Decimal::from_str(&mo).unwrap_or(Decimal::ZERO),
            main_net: Decimal::from_str(&mn).unwrap_or(Decimal::ZERO),
            retail_inflow: Decimal::from_str(&ri).unwrap_or(Decimal::ZERO),
            retail_outflow: Decimal::from_str(&ro).unwrap_or(Decimal::ZERO),
            retail_net: Decimal::from_str(&rn).unwrap_or(Decimal::ZERO),
            source: s,
        });
    }
    flows.reverse();
    Ok(flows)
}
