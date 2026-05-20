/// 龙虎榜
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragonTigerEntry {
    pub code: String,
    pub name: String,
    /// 交易日期
    pub trade_date: String,
    /// 收盘价
    pub close_price: Decimal,
    /// 涨跌幅%
    pub change_rate: Decimal,
    /// 上榜原因
    pub explanation: String,
    /// 买入额
    pub buy_amt: Decimal,
    /// 卖出额
    pub sell_amt: Decimal,
    /// 净买入额
    pub net_buy_amt: Decimal,
    /// 成交额
    pub deal_amt: Decimal,
}
