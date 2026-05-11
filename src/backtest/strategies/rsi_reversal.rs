/// RSI超买超卖策略
///
/// 买入: RSI从超卖区上穿30
/// 卖出: RSI从超买区下穿70
use crate::analysis::rsi::compute_rsi;
use crate::backtest::engine::{Signal, Strategy};
use crate::models::bar::Bar;
use rust_decimal::Decimal;

pub struct RsiReversalStrategy {
    pub oversold: Decimal,
    pub overbought: Decimal,
}

impl Default for RsiReversalStrategy {
    fn default() -> Self {
        Self {
            oversold: Decimal::from(30),
            overbought: Decimal::from(70),
        }
    }
}

impl Strategy for RsiReversalStrategy {
    fn name(&self) -> &str {
        "RSI超买超卖"
    }

    fn generate_signal(&self, bars: &[Bar], index: usize) -> Signal {
        if index < 15 {
            return Signal::Hold;
        }

        let slice = &bars[..=index];
        let rsi = compute_rsi(slice, 14);

        let curr = rsi[index];
        let prev = rsi[index - 1];

        match (curr, prev) {
            (Some(c), Some(p)) => {
                // 从超卖区回升: 前一RSI<30 且 当前RSI>30
                if p < self.oversold && c >= self.oversold {
                    Signal::Buy
                }
                // 从超买区回落: 前一RSI>70 且 当前RSI<=70
                else if p > self.overbought && c <= self.overbought {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        }
    }
}
