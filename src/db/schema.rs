/// SQL建表语句
pub const CREATE_TABLES: &str = r#"
CREATE TABLE IF NOT EXISTS daily_bars (
    code      TEXT NOT NULL,
    date      TEXT NOT NULL,
    open      TEXT NOT NULL,
    high      TEXT NOT NULL,
    low       TEXT NOT NULL,
    close     TEXT NOT NULL,
    volume    INTEGER NOT NULL,
    turnover  TEXT NOT NULL DEFAULT '0',
    PRIMARY KEY (code, date)
);

CREATE INDEX IF NOT EXISTS idx_bars_code_date ON daily_bars(code, date);

CREATE TABLE IF NOT EXISTS trades (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    code         TEXT NOT NULL,
    action       TEXT NOT NULL,
    price        TEXT NOT NULL,
    shares       INTEGER NOT NULL,
    amount       TEXT NOT NULL,
    commission   TEXT NOT NULL DEFAULT '0',
    stamp_tax    TEXT NOT NULL DEFAULT '0',
    transfer_fee TEXT NOT NULL DEFAULT '0',
    total_cost   TEXT NOT NULL DEFAULT '0',
    trade_date   TEXT NOT NULL,
    note         TEXT DEFAULT '',
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS positions (
    code        TEXT PRIMARY KEY,
    name        TEXT DEFAULT '',
    shares      INTEGER NOT NULL,
    avg_cost    TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS watchlist (
    code        TEXT PRIMARY KEY,
    name        TEXT DEFAULT '',
    added_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS alert_rules (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL,
    signal_type TEXT NOT NULL,
    params      TEXT DEFAULT '{}',
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS alerts (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id      INTEGER,
    code         TEXT NOT NULL,
    signal_type  TEXT NOT NULL,
    message      TEXT NOT NULL,
    triggered_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;
