/// K线数据模型 (OHLCV - 开高低收量)
use crate::models::stock::StockCode;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

/// 单根K线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub code: StockCode,
    pub date: NaiveDate,
    pub open: Decimal,
    pub close: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub volume: i64,      // 成交量(股)
    pub turnover: Decimal, // 成交额(元)
}

/// K线时间序列
#[derive(Debug, Clone)]
pub struct BarSeries {
    pub bars: Vec<Bar>,
}

impl BarSeries {
    pub fn new(bars: Vec<Bar>) -> Self {
        Self { bars }
    }

    /// 获取最近N根K线
    pub fn last_n(&self, n: usize) -> &[Bar] {
        let start = self.bars.len().saturating_sub(n);
        &self.bars[start..]
    }

    /// 获取收盘价序列 (用于计算指标)
    pub fn closes(&self) -> Vec<f64> {
        self.bars
            .iter()
            .map(|b| b.close.to_f64().unwrap_or(0.0))
            .collect()
    }

    /// 获取最高价序列
    pub fn highs(&self) -> Vec<f64> {
        self.bars
            .iter()
            .map(|b| b.high.to_f64().unwrap_or(0.0))
            .collect()
    }

    /// 获取最低价序列
    pub fn lows(&self) -> Vec<f64> {
        self.bars
            .iter()
            .map(|b| b.low.to_f64().unwrap_or(0.0))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bars.len()
    }
}
