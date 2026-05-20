/// 高管增减持 + 限售股解禁 + 机构持仓
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 高管增减持
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderTrade {
    pub code: String,
    pub name: String,
    /// 变动人
    pub holder_name: String,
    /// 增持/减持
    pub direction: String,
    /// 变动数量(万股)
    pub change_num: Decimal,
    /// 变动比例%
    pub change_rate: Option<Decimal>,
    /// 变动截止日
    pub end_date: String,
    /// 变动后持股数
    pub after_holder_num: Option<Decimal>,
}

/// 限售股解禁
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestrictedShareRelease {
    pub code: String,
    pub name: String,
    /// 解禁日期
    pub free_date: String,
    /// 解禁数量(万股)
    pub free_shares: Decimal,
    /// 解禁市值(万元)
    pub free_market_cap: Decimal,
    /// 解禁类型
    pub free_type: String,
}

/// 机构持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionHold {
    pub code: String,
    pub name: String,
    /// 机构类型 (基金/QFII/社保/券商/保险/信托)
    pub org_type: String,
    /// 持有机构家数
    pub hold_num: i32,
    /// 持股总数
    pub total_shares: Decimal,
    /// 持股市值
    pub hold_value: Decimal,
    /// 占流通股%
    pub free_ratio: Option<Decimal>,
    /// 加仓/减仓/新进/不变
    pub hold_change: String,
    /// 报告期
    pub report_date: String,
}
