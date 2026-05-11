/// 移动平均线指标 (MA, EMA, BOLL)
///
/// 使用 ta crate 的 SimpleMovingAverage / ExponentialMovingAverage / BollingerBands
use crate::models::bar::Bar;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use ta::indicators::{BollingerBands, BollingerBandsOutput, ExponentialMovingAverage, SimpleMovingAverage};
use ta::Next;

/// 计算简单移动平均线 (SMA)
///
/// period: 周期 (如 5, 10, 20, 60)
/// 返回: 每个bar对应的SMA值 (前期不足时为None)
pub fn compute_sma(bars: &[Bar], period: usize) -> Vec<Option<Decimal>> {
    let mut sma = SimpleMovingAverage::new(period).unwrap();
    let mut result = Vec::with_capacity(bars.len());

    for bar in bars {
        let close = bar.close.to_f64().unwrap_or(0.0);
        let value = sma.next(close);
        result.push(Decimal::from_f64_retain(value));
    }

    // ta crate 的 SMA 会在数据不足时也开始输出，我们需要将前 period-1 个标记为 None
    for item in result.iter_mut().take(period.saturating_sub(1)) {
        *item = None;
    }

    result
}

/// 计算指数移动平均线 (EMA)
pub fn compute_ema(bars: &[Bar], period: usize) -> Vec<Option<Decimal>> {
    let mut ema = ExponentialMovingAverage::new(period).unwrap();
    let mut result = Vec::with_capacity(bars.len());

    for bar in bars {
        let close = bar.close.to_f64().unwrap_or(0.0);
        let value = ema.next(close);
        result.push(Decimal::from_f64_retain(value));
    }

    for item in result.iter_mut().take(period.saturating_sub(1)) {
        *item = None;
    }

    result
}

/// 布林带计算结果
#[derive(Debug, Clone)]
pub struct BollResult {
    pub upper: Option<Decimal>,
    pub middle: Option<Decimal>,
    pub lower: Option<Decimal>,
}

/// 计算布林带 (BOLL)
///
/// period: 周期 (默认20)
/// multiplier: 标准差倍数 (默认2)
pub fn compute_boll(bars: &[Bar], period: usize, multiplier: f64) -> Vec<BollResult> {
    let mut boll = BollingerBands::new(period, multiplier).unwrap();
    let mut results = Vec::with_capacity(bars.len());

    for bar in bars {
        let close = bar.close.to_f64().unwrap_or(0.0);
        let output: BollingerBandsOutput = boll.next(close);
        results.push(BollResult {
            upper: Decimal::from_f64_retain(output.upper),
            middle: Decimal::from_f64_retain(output.average),
            lower: Decimal::from_f64_retain(output.lower),
        });
    }

    // 前 period-1 个标记为 None
    for result in results.iter_mut().take(period.saturating_sub(1)) {
        result.upper = None;
        result.middle = None;
        result.lower = None;
    }

    results
}
