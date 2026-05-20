/// 东方财富数据API客户端
///
/// 1. 股票搜索: https://searchapi.eastmoney.com/api/suggest/get
/// 2. 财务数据: https://datacenter.eastmoney.com/securities/api/data/v1/get
/// 3. 融资融券/龙虎榜/大宗交易/限售解禁/高管增减持: https://datacenter-web.eastmoney.com/api/data/v1/get
/// 4. 券商研报: https://reportapi.eastmoney.com/report/list
/// 5. 机构持仓: https://data.eastmoney.com/dataapi/zlsj/list
/// 6. 行业景气度: https://datacenter-web.eastmoney.com/api/data/v1/get (RPT_INDUSTRY_INDEX)
/// 7. 行业板块行情: https://push2.eastmoney.com/api/qt/clist/get
/// 8. 历史资金流: https://push2his.eastmoney.com/api/qt/moneyflow/hs/get
use crate::error::AstockError;
use crate::models::stock::StockCode;
use crate::models::{
    block_trade::BlockTrade,
    dragon_tiger::DragonTigerEntry,
    insider::{InstitutionHold, InsiderTrade, RestrictedShareRelease},
    margin::MarginData,
    research::ResearchReport,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

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

// ─── 融资融券 ───────────────────────────────────────────

/// 构建融资融券URL (个股，通过上海/深圳交易所API)
pub fn build_margin_url(code: &StockCode, _page_size: u32) -> String {
    // SSE 需要 detailsDate 参数才能按 stockCode 过滤
    // 用空日期让 API 返回最新日期数据
    if code.prefix() == "sh" {
        format!(
            "https://query.sse.com.cn/marketdata/tradedata/queryMargin.do?isPagination=true&tabType=mxtype&detailsDate=&stockCode={}&pageHelp.pageSize=5000&pageHelp.pageCount=50&pageHelp.pageNo=1&pageHelp.beginPage=1&pageHelp.cacheSize=1&pageHelp.endPage=50",
            code.code(),
        )
    } else {
        format!(
            "https://www.szse.cn/api/report/ShowReport/data?SHOWTYPE=JSON&CATALOGID=1837_xxpl&TABKEY=tab2&PAGENO=1&PAGECOUNT=5&txtZqdm={}",
            code.code()
        )
    }
}

/// 解析融资融券响应 (上交所格式)
pub fn parse_margin_sse_response(json: &str, code: &StockCode) -> Result<Vec<MarginData>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("融资融券数据解析失败: {}", e)))?;

    let data = resp
        .get("pageHelp")
        .and_then(|p| p.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("融资融券数据格式错误".into()))?;

    // SSE 会返回所有股票当日数据，需要过滤
    let target_code = code.code();
    let filtered: Vec<&serde_json::Value> = data
        .iter()
        .filter(|item| {
            item.get("stockCode")
                .and_then(|v| v.as_str())
                .map(|s| s == target_code)
                .unwrap_or(false)
        })
        .collect();

    Ok(filtered
        .iter()
        .filter_map(|item| {
            let date_raw = item.get("opDate").and_then(|v| v.as_str())?;
            let trade_date = if date_raw.len() == 8 {
                format!("{}-{}-{}", &date_raw[0..4], &date_raw[4..6], &date_raw[6..8])
            } else {
                date_raw.to_string()
            };
            Some(MarginData {
                code: code.clone(),
                trade_date,
                fin_balance: parse_decimal(item, "rzye"),
                fin_buy_amt: parse_decimal(item, "rzmre"),
                loan_balance: Decimal::ZERO, // SSE 个股不直接给融券余额
                loan_sell_vol: parse_decimal(item, "rqmcl"),
            })
        })
        .collect())
}

/// 解析融资融券响应 (深交所格式)
pub fn parse_margin_szse_response(json: &str, code: &StockCode) -> Result<Vec<MarginData>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("融资融券数据解析失败: {}", e)))?;

    let data = resp
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("融资融券数据格式错误".into()))?;

    Ok(data
        .iter()
        .filter_map(|item| {
            // SZSE 不返回日期字段，用空字符串，由调用方填充
            let trade_date = String::new();
            Some(MarginData {
                code: code.clone(),
                trade_date,
                // SZSE 返回单位是亿元，可能含逗号
                fin_balance: parse_decimal_str(item, "jrrzye") * Decimal::from(100000000),
                fin_buy_amt: parse_decimal_str(item, "jrrzmr") * Decimal::from(100000000),
                loan_balance: parse_decimal_str(item, "jrrjye") * Decimal::from(100000000),
                loan_sell_vol: parse_decimal_str(item, "jrrjmc") * Decimal::from(100000000),
            })
        })
        .collect())
}

