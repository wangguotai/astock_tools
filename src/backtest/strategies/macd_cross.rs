/// MACD金叉死叉策略
///
/// 买入: MACD柱状图从负转正 (金叉)
/// 卖出: MACD柱状图从正转负 (死叉)
use crate::analysis::macd::compute_macd;
use crate::backtest::engine::{Signal, Strategy};
use crate::models::bar::Bar;

pub struct MacdCrossStrategy;

impl Default for MacdCrossStrategy {
    fn default() -> Self {
        Self
    }
}

impl Strategy for MacdCrossStrategy {
    fn name(&self) -> &str {
        "MACD金叉死叉"
    }

    fn generate_signal(&self, bars: &[Bar], index: usize) -> Signal {
        if index < 35 {
            // MACD需要至少35根K线
            return Signal::Hold;
        }

        let slice = &bars[..=index];
        let macd = compute_macd(slice, 12, 26, 9);

        let curr_hist = macd[index].histogram;
        let prev_hist = macd[index - 1].histogram;

        match (curr_hist, prev_hist) {
            (Some(ch), Some(ph)) => {
                // 金叉: 柱状图从负转正
                if ph <= rust_decimal::Decimal::ZERO && ch > rust_decimal::Decimal::ZERO {
                    Signal::Buy
                }
                // 死叉: 柱状图从正转负
                else if ph >= rust_decimal::Decimal::ZERO && ch < rust_decimal::Decimal::ZERO {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        }
    }
}
