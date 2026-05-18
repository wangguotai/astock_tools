/// HTTP接收端处理函数
use crate::db;
use crate::models::stock::StockCode;
use crate::monitor::notifier;
use crate::receiver::models::*;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// 共享状态
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub start_time: Instant,
}

/// POST /api/v1/quote
pub async fn push_quote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushRequest<QuoteData>>,
) -> Result<(StatusCode, Json<ApiResponse>), StatusCode> {
    let code = match StockCode::from_raw(&req.data.code) {
        Ok(c) => c,
        Err(_) => return Ok((StatusCode::BAD_REQUEST, Json(ApiResponse::error("无效股票代码")))),
    };

    let parse_dec = |s: Option<&String>| -> Decimal {
        s.as_ref()
            .and_then(|v| Decimal::from_str(v).ok())
            .unwrap_or(Decimal::ZERO)
    };

    let quote = crate::models::quote::Quote {
        code: code.clone(),
        name: req.data.name.unwrap_or_default(),
        price: parse_dec(Some(&req.data.price)),
        prev_close: parse_dec(req.data.prev_close.as_ref()),
        open: parse_dec(req.data.open.as_ref()),
        high: parse_dec(req.data.high.as_ref()),
        low: parse_dec(req.data.low.as_ref()),
        volume: req.data.volume.unwrap_or(0),
        turnover: parse_dec(req.data.turnover.as_ref()),
        bid: parse_dec(req.data.bid.as_ref()),
        ask: parse_dec(req.data.ask.as_ref()),
        bid_vol: req.data.bid_vol,
        ask_vol: req.data.ask_vol,
        change: parse_dec(req.data.change.as_ref()),
        change_pct: parse_dec(req.data.change_pct.as_ref()),
        high_limit: req.data.high_limit.as_ref().and_then(|v| Decimal::from_str(v).ok()),
        low_limit: req.data.low_limit.as_ref().and_then(|v| Decimal::from_str(v).ok()),
        inner_vol: req.data.inner_vol,
        outer_vol: req.data.outer_vol,
        open_vol: req.data.open_vol,
        time: req.data.time.unwrap_or_default(),
    };

    let conn = state.db.lock().await;
    if let Err(e) = db::quote_snapshot_repo::save_quote_snapshot(&conn, &quote, &req.source) {
        eprintln!("[receiver] 保存行情快照失败: {}", e);
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("保存失败"))));
    }

    // 检查预警规则
    check_price_alerts(&conn, &quote);

    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}

/// POST /api/v1/kline
pub async fn push_kline(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushRequest<KlineBatchData>>,
) -> Result<(StatusCode, Json<ApiResponse>), StatusCode> {
    let conn = state.db.lock().await;
    let mut bars = Vec::new();

    for entry in &req.data.bars {
        let code = match StockCode::from_raw(&entry.code) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let date = chrono::NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d").ok();
        if let Some(date) = date {
            bars.push(crate::models::bar::Bar {
                code,
                date,
                open: Decimal::from_str(&entry.open).unwrap_or(Decimal::ZERO),
                close: Decimal::from_str(&entry.close).unwrap_or(Decimal::ZERO),
                high: Decimal::from_str(&entry.high).unwrap_or(Decimal::ZERO),
                low: Decimal::from_str(&entry.low).unwrap_or(Decimal::ZERO),
                volume: entry.volume.unwrap_or(0),
                turnover: Decimal::from_str(entry.turnover.as_ref().unwrap_or(&"0".to_string())).unwrap_or(Decimal::ZERO),
            });
        }
    }

    if !bars.is_empty() {
        if let Err(e) = db::bar_repo::save_bars(&conn, &bars) {
            eprintln!("[receiver] 保存K线失败: {}", e);
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("保存失败"))));
        }
    }

    let count = bars.len();
    Ok((StatusCode::OK, Json(ApiResponse {
        status: "ok".into(),
        message: Some(format!("保存{}条K线", count)),
    })))
}

/// POST /api/v1/tick
pub async fn push_tick(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushRequest<TickBatchData>>,
) -> Result<(StatusCode, Json<ApiResponse>), StatusCode> {
    let conn = state.db.lock().await;
    let mut ticks = Vec::new();

    for entry in &req.data.ticks {
        let code = match StockCode::from_raw(&entry.code) {
            Ok(c) => c,
            Err(_) => continue,
        };
        ticks.push(crate::models::tick_trade::TickTrade {
            tick_id: entry.id.clone(),
            code,
            trade_time: entry.trade_time.clone(),
            price: Decimal::from_str(&entry.price).unwrap_or(Decimal::ZERO),
            volume: entry.volume,
            direction: entry.direction.clone().unwrap_or_else(|| "neutral".into()),
            source: req.source.clone(),
        });
    }

    if !ticks.is_empty() {
        if let Err(e) = db::tick_repo::save_tick_trades(&conn, &ticks) {
            eprintln!("[receiver] 保存分时成交失败: {}", e);
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("保存失败"))));
        }
    }

    let count = ticks.len();
    Ok((StatusCode::OK, Json(ApiResponse {
        status: "ok".into(),
        message: Some(format!("保存{}条成交", count)),
    })))
}

