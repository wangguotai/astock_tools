/// 综合指标报告：汇总所有技术指标，生成可读报告
use crate::analysis::indicators::{compute_boll, compute_sma, BollResult};
use crate::analysis::kdj::{compute_kdj, KdjResult};
use crate::analysis::macd::{compute_macd, MacdResult};
use crate::analysis::rsi::compute_rsi;
use crate::models::bar::Bar;
use rust_decimal::Decimal;

/// 单个指标值 (带标签)
#[derive(Debug, Clone)]
pub struct IndicatorValue {
    pub name: String,
    pub value: Option<Decimal>,
}

/// 综合指标报告
#[derive(Debug, Clone)]
pub struct IndicatorReport {
    /// MA 指标组
    pub ma: Vec<IndicatorValue>,
    /// MACD
    pub macd: MacdResult,
    /// RSI
    pub rsi: Option<Decimal>,
    /// KDJ
    pub kdj: KdjResult,
    /// 布林带
    pub boll: BollResult,
    /// 当前收盘价
    pub close: Decimal,
    /// 最新日期
    pub date: String,
}

/// 计算所有指标并生成报告
pub fn compute_all_indicators(bars: &[Bar]) -> Option<IndicatorReport> {
    if bars.is_empty() {
        return None;
    }

    let last = bars.last()?;
    let last_idx = bars.len() - 1;

    // MA: 5, 10, 20, 60
    let ma5 = compute_sma(bars, 5);
    let ma10 = compute_sma(bars, 10);
    let ma20 = compute_sma(bars, 20);
    let ma60 = compute_sma(bars, 60);

    let ma = vec![
        IndicatorValue { name: "MA5".into(), value: ma5.get(last_idx).and_then(|v| *v) },
        IndicatorValue { name: "MA10".into(), value: ma10.get(last_idx).and_then(|v| *v) },
        IndicatorValue { name: "MA20".into(), value: ma20.get(last_idx).and_then(|v| *v) },
        IndicatorValue { name: "MA60".into(), value: ma60.get(last_idx).and_then(|v| *v) },
    ];

    // MACD (12, 26, 9)
    let macd_results = compute_macd(bars, 12, 26, 9);
    let macd = macd_results.get(last_idx).cloned().unwrap_or(MacdResult {
        dif: None,
        dea: None,
        histogram: None,
    });

    // RSI (14)
    let rsi_results = compute_rsi(bars, 14);
    let rsi = rsi_results.get(last_idx).and_then(|v| *v);

    // KDJ (9, 3, 3)
    let kdj_results = compute_kdj(bars, 9, 3, 3);
    let kdj = kdj_results.get(last_idx).cloned().unwrap_or(KdjResult {
        k: None,
        d: None,
        j: None,
    });

    // BOLL (20, 2.0)
    let boll_results = compute_boll(bars, 20, 2.0);
    let boll = boll_results.get(last_idx).cloned().unwrap_or(BollResult {
        upper: None,
        middle: None,
        lower: None,
    });

    Some(IndicatorReport {
        ma,
        macd,
        rsi,
        kdj,
        boll,
        close: last.close,
        date: last.date.to_string(),
    })
}

/// 格式化指标值显示
fn fmt_val(v: Option<Decimal>) -> String {
    match v {
        Some(d) => format!("{:.2}", d),
        None => "N/A".to_string(),
    }
}

/// 将报告格式化为可读文本
pub fn format_report(report: &IndicatorReport, code: &str) -> String {
    let mut lines = Vec::new();

    lines.push(format!("{} 技术指标报告 ({})", code, report.date));
    lines.push("─".repeat(60));

    // MA
    let ma_str: Vec<String> = report.ma.iter()
        .filter_map(|v| v.value.map(|val| format!("{}: {:.2}", v.name, val)))
        .collect();
    if !ma_str.is_empty() {
        lines.push(format!("MA:   {}", ma_str.join("  ")));

        // 判断均线排列
        let values: Vec<Option<Decimal>> = report.ma.iter().map(|v| v.value).collect();
        if values.iter().all(|v| v.is_some()) {
            let vals: Vec<Decimal> = values.iter().map(|v| v.unwrap()).collect();
            if vals[0] > vals[1] && vals[1] > vals[2] {
                lines.push("      → 多头排列 (MA5 > MA10 > MA20)".to_string());
            } else if vals[0] < vals[1] && vals[1] < vals[2] {
                lines.push("      → 空头排列 (MA5 < MA10 < MA20)".to_string());
            }
        }
    }

    // MACD
    lines.push(format!(
        "MACD: DIF={}  DEA={}  柱={}",
        fmt_val(report.macd.dif),
        fmt_val(report.macd.dea),
        fmt_val(report.macd.histogram),
    ));
    if let (Some(dif), Some(dea)) = (report.macd.dif, report.macd.dea) {
        if dif > dea {
            lines.push("      → DIF在DEA上方 (金叉/多头)".to_string());
        } else {
            lines.push("      → DIF在DEA下方 (死叉/空头)".to_string());
        }
    }

    // RSI
    lines.push(format!("RSI:  {}", fmt_val(report.rsi)));
    if let Some(rsi) = report.rsi {
        if rsi > Decimal::from(70) {
            lines.push("      → 超买区域 (>70)".to_string());
        } else if rsi < Decimal::from(30) {
            lines.push("      → 超卖区域 (<30)".to_string());
        } else {
            lines.push("      → 中性区域".to_string());
        }
    }

    // KDJ
    lines.push(format!(
        "KDJ:  K={}  D={}  J={}",
        fmt_val(report.kdj.k),
        fmt_val(report.kdj.d),
        fmt_val(report.kdj.j),
    ));
    if let (Some(k), Some(d)) = (report.kdj.k, report.kdj.d) {
        if k > d {
            lines.push("      → K在D上方 (金叉)".to_string());
        } else {
            lines.push("      → K在D下方 (死叉)".to_string());
        }
    }
    if let Some(j) = report.kdj.j {
        if j > Decimal::from(100) {
            lines.push("      → J值超买 (>100)".to_string());
        } else if j < Decimal::ZERO {
            lines.push("      → J值超卖 (<0)".to_string());
        }
    }

    // BOLL
    lines.push(format!(
        "BOLL: 上轨={}  中轨={}  下轨={}",
        fmt_val(report.boll.upper),
        fmt_val(report.boll.middle),
        fmt_val(report.boll.lower),
    ));
    if let (Some(upper), Some(lower)) = (report.boll.upper, report.boll.lower) {
        if report.close > upper {
            lines.push("      → 价格突破上轨 (超买)".to_string());
        } else if report.close < lower {
            lines.push("      → 价格跌破下轨 (超卖)".to_string());
        } else if let Some(mid) = report.boll.middle {
            if report.close > mid {
                lines.push("      → 价格在中轨上方".to_string());
            } else {
                lines.push("      → 价格在中轨下方".to_string());
            }
        }
    }

    lines.push("─".repeat(60));
    lines.join("\n")
}
