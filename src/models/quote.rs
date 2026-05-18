/// 实时行情快照
use crate::models::stock::StockCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub code: StockCode,
    pub name: String,
    /// 当前价格
    pub price: Decimal,
    /// 昨收价
    pub prev_close: Decimal,
    /// 今开
    pub open: Decimal,
    /// 最高
    pub high: Decimal,
    /// 最低
    pub low: Decimal,
    /// 成交量(股)
    pub volume: i64,
    /// 成交额(元)
    pub turnover: Decimal,
    /// 买一价
    pub bid: Decimal,
    /// 卖一价
    pub ask: Decimal,
    /// 买一量
    pub bid_vol: Option<i64>,
    /// 卖一量
    pub ask_vol: Option<i64>,
    /// 涨跌额
    pub change: Decimal,
    /// 涨跌幅(%)
    pub change_pct: Decimal,
    /// 涨停价
    pub high_limit: Option<Decimal>,
    /// 跌停价
    pub low_limit: Option<Decimal>,
    /// 内盘（主动卖出量）
    pub inner_vol: Option<i64>,
    /// 外盘（主动买入量）
    pub outer_vol: Option<i64>,
    /// 开盘成交量
    pub open_vol: Option<i64>,
    /// 数据时间
    pub time: String,
    /// 同花顺 updateTime 字段
    pub update_time: Option<String>,
    /// 交易状态：连续竞价/闭市/休市
    pub stock_status: Option<String>,
}

impl Quote {
    /// 判断是否上涨
    pub fn is_up(&self) -> bool {
        self.change > Decimal::ZERO
    }

    /// 判断是否下跌
    pub fn is_down(&self) -> bool {
        self.change < Decimal::ZERO
    }
}
