pub mod encoding;
pub mod tencent;
pub mod tencent_kline;
pub mod eastmoney;

use crate::data::eastmoney::{
    build_financial_url, build_search_url, parse_financial_response, parse_search_response,
    FinancialInfo, StockSearchResult,
};
use crate::data::tencent::{build_quote_url, parse_tencent_quotes};
use crate::data::tencent_kline::{
    build_daily_kline_url, build_minute_kline_url, parse_kline_data, KlineType,
    TencentKlineResponse,
};
use crate::error::AstockError;
use crate::models::bar::Bar;
use crate::models::quote::Quote;
use crate::models::stock::StockCode;
use encoding::decode_gbk;
use reqwest::Client;

/// 数据客户端，封装所有API调用
pub struct DataClient {
    client: Client,
}

impl DataClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// 搜索股票
    pub async fn search_stocks(&self, query: &str, limit: u32) -> Result<Vec<StockSearchResult>, AstockError> {
        let url = build_search_url(query, limit);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("搜索请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_search_response(&text)
    }

    /// 获取实时行情
    pub async fn get_quotes(&self, codes: &[StockCode]) -> Result<Vec<Quote>, AstockError> {
        let url = build_quote_url(codes);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("行情请求失败: {}", e)))?;
        // 腾讯实时行情返回GBK编码
        let bytes = resp.bytes().await.map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        let text = decode_gbk(&bytes);
        let quotes = parse_tencent_quotes(&text, codes);
        Ok(quotes)
    }

    /// 获取K线数据
    pub async fn get_kline(
        &self,
        code: &StockCode,
        ktype: &KlineType,
        start_date: Option<&str>,
        end_date: Option<&str>,
        count: Option<u32>,
        qfq: bool,
    ) -> Result<Vec<Bar>, AstockError> {
        let url = if ktype.is_minute() {
            build_minute_kline_url(code, ktype, count)
        } else {
            build_daily_kline_url(code, ktype, start_date, end_date, count, qfq)
        };

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("K线请求失败: {}", e)))?;

        let kline_resp: TencentKlineResponse = resp
            .json()
            .await
            .map_err(|e| AstockError::ParseError(format!("K线响应解析失败: {}", e)))?;

        if kline_resp.code != 0 {
            return Err(AstockError::ApiError(
                kline_resp.msg.unwrap_or_else(|| "未知错误".into()),
            ));
        }

        parse_kline_data(&kline_resp, code, ktype, qfq)
    }

    /// 获取财务数据
    pub async fn get_financials(&self, code: &StockCode) -> Result<Vec<FinancialInfo>, AstockError> {
        let url = build_financial_url(code);
        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com")
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("财务数据请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_financial_response(&text)
    }
}
