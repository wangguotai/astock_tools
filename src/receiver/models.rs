/// HTTP接收端请求/响应模型
use serde::{Deserialize, Serialize};

/// 通用推送请求
#[derive(Debug, Deserialize)]
pub struct PushRequest<T> {
    /// 数据来源
    pub source: String,
    /// 推送时间
    pub timestamp: Option<String>,
    /// 推送数据
    pub data: T,
}

/// 实时行情数据
#[derive(Debug, Deserialize)]
pub struct QuoteData {
    pub code: String,
    pub name: Option<String>,
    pub price: String,
    pub prev_close: Option<String>,
    pub open: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub volume: Option<i64>,
    pub turnover: Option<String>,
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub bid_vol: Option<i64>,
    pub ask_vol: Option<i64>,
    pub change: Option<String>,
    pub change_pct: Option<String>,
    pub high_limit: Option<String>,
    pub low_limit: Option<String>,
    pub inner_vol: Option<i64>,
    pub outer_vol: Option<i64>,
    pub open_vol: Option<i64>,
    pub time: Option<String>,
    pub update_time: Option<String>,
    pub stock_status: Option<String>,
}

/// K线数据条目
#[derive(Debug, Deserialize)]
pub struct KlineEntry {
    pub code: String,
    pub date: String,
    pub open: String,
    pub close: String,
    pub high: String,
    pub low: String,
    pub volume: Option<i64>,
    pub turnover: Option<String>,
}

/// K线批量推送
#[derive(Debug, Deserialize)]
pub struct KlineBatchData {
    pub bars: Vec<KlineEntry>,
}

/// 分时成交条目
#[derive(Debug, Deserialize)]
pub struct TickEntry {
    pub id: Option<String>,
    pub code: String,
    pub trade_time: String,
    pub price: String,
    pub volume: i64,
    pub direction: Option<String>,
}

/// 分时成交批量推送
#[derive(Debug, Deserialize)]
pub struct TickBatchData {
    pub ticks: Vec<TickEntry>,
}

/// 盘口快照
#[derive(Debug, Deserialize)]
pub struct OrderBookData {
    pub code: String,
    pub snap_time: String,
    pub bid_prices: Vec<String>,
    pub bid_volumes: Vec<i64>,
    pub ask_prices: Vec<String>,
    pub ask_volumes: Vec<i64>,
}

/// 资金流向
#[derive(Debug, Deserialize)]
pub struct MoneyFlowData {
    pub code: String,
    pub trade_date: String,
    pub main_inflow: String,
    pub main_outflow: String,
    pub main_net: String,
    pub retail_inflow: String,
    pub retail_outflow: String,
    pub retail_net: String,
}

/// 统一响应
#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: Option<String>,
}

impl ApiResponse {
    pub fn ok() -> Self {
        ApiResponse { status: "ok".into(), message: None }
    }

    pub fn error(msg: &str) -> Self {
        ApiResponse { status: "error".into(), message: Some(msg.into()) }
    }
}

/// 状态响应
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

/// 自选股条目
#[derive(Debug, Serialize)]
pub struct WatchlistEntry {
    pub code: String,
    pub name: String,
}
