/// 东方财富数据API客户端
///
/// 1. 股票搜索: https://searchapi.eastmoney.com/api/suggest/get
/// 2. 财务数据: https://datacenter.eastmoney.com/securities/api/data/v1/get
use crate::error::AstockError;
use crate::models::stock::StockCode;
use serde::Deserialize;

/// 股票搜索结果
#[derive(Debug, Clone, Deserialize)]
pub struct StockSearchResult {
    pub code: String,
    pub name: String,
    pub pinyin: String,
    pub market: String, // "沪A", "深A", etc.
}

/// 东方财富搜索API响应
#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(rename = "QuotationCodeTable")]
    quotation_code_table: SearchData,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    #[allow(non_snake_case)]
    Data: Option<Vec<SearchItem>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct SearchItem {
    Code: String,
    Name: String,
    PinYin: String,
    SecurityTypeName: String,
    #[allow(dead_code)]
    MktNum: String,
}

/// 构建搜索URL
pub fn build_search_url(query: &str, count: u32) -> String {
    format!(
        "https://searchapi.eastmoney.com/api/suggest/get?input={}&type=14&token=D43BF722C8E33BDC906FB84D85E326E8&count={}",
        urlencoding::encode(query),
        count
    )
}

/// 解析搜索结果
pub fn parse_search_response(json: &str) -> Result<Vec<StockSearchResult>, AstockError> {
    let resp: SearchResponse = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("搜索结果解析失败: {}", e)))?;

    let items = resp.quotation_code_table.Data.unwrap_or_default();

    Ok(items
        .into_iter()
        .filter(|item| {
            // 只保留A股
            item.SecurityTypeName.contains('A')
        })
        .map(|item| StockSearchResult {
            code: item.Code.clone(),
            name: item.Name,
            pinyin: item.PinYin,
            market: item.SecurityTypeName,
        })
        .collect())
}

/// 构建财务数据请求URL
pub fn build_financial_url(code: &StockCode) -> String {
    format!(
        "https://datacenter.eastmoney.com/securities/api/data/v1/get?reportName=RPT_F10_FINANCE_MAINFINADATA&columns=ALL&filter=(SECURITY_CODE%3D%22{}%22)&pageNumber=1&pageSize=4&sortTypes=-1&sortColumns=NOTICE_DATE",
        code.code()
    )
}

/// 财务数据简要信息
#[derive(Debug, Clone)]
pub struct FinancialInfo {
    pub name: String,
    pub report_date: String,
    pub report_type: String,
    pub eps: Option<f64>,              // 每股收益
    pub bvps: Option<f64>,             // 每股净资产
    pub roe: Option<f64>,              // 净资产收益率(%)
    pub gross_margin: Option<f64>,     // 毛利率(%)
    pub net_margin: Option<f64>,       // 净利率(%)
    pub revenue_yoy: Option<f64>,      // 营收同比增长(%)
    pub profit_yoy: Option<f64>,       // 净利润同比增长(%)
    pub debt_ratio: Option<f64>,       // 资产负债率(%)
}

/// 解析财务数据响应
pub fn parse_financial_response(json: &str) -> Result<Vec<FinancialInfo>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("财务数据解析失败: {}", e)))?;

    let data = resp
        .get("result")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("财务数据格式错误".into()))?;

    Ok(data
        .iter()
        .map(|item| FinancialInfo {
            name: item.get("SECURITY_NAME_ABBR").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            report_date: item.get("REPORT_DATE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            report_type: item.get("REPORT_TYPE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            eps: item.get("EPSJB").and_then(|v| v.as_f64()),
            bvps: item.get("BPS").and_then(|v| v.as_f64()),
            roe: item.get("ROEJQ").and_then(|v| v.as_f64()),
            gross_margin: item.get("XSMLL").and_then(|v| v.as_f64()),
            net_margin: item.get("XSJLL").and_then(|v| v.as_f64()),
            revenue_yoy: item.get("TOTALOPERATEREVETZ").and_then(|v| v.as_f64()),
            profit_yoy: item.get("PARENTNETPROFITTZ").and_then(|v| v.as_f64()),
            debt_ratio: item.get("ZCFZL").and_then(|v| v.as_f64()),
        })
        .collect())
}

// URL encoding helper - handles UTF-8 multi-byte characters correctly
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for byte in s.as_bytes() {
            match *byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(*byte as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_search_url() {
        let url = build_search_url("茅台", 5);
        assert!(url.contains("searchapi.eastmoney.com"));
        assert!(url.contains("%E8%8C%85%E5%8F%B0"));
    }

    #[test]
    fn test_build_financial_url() {
        let code = StockCode::from_raw("600519").unwrap();
        let url = build_financial_url(&code);
        assert!(url.contains("600519"));
        assert!(url.contains("RPT_F10_FINANCE_MAINFINADATA"));
    }
}
