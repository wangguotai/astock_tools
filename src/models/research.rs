/// 券商研报
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub code: String,
    pub name: String,
    /// 机构简称
    pub org_name: String,
    /// 发布日期
    pub publish_date: String,
    /// 评级 (买入/增持/中性/减持/卖出)
    pub rating: String,
    /// 评级变动
    pub rating_change: String,
    /// 今年EPS预测
    pub predict_this_year_eps: Option<f64>,
    /// 明年EPS预测
    pub predict_next_year_eps: Option<f64>,
    /// 报告标题
    pub title: String,
    /// 研究员
    pub researcher: String,
}
