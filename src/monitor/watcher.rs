/// 实时监控循环
///
/// 在交易时段(09:30-11:30, 13:00-15:00)定时轮询自选股和持仓，
/// 检测预警规则，触发桌面通知
use crate::analysis;
use crate::data::DataClient;
use crate::data::tencent_kline::KlineType;
use crate::db;
use crate::models::stock::StockCode;
use crate::monitor::notifier;
use chrono::{Local, Datelike, Weekday};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 判断当前是否在A股交易时段
fn is_market_hours() -> bool {
    let now = Local::now();
    let time = now.time();
    let weekday = now.weekday();

    // 周末不交易
    if weekday == Weekday::Sat || weekday == Weekday::Sun {
        return false;
    }

    // 上午 09:30 - 11:30
    let morning = time >= chrono::NaiveTime::from_hms_opt(9, 30, 0).unwrap()
        && time <= chrono::NaiveTime::from_hms_opt(11, 30, 0).unwrap();
    // 下午 13:00 - 15:00
    let afternoon = time >= chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap()
        && time <= chrono::NaiveTime::from_hms_opt(15, 0, 0).unwrap();

    morning || afternoon
}

/// 启动监控循环
pub async fn start_monitor(interval_secs: u64) -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Ctrl+C 优雅退出
    ctrlc_handler(r)?;

    let client = DataClient::new();
    let conn = db::open_db()?;

    println!("监控已启动 (每{}秒检查一次)", interval_secs);
    println!("交易时段: 09:30-11:30, 13:00-15:00 (工作日)");
    println!("按 Ctrl+C 停止");

    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    while running.load(Ordering::SeqCst) {
        interval.tick().await;

        if !is_market_hours() {
            continue;
        }

        // 检查预警规则
        if let Ok(rules) = db::alert_repo::list_enabled_rules(&conn) {
            for rule in &rules {
                if let Ok(code) = StockCode::from_raw(&rule.code) {
                    check_alert_rule(&client, &conn, &code, rule).await;
                }
            }
        }

        // 检查自选股行情
        if let Ok(watchlist) = db::watch_repo::list_watch(&conn) {
            let codes: Vec<StockCode> = watchlist.iter()
                .filter_map(|w| StockCode::from_raw(&w.code).ok())
                .collect();
            if !codes.is_empty() {
                if let Ok(quotes) = client.get_quotes(&codes).await {
                    let now = Local::now().format("%H:%M:%S");
                    for q in &quotes {
                        let change_pct = q.change_pct;
                        // 涨跌幅超过3%时提示
                        if change_pct > rust_decimal::Decimal::from(3) {
                            let msg = format!("{} {} 涨幅 {:.2}%", q.code.display_wind(), q.name, change_pct);
                            notifier::send_alert("A股异动提醒", &msg);
                        } else if change_pct < rust_decimal::Decimal::from(-3) {
                            let msg = format!("{} {} 跌幅 {:.2}%", q.code.display_wind(), q.name, change_pct);
                            notifier::send_alert("A股异动提醒", &msg);
                        }
                    }
                    // 显示自选股状态
                    for q in &quotes {
                        let arrow = if q.change > rust_decimal::Decimal::ZERO { "↑" } else if q.change < rust_decimal::Decimal::ZERO { "↓" } else { "→" };
                        println!("[{}] {} {} {:.2} ({:.2}%) {}", now, q.code.display_wind(), q.name, q.price, q.change_pct, arrow);
                    }
                }
            }
        }
    }

    println!("\n监控已停止");
    Ok(())
}

/// 检查单个预警规则
async fn check_alert_rule(
    client: &DataClient,
    conn: &rusqlite::Connection,
    code: &StockCode,
    rule: &db::alert_repo::AlertRule,
) {
    match rule.signal_type.as_str() {
        "price_above" | "price_below" => {
            if let Ok(quotes) = client.get_quotes(&[code.clone()]).await {
                if let Some(q) = quotes.first() {
                    let params: serde_json::Value = serde_json::from_str(&rule.params).unwrap_or_default();
                    let target = params["price"].as_f64().unwrap_or(0.0);

                    let triggered = if rule.signal_type == "price_above" {
                        q.price.to_f64().unwrap_or(0.0) > target
                    } else {
                        q.price.to_f64().unwrap_or(0.0) < target
                    };

                    if triggered {
                        let direction = if rule.signal_type == "price_above" { "突破" } else { "跌破" };
                        let msg = format!("{} {} {} 价格{}", code.display_wind(), q.name, direction, target);
                        notifier::send_alert("价格预警", &msg);
                        println!("[预警] {}", msg);
                        let _ = db::alert_repo::record_alert(conn, Some(rule.id), &code.for_api(), &rule.signal_type, &msg);
                    }
                }
            }
        }
        "ma_cross" | "macd_cross" | "kdj_cross" => {
            // 技术指标预警: 获取K线数据并检查
            if let Ok(bars) = client.get_kline(code, &KlineType::Day, None, None, Some(60), true).await {
                if bars.len() < 30 {
                    return;
                }
                if let Some(report) = analysis::compute_all_indicators(&bars) {
                    let triggered = match rule.signal_type.as_str() {
                        "ma_cross" => {
                            let ma5 = report.ma.get(0).and_then(|v| v.value);
                            let ma20 = report.ma.get(2).and_then(|v| v.value);
                            ma5.is_some() && ma20.is_some() && ma5.unwrap() > ma20.unwrap()
                        }
                        "macd_cross" => {
                            report.macd.dif.is_some() && report.macd.dea.is_some()
                                && report.macd.dif.unwrap() > report.macd.dea.unwrap()
                        }
                        "kdj_cross" => {
                            report.kdj.k.is_some() && report.kdj.d.is_some()
                                && report.kdj.k.unwrap() > report.kdj.d.unwrap()
                        }
                        _ => false,
                    };

                    if triggered {
                        let msg = format!("{} 触发{}信号", code.display_wind(), rule.signal_type);
                        notifier::send_alert("技术指标预警", &msg);
                        println!("[预警] {}", msg);
                        let _ = db::alert_repo::record_alert(conn, Some(rule.id), &code.for_api(), &rule.signal_type, &msg);
                    }
                }
            }
        }
        _ => {}
    }
}

use rust_decimal::prelude::ToPrimitive;

fn ctrlc_handler(running: Arc<AtomicBool>) -> anyhow::Result<()> {
    ctrlc::set_handler(move || {
        running.store(false, Ordering::SeqCst);
    })?;
    Ok(())
}
