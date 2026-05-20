pub mod encoding;
pub mod tencent;
pub mod tencent_kline;
pub mod eastmoney;

use crate::data::eastmoney::{
    build_financial_url, build_search_url, parse_financial_response, parse_search_response,
    FinancialInfo, StockSearchResult,
    build_margin_url, parse_margin_sse_response, parse_margin_szse_response,
    build_research_url, parse_research_response,
    build_dragon_tiger_url, parse_dragon_tiger_response,
    build_block_trade_url, parse_block_trade_response,
    build_restricted_release_url, parse_restricted_release_response,
    build_insider_trade_url, parse_insider_trade_response,
    build_institution_hold_url, parse_institution_hold_response,
    build_industry_index_url, parse_industry_index_response,
    build_industry_quote_url, parse_industry_quote_response,
    build_money_flow_history_url, parse_money_flow_history_response,
    IndustryIndex, IndustryQuote, MoneyFlowHistory,
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
use crate::models::{
    block_trade::BlockTrade,
    dragon_tiger::DragonTigerEntry,
    insider::{InstitutionHold, InsiderTrade, RestrictedShareRelease},
    margin::MarginData,
    research::ResearchReport,
};
use encoding::decode_gbk;
use reqwest::Client;
use chrono::Datelike;

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

    /// 获取融资融券数据
    pub async fn get_margin(&self, code: &StockCode, _days: u32) -> Result<Vec<MarginData>, AstockError> {
        if code.prefix() == "sh" {
            let url = build_margin_url(code, 20);
            let resp = self
                .client
                .get(&url)
                .header("Referer", "https://www.sse.com.cn/")
                .send()
                .await
                .map_err(|e| AstockError::ApiError(format!("融资融券请求失败: {}", e)))?;
            let text = resp
                .text()
                .await
                .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
            parse_margin_sse_response(&text, code)
        } else {
            // SZSE 需要传日期参数，尝试当天和前一天
            let today = chrono::Local::now();
            for day_offset in 0..5 {
                let date = (today - chrono::Duration::days(day_offset)).date_naive();
                // 跳过周末
                let weekday = date.weekday().num_days_from_monday();
                if weekday >= 5 {
                    continue;
                }
                let date_str = date.format("%Y-%m-%d").to_string();
                let url = format!(
                    "https://www.szse.cn/api/report/ShowReport/data?SHOWTYPE=JSON&CATALOGID=1837_xxpl&TABKEY=tab2&PAGENO=1&PAGECOUNT=5&txtDate={}&txtZqdm={}",
                    date_str,
                    code.code()
                );
                let resp = self
                    .client
                    .get(&url)
                    .header("Referer", "https://www.szse.cn/")
                    .send()
                    .await
                    .map_err(|e| AstockError::ApiError(format!("融资融券请求失败: {}", e)))?;
                let text = resp
                    .text()
                    .await
                    .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
                let mut result = parse_margin_szse_response(&text, code)?;
                if !result.is_empty() {
                    for m in &mut result {
                        if m.trade_date.is_empty() {
                            m.trade_date = date_str.clone();
                        }
                    }
                    return Ok(result);
                }
            }
            Ok(vec![])
        }
    }

    /// 获取券商研报
    pub async fn get_research_reports(&self, code: &StockCode, size: u32) -> Result<Vec<ResearchReport>, AstockError> {
        let url = build_research_url(code, size);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("研报请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_research_response(&text)
    }

    /// 获取龙虎榜数据
    pub async fn get_dragon_tiger(&self, code: &StockCode, size: u32) -> Result<Vec<DragonTigerEntry>, AstockError> {
        let url = build_dragon_tiger_url(code, size);
        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com")
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("龙虎榜请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_dragon_tiger_response(&text)
    }

    /// 获取大宗交易数据
    pub async fn get_block_trades(&self, code: &StockCode, size: u32) -> Result<Vec<BlockTrade>, AstockError> {
        let url = build_block_trade_url(code, size);
        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com")
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("大宗交易请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_block_trade_response(&text)
    }

    /// 获取限售股解禁数据
    pub async fn get_restricted_releases(&self, code: &StockCode, size: u32) -> Result<Vec<RestrictedShareRelease>, AstockError> {
        let url = build_restricted_release_url(code, size);
        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com")
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("解禁数据请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_restricted_release_response(&text)
    }

    /// 获取高管增减持数据
    pub async fn get_insider_trades(&self, code: &StockCode, size: u32) -> Result<Vec<InsiderTrade>, AstockError> {
        let url = build_insider_trade_url(code, size);
        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com")
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("高管增减持请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_insider_trade_response(&text)
    }

    /// 获取机构持仓数据
    pub async fn get_institution_holds(&self, code: &StockCode, report_date: &str, size: u32) -> Result<Vec<InstitutionHold>, AstockError> {
        let url = build_institution_hold_url(code, report_date, size);
        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com")
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("机构持仓请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_institution_hold_response(&text)
    }

    /// 获取行业景气度指标
    pub async fn get_industry_index(&self, size: u32) -> Result<Vec<IndustryIndex>, AstockError> {
        let url = build_industry_index_url(size);
        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com")
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("行业景气度请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_industry_index_response(&text)
    }

    /// 获取行业板块行情
    pub async fn get_industry_quotes(&self, size: u32) -> Result<Vec<IndustryQuote>, AstockError> {
        let url = build_industry_quote_url(size);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("行业行情请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_industry_quote_response(&text)
    }

    /// 获取历史资金流数据 (按日)
    pub async fn get_money_flow_history(&self, code: &StockCode, days: u32) -> Result<Vec<MoneyFlowHistory>, AstockError> {
        let url = build_money_flow_history_url(code, days);
        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com")
            .send()
            .await
            .map_err(|e| AstockError::ApiError(format!("资金流请求失败: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| AstockError::ApiError(format!("读取响应失败: {}", e)))?;
        parse_money_flow_history_response(&text)
    }
}
