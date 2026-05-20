/// 大宗交易
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTrade {
    pub code: String,
    pub name: String,
    /// 交易日期
    pub trade_date: String,
    /// 成交价
    pub deal_price: Decimal,
    /// 成交量(股)
    pub deal_volume: Decimal,
    /// 成交额
    pub deal_amt: Decimal,
    /// 溢价率%
    pub premium_ratio: Option<Decimal>,
    /// 买方营业部
    pub buyer_name: String,
    /// 卖方营业部
    pub seller_name: String,
}