// ─── 券商研报 ───────────────────────────────────────────

/// 构建券商研报URL
pub fn build_research_url(code: &StockCode, page_size: u32) -> String {
    format!(
        "https://reportapi.eastmoney.com/report/list?industryCode=*&pageSize={}&industry=*&rating=*&ratingChange=*&beginTime=&endTime=&pageNo=1&fields=&qType=0&orgCode=&code={}&rcode=&p=1&pageNum=1&pageNumber=1",
        page_size,
        code.code()
    )
}

/// 解析券商研报响应
pub fn parse_research_response(json: &str) -> Result<Vec<ResearchReport>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("研报数据解析失败: {}", e)))?;

    let data = resp
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("研报数据格式错误".into()))?;

    Ok(data
        .iter()
        .map(|item| ResearchReport {
            code: item.get("stockCode").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: item.get("stockName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            org_name: item.get("orgSName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            publish_date: item.get("publishDate").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            rating: item.get("emRatingName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            rating_change: item.get("ratingChange").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            predict_this_year_eps: item.get("predictThisYearEps").and_then(|v| v.as_f64()),
            predict_next_year_eps: item.get("predictNextYearEps").and_then(|v| v.as_f64()),
            title: item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            researcher: item.get("researcher").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
        .collect())
}

// ─── 龙虎榜 ─────────────────────────────────────────────

/// 构建龙虎榜URL (个股)
pub fn build_dragon_tiger_url(code: &StockCode, page_size: u32) -> String {
    format!(
        "https://datacenter-web.eastmoney.com/api/data/v1/get?reportName=RPT_DAILYBILLBOARD_DETAILSNEW&columns=ALL&filter=(SECURITY_CODE%3D%22{}%22)&pageSize={}&sortTypes=-1&sortColumns=TRADE_DATE&source=WEB&client=WEB",
        code.code(),
        page_size
    )
}

/// 解析龙虎榜响应
pub fn parse_dragon_tiger_response(json: &str) -> Result<Vec<DragonTigerEntry>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("龙虎榜数据解析失败: {}", e)))?;

    let data = resp
        .get("result")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("龙虎榜数据格式错误".into()))?;

    Ok(data
        .iter()
        .map(|item| DragonTigerEntry {
            code: item.get("SECURITY_CODE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: item.get("SECURITY_NAME_ABBR").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            trade_date: item.get("TRADE_DATE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            close_price: parse_decimal(item, "CLOSE_PRICE"),
            change_rate: parse_decimal(item, "CHANGE_RATE"),
            explanation: item.get("EXPLAIN").or_else(|| item.get("EXPLANATION")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
            buy_amt: parse_decimal(item, "BILLBOARD_BUY_AMT"),
            sell_amt: parse_decimal(item, "BILLBOARD_SELL_AMT"),
            net_buy_amt: parse_decimal(item, "BILLBOARD_NET_AMT"),
            deal_amt: parse_decimal(item, "BILLBOARD_DEAL_AMT"),
        })
        .collect())
}

// ─── 大宗交易 ───────────────────────────────────────────

/// 构建大宗交易URL (个股)
pub fn build_block_trade_url(code: &StockCode, page_size: u32) -> String {
    format!(
        "https://datacenter-web.eastmoney.com/api/data/v1/get?reportName=RPT_DATA_BLOCKTRADE&columns=ALL&filter=(SECURITY_CODE%3D%22{}%22)&pageSize={}&sortTypes=-1&sortColumns=TRADE_DATE&source=WEB&client=WEB",
        code.code(),
        page_size
    )
}

/// 解析大宗交易响应
pub fn parse_block_trade_response(json: &str) -> Result<Vec<BlockTrade>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("大宗交易数据解析失败: {}", e)))?;

    let data = resp
        .get("result")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("大宗交易数据格式错误".into()))?;

    Ok(data
        .iter()
        .map(|item| BlockTrade {
            code: item.get("SECURITY_CODE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: item.get("SECURITY_NAME_ABBR").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            trade_date: item.get("TRADE_DATE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            deal_price: parse_decimal(item, "DEAL_PRICE"),
            deal_volume: parse_decimal(item, "DEAL_VOLUME"),
            deal_amt: parse_decimal(item, "DEAL_AMT"),
            premium_ratio: item.get("PREMIUM_RATIO").and_then(|v| v.as_f64()).map(|v| Decimal::from_f64_retain(v)).flatten(),
            buyer_name: item.get("BUYER_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            seller_name: item.get("SELLER_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
        .collect())
}

// ─── 限售股解禁 ─────────────────────────────────────────

/// 构建限售股解禁URL (个股)
pub fn build_restricted_release_url(code: &StockCode, page_size: u32) -> String {
    format!(
        "https://datacenter-web.eastmoney.com/api/data/v1/get?reportName=RPT_LIFT_STAGE&columns=ALL&filter=(SECURITY_CODE%3D%22{}%22)&pageSize={}&sortTypes=-1&sortColumns=FREE_DATE&source=WEB&client=WEB",
        code.code(),
        page_size
    )
}

/// 解析限售股解禁响应
pub fn parse_restricted_release_response(json: &str) -> Result<Vec<RestrictedShareRelease>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("解禁数据解析失败: {}", e)))?;

    // 检查是否有数据
    let result = resp.get("result");
    if result.is_none() || result.unwrap().is_null() {
        return Ok(vec![]);
    }

    let data = result
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("解禁数据格式错误".into()))?;

    Ok(data
        .iter()
        .map(|item| RestrictedShareRelease {
            code: item.get("SECURITY_CODE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: item.get("SECURITY_NAME_ABBR").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            free_date: item.get("FREE_DATE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            free_shares: parse_decimal(item, "CURRENT_FREE_SHARES"),
            free_market_cap: parse_decimal(item, "LIFT_MARKET_CAP"),
            free_type: item.get("FREE_SHARES_TYPE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
        .collect())
}

// ─── 高管增减持 ─────────────────────────────────────────

/// 构建高管增减持URL (个股)
pub fn build_insider_trade_url(code: &StockCode, page_size: u32) -> String {
    format!(
        "https://datacenter-web.eastmoney.com/api/data/v1/get?reportName=RPT_SHARE_HOLDER_INCREASE&columns=ALL&filter=(SECURITY_CODE%3D%22{}%22)&pageSize={}&sortTypes=-1&sortColumns=END_DATE&source=WEB&client=WEB",
        code.code(),
        page_size
    )
}

/// 解析高管增减持响应
pub fn parse_insider_trade_response(json: &str) -> Result<Vec<InsiderTrade>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("高管增减持数据解析失败: {}", e)))?;

    let data = resp
        .get("result")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("高管增减持数据格式错误".into()))?;

    Ok(data
        .iter()
        .map(|item| InsiderTrade {
            code: item.get("SECURITY_CODE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: item.get("SECURITY_NAME_ABBR").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            holder_name: item.get("HOLDER_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            direction: item.get("DIRECTION").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            change_num: parse_decimal(item, "CHANGE_NUM"),
            change_rate: item.get("CHANGE_RATE").and_then(|v| v.as_f64()).map(|v| Decimal::from_f64_retain(v)).flatten(),
            end_date: item.get("END_DATE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            after_holder_num: item.get("AFTER_HOLDER_NUM").and_then(|v| v.as_f64()).map(|v| Decimal::from_f64_retain(v)).flatten(),
        })
        .collect())
}

// ─── 机构持仓 ───────────────────────────────────────────

/// 构建机构持仓URL
pub fn build_institution_hold_url(code: &StockCode, report_date: &str, page_size: u32) -> String {
    format!(
        "https://data.eastmoney.com/dataapi/zlsj/list?date={}&type=1&zjc=0&sortField=HOULD_NUM&sortDirec=1&pageNum=1&pageSize={}&code={}",
        report_date,
        page_size,
        code.code()
    )
}

/// 解析机构持仓响应
pub fn parse_institution_hold_response(json: &str) -> Result<Vec<InstitutionHold>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("机构持仓数据解析失败: {}", e)))?;

    let data = resp
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("机构持仓数据格式错误".into()))?;

    Ok(data
        .iter()
        .filter_map(|item| {
            let code = item.get("SECURITY_CODE").and_then(|v| v.as_str())?;
            Some(InstitutionHold {
                code: code.to_string(),
                name: item.get("SECURITY_NAME_ABBR").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                org_type: item.get("ORG_TYPE_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                hold_num: item.get("HOULD_NUM").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                total_shares: parse_decimal(item, "TOTAL_SHARES"),
                hold_value: parse_decimal(item, "HOLD_VALUE"),
                free_ratio: item.get("FREESHARES_RATIO").and_then(|v| v.as_f64()).map(|v| Decimal::from_f64_retain(v)).flatten(),
                hold_change: item.get("HOLDCHA").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                report_date: item.get("REPORT_DATE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            })
        })
        .collect())
}

// ─── 行业景气度 ─────────────────────────────────────────

/// 行业景气度指标
#[derive(Debug, Clone)]
pub struct IndustryIndex {
    /// 行业板块名称
    pub board_name: String,
    /// 指标名称
    pub indicator_name: String,
    /// 指标值
    pub indicator_value: String,
    /// 日变化率%
    pub change_rate: Option<f64>,
    /// 3月变化率%
    pub change_rate_3m: Option<f64>,
    /// 1年变化率%
    pub change_rate_1y: Option<f64>,
}

/// 构建行业景气度URL
pub fn build_industry_index_url(page_size: u32) -> String {
    format!(
        "https://datacenter-web.eastmoney.com/api/data/v1/get?reportName=RPT_INDUSTRY_INDEX&columns=ALL&filter=(IS_NEWEST%3D%22True%22)&pageSize={}&sortTypes=-1&sortColumns=CHANGE_RATE&source=WEB&client=WEB",
        page_size
    )
}

/// 解析行业景气度响应
pub fn parse_industry_index_response(json: &str) -> Result<Vec<IndustryIndex>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("行业景气度数据解析失败: {}", e)))?;

    let data = resp
        .get("result")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("行业景气度数据格式错误".into()))?;

    Ok(data
        .iter()
        .map(|item| IndustryIndex {
            board_name: item.get("BOARD_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            indicator_name: item.get("INDICATOR_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            indicator_value: item.get("INDICATOR_VALUE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            change_rate: item.get("CHANGE_RATE").and_then(|v| v.as_f64()),
            change_rate_3m: item.get("CHANGERATE_3M").and_then(|v| v.as_f64()),
            change_rate_1y: item.get("CHANGERATE_1Y").and_then(|v| v.as_f64()),
        })
        .collect())
}

// ─── 行业板块行情 ───────────────────────────────────────

/// 行业板块行情
#[derive(Debug, Clone)]
pub struct IndustryQuote {
    /// 板块代码
    pub code: String,
    /// 板块名称
    pub name: String,
    /// 最新价
    pub price: Option<f64>,
    /// 涨跌幅%
    pub change_pct: Option<f64>,
    /// 涨跌额
    pub change_amt: Option<f64>,
}

/// 构建行业板块行情URL
pub fn build_industry_quote_url(page_size: u32) -> String {
    format!(
        "https://push2.eastmoney.com/api/qt/clist/get?pn=1&pz={}&po=1&np=1&fltt=2&invt=2&fid=f3&fs=m:90+t:2&fields=f2,f3,f4,f12,f14",
        page_size
    )
}

/// 解析行业板块行情响应
pub fn parse_industry_quote_response(json: &str) -> Result<Vec<IndustryQuote>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("行业行情数据解析失败: {}", e)))?;

    let data = resp
        .get("data")
        .and_then(|d| d.get("diff"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| AstockError::ParseError("行业行情数据格式错误".into()))?;

    Ok(data
        .iter()
        .map(|item| IndustryQuote {
            code: item.get("f12").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: item.get("f14").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            price: item.get("f2").and_then(|v| v.as_f64()),
            change_pct: item.get("f3").and_then(|v| v.as_f64()),
            change_amt: item.get("f4").and_then(|v| v.as_f64()),
        })
        .collect())
}

