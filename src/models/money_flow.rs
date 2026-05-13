/// 资金流向
use crate::models::stock::StockCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyFlow {
    pub code: StockCode,
    /// 交易日期 YYYY-MM-DD
    pub trade_date: String,
    /// 主力流入(元)
    pub main_inflow: Decimal,
    /// 主力流出(元)
    pub main_outflow: Decimal,
    /// 主力净流入(元)
    pub main_net: Decimal,
    /// 散户流入(元)
    pub retail_inflow: Decimal,
    /// 散户流出(元)
    pub retail_outflow: Decimal,
    /// 散户净流入(元)
    pub retail_net: Decimal,
    /// 数据来源
    pub source: String,
}
