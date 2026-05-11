/// 交易记录仓库
use crate::backtest::costs::{calc_buy_cost, calc_sell_cost};
use crate::models::stock::StockCode;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use std::str::FromStr;

/// 交易记录
#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub id: i64,
    pub code: String,
    pub action: String,
    pub price: Decimal,
    pub shares: i64,
    pub amount: Decimal,
    pub commission: Decimal,
    pub stamp_tax: Decimal,
    pub transfer_fee: Decimal,
    pub total_cost: Decimal,
    pub trade_date: String,
    pub note: String,
}

/// 添加交易记录
pub fn add_trade(
    conn: &Connection,
    code: &StockCode,
    action: &str,
    price: Decimal,
    shares: i64,
    trade_date: &str,
    note: &str,
) -> anyhow::Result<i64> {
    let amount = price * Decimal::from(shares);

    // 根据买卖方向计算费用
    let total_cost = match action {
        "BUY" => calc_buy_cost(price, shares),
        "SELL" => calc_sell_cost(price, shares),
        _ => Decimal::ZERO,
    };

    // 简化: 佣金 = 总费用 * 0.6 (近似), 印花税 = 卖出时 amount*0.0005, 过户费 = amount*0.00001
    let (commission, stamp_tax, transfer_fee) = match action {
        "BUY" => {
            let tf = amount * Decimal::from_str("0.00001").unwrap();
            let comm = total_cost - tf;
            (comm, Decimal::ZERO, tf)
        }
        "SELL" => {
            let st = amount * Decimal::from_str("0.0005").unwrap();
            let tf = amount * Decimal::from_str("0.00001").unwrap();
            let comm = total_cost - st - tf;
            (comm, st, tf)
        }
        _ => (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
    };

    conn.execute(
        "INSERT INTO trades (code, action, price, shares, amount, commission, stamp_tax, transfer_fee, total_cost, trade_date, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            code.for_api(),
            action,
            price.to_string(),
            shares,
            amount.to_string(),
            commission.to_string(),
            stamp_tax.to_string(),
            transfer_fee.to_string(),
            total_cost.to_string(),
            trade_date,
            note,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 查询交易记录
pub fn list_trades(
    conn: &Connection,
    code: Option<&StockCode>,
    action: Option<&str>,
) -> anyhow::Result<Vec<TradeRecord>> {
    let mut sql = String::from(
        "SELECT id, code, action, price, shares, amount, commission, stamp_tax, transfer_fee, total_cost, trade_date, note FROM trades WHERE 1=1"
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(c) = code {
        param_values.push(Box::new(c.for_api()));
        sql.push_str(&format!(" AND code = ?{}", param_values.len()));
    }
    if let Some(a) = action {
        param_values.push(Box::new(a.to_uppercase()));
        sql.push_str(&format!(" AND action = ?{}", param_values.len()));
    }
    sql.push_str(" ORDER BY trade_date DESC, id DESC");

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(TradeRecord {
            id: row.get(0)?,
            code: row.get(1)?,
            action: row.get(2)?,
            price: Decimal::from_str(&row.get::<_, String>(3)?).unwrap_or(Decimal::ZERO),
            shares: row.get(4)?,
            amount: Decimal::from_str(&row.get::<_, String>(5)?).unwrap_or(Decimal::ZERO),
            commission: Decimal::from_str(&row.get::<_, String>(6)?).unwrap_or(Decimal::ZERO),
            stamp_tax: Decimal::from_str(&row.get::<_, String>(7)?).unwrap_or(Decimal::ZERO),
            transfer_fee: Decimal::from_str(&row.get::<_, String>(8)?).unwrap_or(Decimal::ZERO),
            total_cost: Decimal::from_str(&row.get::<_, String>(9)?).unwrap_or(Decimal::ZERO),
            trade_date: row.get(10)?,
            note: row.get(11)?,
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

/// 删除交易记录
pub fn delete_trade(conn: &Connection, id: i64) -> anyhow::Result<bool> {
    let affected = conn.execute("DELETE FROM trades WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

/// 交易汇总
pub fn trade_summary(conn: &Connection) -> anyhow::Result<TradeSummary> {
    let mut stmt = conn.prepare(
        "SELECT COUNT(*) as total,
                SUM(CASE WHEN action='BUY' THEN 1 ELSE 0 END) as buys,
                SUM(CASE WHEN action='SELL' THEN 1 ELSE 0 END) as sells,
                SUM(CAST(amount AS REAL)) as total_amount,
                SUM(CAST(total_cost AS REAL)) as total_fees
         FROM trades"
    )?;

    let summary = stmt.query_row([], |row| {
        Ok(TradeSummary {
            total_trades: row.get::<_, i64>(0)?,
            buy_count: row.get::<_, i64>(1)?,
            sell_count: row.get::<_, i64>(2)?,
            total_amount: Decimal::from_f64_retain(row.get::<_, f64>(3)?).unwrap_or(Decimal::ZERO),
            total_fees: Decimal::from_f64_retain(row.get::<_, f64>(4)?).unwrap_or(Decimal::ZERO),
        })
    })?;

    Ok(summary)
}

#[derive(Debug)]
pub struct TradeSummary {
    pub total_trades: i64,
    pub buy_count: i64,
    pub sell_count: i64,
    pub total_amount: Decimal,
    pub total_fees: Decimal,
}