// ─── 历史资金流 ─────────────────────────────────────────

/// 历史资金流数据 (按日)
#[derive(Debug, Clone)]
pub struct MoneyFlowHistory {
    /// 日期
    pub date: String,
    /// 主力净流入
    pub main_net_inflow: Decimal,
    /// 小单净流入
    pub small_net_inflow: Decimal,
    /// 中单净流入
    pub medium_net_inflow: Decimal,
    /// 大单净流入
    pub large_net_inflow: Decimal,
}

/// 构建历史资金流URL
pub fn build_money_flow_history_url(code: &StockCode, days: u32) -> String {
    format!(
        "https://push2his.eastmoney.com/api/qt/stock/fflow/kline/get?secid={}&fields1=f1,f2,f3,f7&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61,f62,f63,f64,f65&klt=101&lmt={}",
        code.for_eastmoney(),
        days
    )
}

/// 解析历史资金流响应
pub fn parse_money_flow_history_response(json: &str) -> Result<Vec<MoneyFlowHistory>, AstockError> {
    let resp: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AstockError::ParseError(format!("资金流数据解析失败: {}", e)))?;

    let data = resp
        .get("data")
        .and_then(|d| d.as_object())
        .ok_or_else(|| AstockError::ParseError("资金流数据格式错误".into()))?;

    // klines 数组，每行逗号分隔: 日期,主力净流入,小单净流入,中单净流入,大单净流入,特大单净流入
    let klines = data
        .get("klines")
        .and_then(|k| k.as_array())
        .ok_or_else(|| AstockError::ParseError("资金流klines格式错误".into()))?;

    Ok(klines
        .iter()
        .filter_map(|line| {
            let s = line.as_str()?;
            let fields: Vec<&str> = s.split(',').collect();
            if fields.len() < 5 {
                return None;
            }
            Some(MoneyFlowHistory {
                date: fields[0].to_string(),
                main_net_inflow: Decimal::from_str(fields[1]).ok()?,
                small_net_inflow: Decimal::from_str(fields[2]).ok()?,
                medium_net_inflow: Decimal::from_str(fields[3]).ok()?,
                large_net_inflow: Decimal::from_str(fields[4]).ok()?,
            })
        })
        .collect())
}

