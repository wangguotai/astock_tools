/// 腾讯K线数据API客户端
///
/// API: https://ifzq.gtimg.cn/appstock/app/fqkline/get
/// 日线/周线/月线: ?param=sh600519,day,2024-01-01,2025-12-01,120,qfq
/// 分钟线: https://ifzq.gtimg.cn/appstock/app/kline/mkline?param=sh600519,m5,,10
///
/// ⚠️ 重要: 腾讯K线字段顺序是 [date, open, close, high, low, volume]
/// 不是常见的 [date, open, high, low, close, volume]！close在high前面
use crate::error::AstockError;
use crate::models::bar::Bar;
use crate::models::stock::StockCode;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

/// 腾讯K线API的JSON响应结构
#[derive(Debug, Deserialize)]
pub struct TencentKlineResponse {
    pub code: i32,
    pub msg: Option<String>,
    pub data: serde_json::Value,
}

/// K线类型
#[derive(Debug, Clone)]
pub enum KlineType {
    Day,
    Week,
    Month,
    Min5,
    Min15,
    Min30,
    Min60,
}

impl KlineType {
    /// 转为腾讯API的period参数
    pub fn to_period(&self) -> &str {
        match self {
            KlineType::Day => "day",
            KlineType::Week => "week",
            KlineType::Month => "month",
            KlineType::Min5 => "m5",
            KlineType::Min15 => "m15",
            KlineType::Min30 => "m30",
            KlineType::Min60 => "m60",
        }
    }

    /// 是否为分钟线
    pub fn is_minute(&self) -> bool {
        matches!(self, KlineType::Min5 | KlineType::Min15 | KlineType::Min30 | KlineType::Min60)
    }

    /// 从字符串解析
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "day" | "d" | "daily" => Some(KlineType::Day),
            "week" | "w" | "weekly" => Some(KlineType::Week),
            "month" | "m" | "monthly" => Some(KlineType::Month),
            "m5" | "5" | "min5" => Some(KlineType::Min5),
            "m15" | "15" | "min15" => Some(KlineType::Min15),
            "m30" | "30" | "min30" => Some(KlineType::Min30),
            "m60" | "60" | "min60" => Some(KlineType::Min60),
            _ => None,
        }
    }
}

/// 构建日线/周线/月线请求URL
pub fn build_daily_kline_url(
    code: &StockCode,
    ktype: &KlineType,
    start_date: Option<&str>,
    end_date: Option<&str>,
    count: Option<u32>,
    qfq: bool,
) -> String {
    let period = ktype.to_period();
    let start = start_date.unwrap_or("");
    let end = end_date.unwrap_or("");
    let count_str = count.map(|n| n.to_string()).unwrap_or_default();
    let fq = if qfq { "qfq" } else { "" };

    format!(
        "https://ifzq.gtimg.cn/appstock/app/fqkline/get?param={},{},{},{},{},{}",
        code.for_api(),
        period,
        start,
        end,
        count_str,
        fq
    )
}

/// 构建分钟线请求URL
pub fn build_minute_kline_url(
    code: &StockCode,
    ktype: &KlineType,
    count: Option<u32>,
) -> String {
    let period = ktype.to_period();
    let count_str = count.map(|n| n.to_string()).unwrap_or_default();

    format!(
        "https://ifzq.gtimg.cn/appstock/app/kline/mkline?param={},{},,{}",
        code.for_api(),
        period,
        count_str
    )
}

/// 解析K线响应中的数据数组
///
/// 日线/周线/月线: data.code.qfqday 或 data.code.day
/// 分钟线: data.code.m5/m15/m30/m60
///
/// 字段顺序: [date, open, close, high, low, volume]
/// ⚠️ close在high前面！
pub fn parse_kline_data(
    response: &TencentKlineResponse,
    code: &StockCode,
    ktype: &KlineType,
    qfq: bool,
) -> Result<Vec<Bar>, AstockError> {
    let code_key = code.for_api(); // "sh600519"
    let data_obj = response
        .data
        .get(&code_key)
        .ok_or_else(|| AstockError::ParseError("响应中找不到股票数据".into()))?;

    // 根据K线类型和复权选项确定JSON字段名
    let key = if ktype.is_minute() {
        ktype.to_period().to_string()
    } else if qfq {
        format!("qfq{}", ktype.to_period())
    } else {
        ktype.to_period().to_string()
    };

    let kline_array = data_obj
        .get(&key)
        .ok_or_else(|| AstockError::ParseError(format!("响应中找不到K线数据字段: {}", key)))?;

    let arr = kline_array
        .as_array()
        .ok_or_else(|| AstockError::ParseError("K线数据不是数组".into()))?;

    let mut bars = Vec::new();
    for item in arr {
        let fields = item
            .as_array()
            .ok_or_else(|| AstockError::ParseError("K线条目不是数组".into()))?;

        if fields.len() < 6 {
            continue;
        }

        // 字段顺序: [date, open, close, high, low, volume]
        // ⚠️ 注意: close在high前面！
        let date_str = fields[0].as_str().unwrap_or("");
        let date = parse_date(date_str);
        let open = parse_decimal_value(&fields[1]);
        let close = parse_decimal_value(&fields[2]);
        let high = parse_decimal_value(&fields[3]);
        let low = parse_decimal_value(&fields[4]);
        let volume = parse_i64_value(&fields[5]);

        if let Some(date) = date {
            bars.push(Bar {
                code: code.clone(),
                date,
                open,
                close,
                high,
                low,
                volume,
                turnover: Decimal::ZERO, // 腾讯K线不直接返回成交额
            });
        }
    }

    Ok(bars)
}

/// 解析日期字符串
/// 日线: "2024-01-02"
/// 分钟线: "202401021500"
fn parse_date(s: &str) -> Option<NaiveDate> {
    if s.contains('-') {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
    } else if s.len() >= 8 {
        // 分钟线格式: "202401021500"，取前8位作为日期
        NaiveDate::parse_from_str(&s[..8], "%Y%m%d").ok()
    } else {
        None
    }
}

fn parse_decimal_value(val: &serde_json::Value) -> Decimal {
    match val {
        serde_json::Value::String(s) => Decimal::from_str(s).unwrap_or(Decimal::ZERO),
        serde_json::Value::Number(n) => {
            Decimal::from_str(&n.to_string()).unwrap_or(Decimal::ZERO)
        }
        _ => Decimal::ZERO,
    }
}

fn parse_i64_value(val: &serde_json::Value) -> i64 {
    match val {
        serde_json::Value::String(s) => s.parse().unwrap_or(0),
        serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kline_type_parsing() {
        assert!(matches!(KlineType::from_str_opt("day"), Some(KlineType::Day)));
        assert!(matches!(KlineType::from_str_opt("m5"), Some(KlineType::Min5)));
        assert!(matches!(KlineType::from_str_opt("week"), Some(KlineType::Week)));
        assert!(KlineType::from_str_opt("invalid").is_none());
    }

    #[test]
    fn test_build_daily_url() {
        let code = StockCode::from_raw("600519").unwrap();
        let url = build_daily_kline_url(
            &code,
            &KlineType::Day,
            Some("2024-01-01"),
            None,
            Some(120),
            true,
        );
        assert!(url.contains("sh600519"));
        assert!(url.contains("day"));
        assert!(url.contains("qfq"));
    }

    #[test]
    fn test_build_minute_url() {
        let code = StockCode::from_raw("600519").unwrap();
        let url = build_minute_kline_url(&code, &KlineType::Min5, Some(10));
        assert!(url.contains("sh600519"));
        assert!(url.contains("m5"));
    }
}
