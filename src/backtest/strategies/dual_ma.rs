/// 双均线交叉策略
///
/// 买入: MA5上穿MA20 (金叉)
/// 卖出: MA5下穿MA20 (死叉)
use crate::analysis::indicators::compute_sma;
use crate::backtest::engine::{Signal, Strategy};
use crate::models::bar::Bar;

pub struct DualMaStrategy {
    pub short_period: usize,
    pub long_period: usize,
}

impl Default for DualMaStrategy {
    fn default() -> Self {
        Self {
            short_period: 5,
            long_period: 20,
        }
    }
}

impl Strategy for DualMaStrategy {
    fn name(&self) -> &str {
        "双均线交叉"
    }

    fn generate_signal(&self, bars: &[Bar], index: usize) -> Signal {
        if index < self.long_period {
            return Signal::Hold;
        }

        let slice = &bars[..=index];
        let short_ma = compute_sma(slice, self.short_period);
        let long_ma = compute_sma(slice, self.long_period);

        let curr_short = short_ma[index];
        let prev_short = short_ma[index - 1];
        let curr_long = long_ma[index];
        let prev_long = long_ma[index - 1];

        match (curr_short, prev_short, curr_long, prev_long) {
            (Some(cs), Some(ps), Some(cl), Some(pl)) => {
                // 金叉: 短均线从下方穿越长均线
                if ps <= pl && cs > cl {
                    Signal::Buy
                }
                // 死叉: 短均线从上方穿越长均线
                else if ps >= pl && cs < cl {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        }
    }
}