// ─── 通用辅助函数 ───────────────────────────────────────

/// 从 JSON Value 中解析 Decimal，解析失败返回 0
fn parse_decimal(item: &serde_json::Value, key: &str) -> Decimal {
    item.get(key)
        .and_then(|v| v.as_f64())
        .and_then(|v| Decimal::from_f64_retain(v))
        .unwrap_or(Decimal::ZERO)
}

/// 从 JSON Value 中解析字符串形式的 Decimal（可能含逗号），解析失败返回 0
fn parse_decimal_str(item: &serde_json::Value, key: &str) -> Decimal {
    item.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.replace(',', ""))
        .and_then(|s| Decimal::from_str(&s).ok())
        .or_else(|| item.get(key).and_then(|v| v.as_f64()).and_then(|v| Decimal::from_f64_retain(v)))
        .unwrap_or(Decimal::ZERO)
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

    #[test]
    fn test_build_margin_url() {
        let code = StockCode::from_raw("600519").unwrap();
        let url = build_margin_url(&code, 20);
        assert!(url.contains("600519"));
        assert!(url.contains("query.sse.com.cn"));

        let code_sz = StockCode::from_raw("000001").unwrap();
        let url_sz = build_margin_url(&code_sz, 20);
        assert!(url_sz.contains("000001"));
        assert!(url_sz.contains("szse.cn"));
    }

    #[test]
    fn test_build_research_url() {
        let code = StockCode::from_raw("600519").unwrap();
        let url = build_research_url(&code, 10);
        assert!(url.contains("600519"));
        assert!(url.contains("reportapi.eastmoney.com"));
    }

    #[test]
    fn test_build_dragon_tiger_url() {
        let code = StockCode::from_raw("000001").unwrap();
        let url = build_dragon_tiger_url(&code, 20);
        assert!(url.contains("000001"));
        assert!(url.contains("RPT_DAILYBILLBOARD_DETAILSNEW"));
    }

    #[test]
    fn test_build_block_trade_url() {
        let code = StockCode::from_raw("600519").unwrap();
        let url = build_block_trade_url(&code, 20);
        assert!(url.contains("600519"));
        assert!(url.contains("RPT_DATA_BLOCKTRADE"));
    }

    #[test]
    fn test_build_insider_trade_url() {
        let code = StockCode::from_raw("600519").unwrap();
        let url = build_insider_trade_url(&code, 20);
        assert!(url.contains("600519"));
        assert!(url.contains("RPT_SHARE_HOLDER_INCREASE"));
    }

    #[test]
    fn test_build_money_flow_history_url() {
        let code = StockCode::from_raw("600519").unwrap();
        let url = build_money_flow_history_url(&code, 20);
        assert!(url.contains("1.600519"));
        assert!(url.contains("push2his.eastmoney.com"));
    }
}
