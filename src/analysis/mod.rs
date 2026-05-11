/// 技术分析指标计算模块
///
/// 基于 ta crate 实现 MA/EMA/BOLL，自定义实现 KDJ
pub mod indicators;
pub mod macd;
pub mod rsi;
pub mod kdj;
pub mod report;

pub use report::{IndicatorReport, IndicatorValue, compute_all_indicators, format_report};
