/// KDJ金叉死叉策略
///
/// 买入: K线上穿D线 (金叉)
/// 卖出: K线下穿D线 (死叉)
use crate::analysis::kdj::compute_kdj;
use crate::backtest::engine::{Signal, Strategy};
use crate::models::bar::Bar;

pub struct KdjCrossStrategy;

impl Default for KdjCrossStrategy {
    fn default() -> Self {
        Self
    }
}

impl Strategy for KdjCrossStrategy {
    fn name(&self) -> &str {
        "KDJ金叉死叉"
    }

    fn generate_signal(&self, bars: &[Bar], index: usize) -> Signal {
        if index < 10 {
            return Signal::Hold;
        }

        let slice = &bars[..=index];
        let kdj = compute_kdj(slice, 9, 3, 3);

        let curr_k = kdj[index].k;
        let curr_d = kdj[index].d;
        let prev_k = kdj[index - 1].k;
        let prev_d = kdj[index - 1].d;

        match (curr_k, curr_d, prev_k, prev_d) {
            (Some(ck), Some(cd), Some(pk), Some(pd)) => {
                // 金叉: K从下方穿越D
                if pk <= pd && ck > cd {
                    Signal::Buy
                }
                // 死叉: K从上方穿越D
                else if pk >= pd && ck < cd {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        }
    }
}
