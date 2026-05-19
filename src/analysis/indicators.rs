/// 移动平均线指标 (MA, EMA, BOLL)
///
/// 使用 ta crate 的 SimpleMovingAverage / ExponentialMovingAverage / BollingerBands
use crate::models::bar::Bar;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
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

/// 均线结果 (用于告警)
#[derive(Debug, Clone)]
pub struct MaBundle {
    pub ma5: Option<f64>,
    pub ma10: Option<f64>,
    pub ma20: Option<f64>,
}

/// 计算 MA 均线 (内部使用 f64)
fn calc_ma_f64(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period {
        return None;
    }
    let sum: f64 = closes[closes.len() - period..].iter().sum();
    Some(sum / period as f64)
}

/// 计算 MA5/MA10/MA20
pub fn calc_ma_bundle(bars: &[Bar]) -> MaBundle {
    let closes: Vec<f64> = bars
        .iter()
        .map(|b| b.close.to_f64().unwrap_or(0.0))
        .collect();

    MaBundle {
        ma5: calc_ma_f64(&closes, 5),
        ma10: calc_ma_f64(&closes, 10),
        ma20: calc_ma_f64(&closes, 20),
    }
}

/// 计算 RSI (14日) - 纯 f64 版本用于告警
pub fn calc_rsi14(bars: &[Bar]) -> Option<f64> {
    if bars.len() < 15 {
        return None;
    }

    let closes: Vec<f64> = bars
        .iter()
        .map(|b| b.close.to_f64().unwrap_or(0.0))
        .collect();

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..closes.len() {
        let change = closes[i] - closes[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    let period = 14;
    if gains.len() < period {
        return None;
    }

    let mut avg_gain: f64 = gains[gains.len() - period..].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[gains.len() - period..].iter().sum::<f64>() / period as f64;

    for i in (gains.len() - period)..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;
    }

    if avg_loss == 0.0 {
        return Some(100.0);
    }

    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}

/// 均线交叉类型
#[derive(Debug, Clone, PartialEq)]
pub enum CrossType {
    GoldenCross, // 金叉：MA5 上穿 MA10
    DeadCross,   // 死叉：MA5 下穿 MA10
}

/// 均线交叉检测结果
#[derive(Debug, Clone)]
pub struct CrossResult {
    pub cross_type: CrossType,
    pub ma5_before: f64,
    pub ma10_before: f64,
    pub ma5_after: f64,
    pub ma10_after: f64,
}

/// 检测均线交叉 (MA5 与 MA10)
pub fn detect_cross(bars: &[Bar]) -> Option<CrossResult> {
    if bars.len() < 21 {
        return None;
    }

    let closes: Vec<f64> = bars.iter().map(|b| b.close.to_f64().unwrap_or(0.0)).collect();

    let prev_closes = &closes[..closes.len() - 1];
    let ma5_before = calc_ma_f64(prev_closes, 5);
    let ma10_before = calc_ma_f64(prev_closes, 10);

    let ma5_after = calc_ma_f64(&closes, 5);
    let ma10_after = calc_ma_f64(&closes, 10);

    match (ma5_before, ma10_before, ma5_after, ma10_after) {
        (Some(m5b), Some(m10b), Some(m5a), Some(m10a)) => {
            if m5b <= m10b && m5a > m10a {
                return Some(CrossResult {
                    cross_type: CrossType::GoldenCross,
                    ma5_before: m5b,
                    ma10_before: m10b,
                    ma5_after: m5a,
                    ma10_after: m10a,
                });
            }
            if m5b >= m10b && m5a < m10a {
                return Some(CrossResult {
                    cross_type: CrossType::DeadCross,
                    ma5_before: m5b,
                    ma10_before: m10b,
                    ma5_after: m5a,
                    ma10_after: m10a,
                });
            }
            None
        }
        _ => None,
    }
}
