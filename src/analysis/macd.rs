/// MACD 指标 (Moving Average Convergence Divergence)
///
/// 使用 ta crate 的 MovingAverageConvergenceDivergence
use crate::models::bar::Bar;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use ta::indicators::MovingAverageConvergenceDivergence;
use ta::Next;

/// MACD 计算结果
#[derive(Debug, Clone)]
pub struct MacdResult {
    /// DIF: 快线 (MACD线)
    pub dif: Option<Decimal>,
    /// DEA: 慢线 (信号线)
    pub dea: Option<Decimal>,
    /// MACD柱状图 = (DIF - DEA) * 2
    pub histogram: Option<Decimal>,
}

/// 计算MACD
///
/// fast_period: 快线周期 (默认12)
/// slow_period: 慢线周期 (默认26)
/// signal_period: 信号线周期 (默认9)
pub fn compute_macd(
    bars: &[Bar],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Vec<MacdResult> {
    let mut macd = MovingAverageConvergenceDivergence::new(fast_period, slow_period, signal_period).unwrap();
    let mut results = Vec::with_capacity(bars.len());

    for bar in bars {
        let close = bar.close.to_f64().unwrap_or(0.0);
        let output = macd.next(close);
        results.push(MacdResult {
            dif: Decimal::from_f64_retain(output.macd),
            dea: Decimal::from_f64_retain(output.signal),
            histogram: Decimal::from_f64_retain(output.histogram),
        });
    }

    // MACD 需要 slow_period + signal_period - 1 个数据点才有有效值
    let warmup_count = (slow_period + signal_period - 1).min(results.len());
    for result in results.iter_mut().take(warmup_count) {
        result.dif = None;
        result.dea = None;
        result.histogram = None;
    }

    results
}
