/// 回测配置
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// 回测配置参数
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// 初始资金 (默认100000)
    pub initial_capital: Decimal,
    /// 开始日期
    pub start_date: Option<String>,
    /// 结束日期
    pub end_date: Option<String>,
    /// 是否启用T+1规则 (默认true)
    pub t_plus_1: bool,
    /// 滑点 (默认0.1%)
    pub slippage: Decimal,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: dec!(100000),
            start_date: None,
            end_date: None,
            t_plus_1: true,
            slippage: dec!(0.001), // 0.1%
        }
    }
}
