/// RSI 指标 (Relative Strength Index 相对强弱指标)
///
/// 使用 ta crate 的 RelativeStrengthIndex
use crate::models::bar::Bar;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use ta::indicators::RelativeStrengthIndex;
use ta::Next;

/// 计算RSI
///
/// period: 周期 (默认14)
pub fn compute_rsi(bars: &[Bar], period: usize) -> Vec<Option<Decimal>> {
    let mut rsi = RelativeStrengthIndex::new(period).unwrap();
    let mut result = Vec::with_capacity(bars.len());

    for bar in bars {
        let close = bar.close.to_f64().unwrap_or(0.0);
        let value = rsi.next(close);
        result.push(Decimal::from_f64_retain(value));
    }

    // RSI 需要 period 个数据点
    for item in result.iter_mut().take(period.saturating_sub(1)) {
        *item = None;
    }

    result
}
