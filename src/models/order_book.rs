/// 盘口5档快照
use crate::models::stock::StockCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub code: StockCode,
    /// 快照时间
    pub snap_time: String,
    /// 买1-买5价格
    pub bid_prices: [Decimal; 5],
    /// 买1-买5量(股)
    pub bid_volumes: [i64; 5],
    /// 卖1-卖5价格
    pub ask_prices: [Decimal; 5],
    /// 卖1-卖5量(股)
    pub ask_volumes: [i64; 5],
    /// 数据来源
    pub source: String,
}
