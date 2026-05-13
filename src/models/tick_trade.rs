/// 分时成交明细
use crate::models::stock::StockCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickTrade {
    pub code: StockCode,
    /// 成交时间 HH:MM:SS
    pub trade_time: String,
    /// 成交价格
    pub price: Decimal,
    /// 成交量(股)
    pub volume: i64,
    /// 成交方向: buy/sell/neutral
    pub direction: String,
    /// 数据来源
    pub source: String,
}
