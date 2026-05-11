/// KDJ 指标 (随机指标)
///
/// KDJ 是中国股市常用的技术指标，基于 Stochastic 扩展:
/// - K = 2/3 * 前K + 1/3 * RSV
/// - D = 2/3 * 前D + 1/3 * K
/// - J = 3 * K - 2 * D
///
/// ta crate 不直接提供 KDJ，这里自定义实现
use crate::models::bar::Bar;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// KDJ 计算结果
#[derive(Debug, Clone)]
pub struct KdjResult {
    pub k: Option<Decimal>,
    pub d: Option<Decimal>,
    pub j: Option<Decimal>,
}

/// 计算KDJ指标
///
/// period: RSV计算周期 (默认9)
/// k_smooth: K值平滑系数 (默认3，即 2/3 权重给前值)
/// d_smooth: D值平滑系数 (默认3)
pub fn compute_kdj(bars: &[Bar], period: usize, k_smooth: usize, d_smooth: usize) -> Vec<KdjResult> {
    let mut results = Vec::with_capacity(bars.len());
    let mut prev_k: f64 = 50.0; // K初始值
    let mut prev_d: f64 = 50.0; // D初始值

    for (i, _bar) in bars.iter().enumerate() {
        if i + 1 < period {
            // 数据不足，无法计算RSV
            results.push(KdjResult {
                k: None,
                d: None,
                j: None,
            });
            continue;
        }

        // 计算RSV (Raw Stochastic Value)
        // RSV = (Close - LowN) / (HighN - LowN) * 100
        // 其中 LowN/HighN 是最近N根K线的最低/最高价
        let window = &bars[i + 1 - period..=i];
        let low_n = window.iter().map(|b| b.low.to_f64().unwrap_or(0.0)).fold(f64::MAX, f64::min);
        let high_n = window.iter().map(|b| b.high.to_f64().unwrap_or(0.0)).fold(f64::MIN, f64::max);
        let close = bars[i].close.to_f64().unwrap_or(0.0);

        let rsv = if (high_n - low_n).abs() < 1e-10 {
            50.0 // 避免除零
        } else {
            (close - low_n) / (high_n - low_n) * 100.0
        };

        // K = (k_smooth - 1) / k_smooth * prev_K + 1 / k_smooth * RSV
        let k = (k_smooth - 1) as f64 / k_smooth as f64 * prev_k
            + 1.0 / k_smooth as f64 * rsv;

        // D = (d_smooth - 1) / d_smooth * prev_D + 1 / d_smooth * K
        let d = (d_smooth - 1) as f64 / d_smooth as f64 * prev_d
            + 1.0 / d_smooth as f64 * k;

        // J = 3 * K - 2 * D
        let j = 3.0 * k - 2.0 * d;

        prev_k = k;
        prev_d = d;

        results.push(KdjResult {
            k: Decimal::from_f64_retain(k),
            d: Decimal::from_f64_retain(d),
            j: Decimal::from_f64_retain(j),
        });
    }

    results
}
