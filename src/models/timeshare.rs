/// 分时历史数据点
use crate::models::stock::StockCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimesharePoint {
    pub code: StockCode,
    /// 成交时间 HH:MM:SS
    pub trade_time: String,
    /// 成交价格
    pub price: Decimal,
    /// 当笔成交量(股)
    pub volume: i64,
    /// 成交均价
    pub avg_price: Option<Decimal>,
    /// 累计成交量
    pub cum_volume: Option<i64>,
}