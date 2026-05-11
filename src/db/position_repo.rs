/// 持仓管理仓库
use crate::models::stock::StockCode;
use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use std::str::FromStr;

/// 持仓记录
#[derive(Debug, Clone)]
pub struct PositionRecord {
    pub code: String,
    pub name: String,
    pub shares: i64,
    pub avg_cost: Decimal,
    pub current_price: Decimal,
}

/// 更新或添加持仓 (买入时)
pub fn upsert_position(
    conn: &Connection,
    code: &StockCode,
    name: &str,
    shares: i64,
    price: Decimal,
) -> anyhow::Result<()> {
    // 查看现有持仓
    let existing = get_position(conn, code)?;

    match existing {
        Some((old_shares, old_avg_cost)) => {
            // 加仓: 计算新的均价
            let new_shares = old_shares + shares;
            let new_avg_cost = if new_shares > 0 {
                (old_avg_cost * Decimal::from(old_shares) + price * Decimal::from(shares))
                    / Decimal::from(new_shares)
            } else {
                price
            };
            conn.execute(
                "UPDATE positions SET shares = ?1, avg_cost = ?2, name = ?3, updated_at = datetime('now') WHERE code = ?4",
                params![new_shares, new_avg_cost.to_string(), name, code.for_api()],
            )?;
        }
        None => {
            conn.execute(
                "INSERT INTO positions (code, name, shares, avg_cost) VALUES (?1, ?2, ?3, ?4)",
                params![code.for_api(), name, shares, price.to_string()],
            )?;
        }
    }

    Ok(())
}

/// 减仓 (卖出时)
pub fn reduce_position(
    conn: &Connection,
    code: &StockCode,
    shares: i64,
) -> anyhow::Result<bool> {
    let existing = get_position(conn, code)?;

    match existing {
        Some((old_shares, old_avg_cost)) => {
            let new_shares = old_shares - shares;
            if new_shares <= 0 {
                // 清仓
                conn.execute("DELETE FROM positions WHERE code = ?1", params![code.for_api()])?;
            } else {
                conn.execute(
                    "UPDATE positions SET shares = ?1, updated_at = datetime('now') WHERE code = ?2",
                    params![new_shares, code.for_api()],
                )?;
            }
            Ok(true)
        }
        None => Ok(false),
    }
}

/// 查询所有持仓
pub fn list_positions(conn: &Connection) -> anyhow::Result<Vec<PositionRecord>> {
    let mut stmt = conn.prepare(
        "SELECT code, name, shares, avg_cost FROM positions ORDER BY code"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(PositionRecord {
            code: row.get(0)?,
            name: row.get(1)?,
            shares: row.get(2)?,
            avg_cost: Decimal::from_str(&row.get::<_, String>(3)?).unwrap_or(Decimal::ZERO),
            current_price: Decimal::ZERO, // 需要实时获取
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

/// 删除持仓
pub fn remove_position(conn: &Connection, code: &StockCode) -> anyhow::Result<bool> {
    let affected = conn.execute("DELETE FROM positions WHERE code = ?1", params![code.for_api()])?;
    Ok(affected > 0)
}

/// 获取单个持仓
fn get_position(conn: &Connection, code: &StockCode) -> anyhow::Result<Option<(i64, Decimal)>> {
    let mut stmt = conn.prepare("SELECT shares, avg_cost FROM positions WHERE code = ?1")?;
    let result = stmt.query_row(params![code.for_api()], |row| {
        let shares: i64 = row.get(0)?;
        let avg_cost: String = row.get(1)?;
        Ok((shares, Decimal::from_str(&avg_cost).unwrap_or(Decimal::ZERO)))
    });

    match result {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
