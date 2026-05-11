/// 内置回测策略集合
pub mod dual_ma;
pub mod macd_cross;
pub mod rsi_reversal;
pub mod kdj_cross;
pub mod boll_break;

use crate::backtest::engine::Strategy;

/// 获取所有内置策略列表
pub fn list_strategies() -> Vec<(&'static str, &'static str)> {
    vec![
        ("dual_ma", "双均线交叉 (MA5/MA20)"),
        ("macd_cross", "MACD金叉死叉"),
        ("rsi_reversal", "RSI超买超卖"),
        ("kdj_cross", "KDJ金叉死叉"),
        ("boll_break", "布林带突破"),
    ]
}

/// 根据名称创建策略
pub fn create_strategy(name: &str) -> Option<Box<dyn Strategy>> {
    match name {
        "dual_ma" => Some(Box::new(dual_ma::DualMaStrategy::default())),
        "macd_cross" => Some(Box::new(macd_cross::MacdCrossStrategy::default())),
        "rsi_reversal" => Some(Box::new(rsi_reversal::RsiReversalStrategy::default())),
        "kdj_cross" => Some(Box::new(kdj_cross::KdjCrossStrategy::default())),
        "boll_break" => Some(Box::new(boll_break::BollBreakStrategy::default())),
        _ => None,
    }
}
