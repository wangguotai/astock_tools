/// 统一错误类型定义
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AstockError {
    /// 无效的股票代码
    #[error("无效的股票代码: {0}")]
    InvalidCode(String),

    /// API请求失败
    #[error("API请求失败: {0}")]
    ApiError(String),

    /// 数据解析失败
    #[error("数据解析失败: {0}")]
    ParseError(String),

    /// 市场已关闭
    #[error("市场已关闭")]
    MarketClosed,
}
