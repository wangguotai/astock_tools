/// 布林带突破策略
///
/// 买入: 收盘价从下方突破下轨 (超卖反弹)
/// 卖出: 收盘价从上方突破上轨 (超买回落)
use crate::analysis::indicators::compute_boll;
use crate::backtest::engine::{Signal, Strategy};
use crate::models::bar::Bar;

pub struct BollBreakStrategy {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for BollBreakStrategy {
    fn default() -> Self {
        Self {
            period: 20,
            multiplier: 2.0,
        }
    }
}

impl Strategy for BollBreakStrategy {
    fn name(&self) -> &str {
        "布林带突破"
    }

    fn generate_signal(&self, bars: &[Bar], index: usize) -> Signal {
        if index < self.period {
            return Signal::Hold;
        }

        let slice = &bars[..=index];
        let boll = compute_boll(slice, self.period, self.multiplier);

        let curr_close = bars[index].close;
        let prev_close = bars[index - 1].close;
        let curr_upper = boll[index].upper;
        let curr_lower = boll[index].lower;
        let prev_upper = boll[index - 1].upper;
        let prev_lower = boll[index - 1].lower;

        match (curr_upper, curr_lower, prev_upper, prev_lower) {
            (Some(cu), Some(cl), Some(pu), Some(pl)) => {
                // 收盘价从下方突破下轨 (反弹买入)
                if prev_close <= pl && curr_close > cl {
                    Signal::Buy
                }
                // 收盘价从上方突破上轨 (回落卖出)
                else if prev_close >= pu && curr_close < cu {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        }
    }
}
