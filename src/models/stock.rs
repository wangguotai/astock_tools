/// 股票代码类型，自动处理市场前缀映射
///
/// A股代码规则:
/// - 6xxxxx → 上海证券交易所 (sh)
/// - 0xxxxx → 深圳主板 (sz)
/// - 3xxxxx → 深圳创业板 (sz)
/// - 8xxxxx, 4xxxxx → 北京交易所 (bj)
///
/// 内部统一使用小写格式: "sh600519", "sz000001"
use crate::error::AstockError;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StockCode {
    /// 市场前缀: "sh", "sz", "bj"
    prefix: String,
    /// 6位数字代码
    code: String,
}

impl StockCode {
    /// 从各种格式创建StockCode
    ///
    /// 支持的输入格式:
    /// - "600519"        → sh600519 (根据首位数字自动判断市场)
    /// - "sh600519"      → sh600519
    /// - "SH600519"      → sh600519
    /// - "600519.SH"     → sh600519 (Wind格式)
    pub fn from_raw(input: &str) -> Result<Self, AstockError> {
        let original = input.trim().to_uppercase();

        // 处理 Wind 格式: "600519.SH" → "SH600519"
        let normalized = if original.contains('.') {
            let parts: Vec<&str> = original.split('.').collect();
            if parts.len() != 2 {
                return Err(AstockError::InvalidCode(original));
            }
            format!("{}{}", parts[1], parts[0])
        } else {
            original
        };

        let (prefix, code) = if normalized.starts_with("SH")
            || normalized.starts_with("SZ")
            || normalized.starts_with("BJ")
        {
            // 已有前缀: "SH600519" → ("sh", "600519")
            let prefix = &normalized[..2];
            let code = &normalized[2..];
            if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
                return Err(AstockError::InvalidCode(normalized));
            }
            (prefix.to_lowercase(), code.to_string())
        } else {
            // 纯数字: "600519" → 根据首位判断市场
            let code = &normalized;
            if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
                return Err(AstockError::InvalidCode(normalized));
            }
            let prefix = match code.chars().next() {
                Some('6') => "sh",
                Some('0') | Some('3') => "sz",
                Some('8') | Some('4') => "bj",
                _ => return Err(AstockError::InvalidCode(normalized)),
            };
            (prefix.to_string(), code.to_string())
        };

        Ok(Self { prefix, code })
    }

    /// 用于腾讯/新浪API的格式: "sh600519"
    pub fn for_api(&self) -> String {
        format!("{}{}", self.prefix, self.code)
    }

    /// 6位纯数字代码: "600519"
    pub fn code(&self) -> &str {
        &self.code
    }

    /// 市场前缀: "sh", "sz", "bj"
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Wind格式显示: "600519.SH"
    pub fn display_wind(&self) -> String {
        format!("{}.{}", self.code, self.prefix.to_uppercase())
    }

    /// 东方财富API的secid格式: "1.600519" (1=沪, 0=深)
    pub fn for_eastmoney(&self) -> String {
        let market_id = match self.prefix.as_str() {
            "sh" => "1",
            "sz" => "0",
            "bj" => "0",
            _ => "1",
        };
        format!("{}.{}", market_id, self.code)
    }
}

impl fmt::Display for StockCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix, self.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw_pure_number() {
        assert_eq!(StockCode::from_raw("600519").unwrap().for_api(), "sh600519");
        assert_eq!(StockCode::from_raw("000001").unwrap().for_api(), "sz000001");
        assert_eq!(StockCode::from_raw("300750").unwrap().for_api(), "sz300750");
        assert_eq!(StockCode::from_raw("830799").unwrap().for_api(), "bj830799");
    }

    #[test]
    fn test_from_raw_with_prefix() {
        assert_eq!(StockCode::from_raw("sh600519").unwrap().for_api(), "sh600519");
        assert_eq!(StockCode::from_raw("SZ000001").unwrap().for_api(), "sz000001");
    }

    #[test]
    fn test_from_raw_wind_format() {
        assert_eq!(StockCode::from_raw("600519.SH").unwrap().for_api(), "sh600519");
        assert_eq!(StockCode::from_raw("000001.SZ").unwrap().for_api(), "sz000001");
    }

    #[test]
    fn test_invalid_code() {
        assert!(StockCode::from_raw("12345").is_err());
        assert!(StockCode::from_raw("abcdef").is_err());
        assert!(StockCode::from_raw("").is_err());
    }

    #[test]
    fn test_display_formats() {
        let code = StockCode::from_raw("600519").unwrap();
        assert_eq!(code.display_wind(), "600519.SH");
        assert_eq!(code.for_eastmoney(), "1.600519");
        assert_eq!(format!("{}", code), "sh600519");
    }
}
