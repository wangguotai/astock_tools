/// 预警规则管理
use rusqlite::{params, Connection};

/// 预警规则
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub id: i64,
    pub code: String,
    pub signal_type: String,
    pub params: String,
    pub enabled: bool,
}

/// 添加预警规则
pub fn add_alert_rule(
    conn: &Connection,
    code: &str,
    signal_type: &str,
    params_json: &str,
) -> anyhow::Result<i64> {
    conn.execute(
        "INSERT INTO alert_rules (code, signal_type, params) VALUES (?1, ?2, ?3)",
        params![code, signal_type, params_json],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 查询所有启用的预警规则
pub fn list_enabled_rules(conn: &Connection) -> anyhow::Result<Vec<AlertRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, code, signal_type, params, enabled FROM alert_rules WHERE enabled = 1 ORDER BY id"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AlertRule {
            id: row.get(0)?,
            code: row.get(1)?,
            signal_type: row.get(2)?,
            params: row.get(3)?,
            enabled: row.get::<_, i32>(4)? == 1,
        })
    })?;
    let mut rules = Vec::new();
    for row in rows {
        rules.push(row?);
    }
    Ok(rules)
}

/// 查询所有预警规则
pub fn list_all_rules(conn: &Connection) -> anyhow::Result<Vec<AlertRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, code, signal_type, params, enabled FROM alert_rules ORDER BY id"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AlertRule {
            id: row.get(0)?,
            code: row.get(1)?,
            signal_type: row.get(2)?,
            params: row.get(3)?,
            enabled: row.get::<_, i32>(4)? == 1,
        })
    })?;
    let mut rules = Vec::new();
    for row in rows {
        rules.push(row?);
    }
    Ok(rules)
}

/// 删除预警规则
pub fn delete_alert_rule(conn: &Connection, id: i64) -> anyhow::Result<bool> {
    let affected = conn.execute("DELETE FROM alert_rules WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

/// 记录触发的预警
pub fn record_alert(
    conn: &Connection,
    rule_id: Option<i64>,
    code: &str,
    signal_type: &str,
    message: &str,
) -> anyhow::Result<i64> {
    conn.execute(
        "INSERT INTO alerts (rule_id, code, signal_type, message) VALUES (?1, ?2, ?3, ?4)",
        params![rule_id, code, signal_type, message],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 查询预警历史
pub fn list_alert_history(conn: &Connection, limit: i64) -> anyhow::Result<Vec<(i64, String, String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, code, signal_type, message, triggered_at FROM alerts ORDER BY id DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    })?;
    let mut alerts = Vec::new();
    for row in rows {
        alerts.push(row?);
    }
    Ok(alerts)
}
