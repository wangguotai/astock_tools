/// 融资融券数据
use crate::models::stock::StockCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginData {
    pub code: StockCode,
    /// 交易日期 YYYY-MM-DD
    pub trade_date: String,
    /// 融资余额(元)
    pub fin_balance: Decimal,
    /// 融资买入额(元)
    pub fin_buy_amt: Decimal,
    /// 融券余额(元)
    pub loan_balance: Decimal,
    /// 融券卖出量(股)
    pub loan_sell_vol: Decimal,
}
