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

CREATE TABLE IF NOT EXISTS tick_trades (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    tick_id    TEXT UNIQUE,
    code       TEXT NOT NULL,
    trade_time TEXT NOT NULL,
    price      TEXT NOT NULL,
    volume     INTEGER NOT NULL,
    direction  TEXT NOT NULL DEFAULT 'neutral',
    source     TEXT NOT NULL DEFAULT '10jqka',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_tick_code_time ON tick_trades(code, trade_time);
CREATE INDEX IF NOT EXISTS idx_tick_tick_id ON tick_trades(tick_id);

CREATE TABLE IF NOT EXISTS order_book_snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL,
    snap_time   TEXT NOT NULL,
    bid_prices  TEXT NOT NULL,
    bid_volumes TEXT NOT NULL,
    ask_prices  TEXT NOT NULL,
    ask_volumes TEXT NOT NULL,
    source      TEXT NOT NULL DEFAULT '10jqka',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_ob_code_time ON order_book_snapshots(code, snap_time);

CREATE TABLE IF NOT EXISTS money_flow (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    code           TEXT NOT NULL,
    trade_date     TEXT NOT NULL,
    main_inflow    TEXT NOT NULL,
    main_outflow   TEXT NOT NULL,
    main_net       TEXT NOT NULL,
    retail_inflow  TEXT NOT NULL,
    retail_outflow TEXT NOT NULL,
    retail_net     TEXT NOT NULL,
    source         TEXT NOT NULL DEFAULT '10jqka',
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(code, trade_date)
);
CREATE INDEX IF NOT EXISTS idx_mf_code_date ON money_flow(code, trade_date);

CREATE TABLE IF NOT EXISTS quote_snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL,
    name        TEXT DEFAULT '',
    price       TEXT NOT NULL,
    prev_close  TEXT NOT NULL,
    open        TEXT NOT NULL,
    high        TEXT NOT NULL,
    low         TEXT NOT NULL,
    volume      INTEGER NOT NULL DEFAULT 0,
    turnover    TEXT NOT NULL DEFAULT '0',
    bid         TEXT NOT NULL DEFAULT '0',
    ask         TEXT NOT NULL DEFAULT '0',
    bid_vol     INTEGER,
    ask_vol     INTEGER,
    change_val  TEXT NOT NULL DEFAULT '0',
    change_pct  TEXT NOT NULL DEFAULT '0',
    high_limit  TEXT,
    low_limit   TEXT,
    inner_vol   INTEGER,
    outer_vol   INTEGER,
    open_vol    INTEGER,
    quote_time  TEXT DEFAULT '',
    update_time TEXT,
    stock_status TEXT,
    source      TEXT NOT NULL DEFAULT '10jqka',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_qs_code_time ON quote_snapshots(code, created_at);
"#;