/// POST /api/v1/orderbook
pub async fn push_orderbook(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushRequest<OrderBookData>>,
) -> Result<(StatusCode, Json<ApiResponse>), StatusCode> {
    let code = match StockCode::from_raw(&req.data.code) {
        Ok(c) => c,
        Err(_) => return Ok((StatusCode::BAD_REQUEST, Json(ApiResponse::error("无效股票代码")))),
    };

    let parse_arr = |strs: &[String]| -> [Decimal; 5] {
        let mut arr = [Decimal::ZERO; 5];
        for (i, s) in strs.iter().take(5).enumerate() {
            arr[i] = Decimal::from_str(s).unwrap_or(Decimal::ZERO);
        }
        arr
    };
    let parse_vol_arr = |vols: &[i64]| -> [i64; 5] {
        let mut arr = [0i64; 5];
        for (i, &v) in vols.iter().take(5).enumerate() {
            arr[i] = v;
        }
        arr
    };

    let ob = crate::models::order_book::OrderBookSnapshot {
        code,
        snap_time: req.data.snap_time,
        bid_prices: parse_arr(&req.data.bid_prices),
        bid_volumes: parse_vol_arr(&req.data.bid_volumes),
        ask_prices: parse_arr(&req.data.ask_prices),
        ask_volumes: parse_vol_arr(&req.data.ask_volumes),
        source: req.source,
    };

    let conn = state.db.lock().await;
    if let Err(e) = db::orderbook_repo::save_order_book(&conn, &ob) {
        eprintln!("[receiver] 保存盘口快照失败: {}", e);
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("保存失败"))));
    }

    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}

/// POST /api/v1/moneyflow
pub async fn push_moneyflow(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushRequest<MoneyFlowData>>,
) -> Result<(StatusCode, Json<ApiResponse>), StatusCode> {
    let code = match StockCode::from_raw(&req.data.code) {
        Ok(c) => c,
        Err(_) => return Ok((StatusCode::BAD_REQUEST, Json(ApiResponse::error("无效股票代码")))),
    };

    let parse_dec = |s: &str| Decimal::from_str(s).unwrap_or(Decimal::ZERO);

    let mf = crate::models::money_flow::MoneyFlow {
        code,
        trade_date: req.data.trade_date,
        main_inflow: parse_dec(&req.data.main_inflow),
        main_outflow: parse_dec(&req.data.main_outflow),
        main_net: parse_dec(&req.data.main_net),
        retail_inflow: parse_dec(&req.data.retail_inflow),
        retail_outflow: parse_dec(&req.data.retail_outflow),
        retail_net: parse_dec(&req.data.retail_net),
        source: req.source,
    };

    let conn = state.db.lock().await;
    if let Err(e) = db::moneyflow_repo::save_money_flow(&conn, &mf) {
        eprintln!("[receiver] 保存资金流向失败: {}", e);
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("保存失败"))));
    }

    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}

/// GET /api/v1/status
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok".into(),
        version: "0.1.0".into(),
        uptime_secs: state.start_time.elapsed().as_secs(),
    })
}

/// GET /api/v1/watchlist
pub async fn get_watchlist(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<WatchlistEntry>> {
    let conn = state.db.lock().await;
    let entries = match db::watch_repo::list_watch(&conn) {
        Ok(list) => list.iter().map(|w| WatchlistEntry {
            code: w.code.clone(),
            name: w.name.clone(),
        }).collect(),
        Err(_) => Vec::new(),
    };
    Json(entries)
}

/// 检查价格预警
fn check_price_alerts(conn: &rusqlite::Connection, quote: &crate::models::quote::Quote) {
    let rules = match db::alert_repo::list_enabled_rules(conn) {
        Ok(r) => r,
        Err(_) => return,
    };

    for rule in &rules {
        if rule.code != quote.code.for_api() {
            continue;
        }

        match rule.signal_type.as_str() {
            "price_above" => {
                let params: serde_json::Value = serde_json::from_str(&rule.params).unwrap_or_default();
                let target = params["price"].as_f64().unwrap_or(0.0);
                use rust_decimal::prelude::ToPrimitive;
                let price_f64 = quote.price.to_f64().unwrap_or(0.0);
                if price_f64 > target {
                    let msg = format!("{} {} 突破价格{}", quote.code.display_wind(), quote.name, target);
                    notifier::send_alert("价格预警", &msg);
                    println!("[预警] {}", msg);
                    let _ = db::alert_repo::record_alert(conn, Some(rule.id), &quote.code.for_api(), "price_above", &msg);
                }
            }
            "price_below" => {
                let params: serde_json::Value = serde_json::from_str(&rule.params).unwrap_or_default();
                let target = params["price"].as_f64().unwrap_or(0.0);
                use rust_decimal::prelude::ToPrimitive;
                let price_f64 = quote.price.to_f64().unwrap_or(0.0);
                if price_f64 < target {
                    let msg = format!("{} {} 跌破价格{}", quote.code.display_wind(), quote.name, target);
                    notifier::send_alert("价格预警", &msg);
                    println!("[预警] {}", msg);
                    let _ = db::alert_repo::record_alert(conn, Some(rule.id), &quote.code.for_api(), "price_below", &msg);
                }
            }
            _ => {}
        }
    }
}
