mod analysis;
mod backtest;
mod cli;
mod data;
mod db;
mod display;
mod error;
mod models;
mod monitor;
mod receiver;

use cli::{Cli, Commands, BacktestCommands, TradeCommands, PositionCommands, WatchCommands, AlertCommands, MonitorCommands};
use clap::Parser;
use data::DataClient;
use data::tencent_kline::KlineType;
use display::table;
use models::stock::StockCode;
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = DataClient::new();

    match cli.command {
        Commands::Search { query, limit } => {
            let results = client.search_stocks(&query, limit).await?;
            if results.is_empty() {
                println!("未找到匹配的股票");
            } else {
                for r in &results {
                    println!("{} {} ({}) [{}]", r.code, r.name, r.pinyin, r.market);
                }
            }
        }

        Commands::Quote { codes } => {
            let stock_codes: Vec<StockCode> = codes
                .split(',')
                .filter_map(|s| StockCode::from_raw(s.trim()).ok())
                .collect();
            if stock_codes.is_empty() {
                println!("请输入有效的股票代码");
                return Ok(());
            }
            let quotes = client.get_quotes(&stock_codes).await?;
            table::display_quotes(&quotes);
        }

        Commands::Kline { code, ktype, count, qfq } => {
            let stock_code = StockCode::from_raw(&code)?;
            let kline_type = KlineType::from_str_opt(&ktype)
                .ok_or_else(|| error::AstockError::ParseError(format!("不支持的K线类型: {}", ktype)))?;

            let bars = client
                .get_kline(&stock_code, &kline_type, None, None, Some(count), qfq)
                .await?;

            if bars.is_empty() {
                println!("未获取到K线数据");
            } else {
                println!("{} K线数据 (共{}条):", stock_code.display_wind(), bars.len());
                table::display_kline(&bars, count as usize);
            }
        }

        Commands::Info { code } => {
            let stock_code = StockCode::from_raw(&code)?;
            let financials = client.get_financials(&stock_code).await?;

            if financials.is_empty() {
                println!("未获取到财务数据");
            } else {
                println!("{} 基本面信息:", stock_code.display_wind());
                for f in &financials {
                    println!("\n--- {} ({}) ---", f.report_date, f.report_type);
                    if let Some(eps) = f.eps {
                        println!("  每股收益: {:.4}", eps);
                    }
                    if let Some(bvps) = f.bvps {
                        println!("  每股净资产: {:.2}", bvps);
                    }
                    if let Some(roe) = f.roe {
                        println!("  净资产收益率: {:.2}%", roe);
                    }
                    if let Some(gm) = f.gross_margin {
                        println!("  毛利率: {:.2}%", gm);
                    }
                    if let Some(nm) = f.net_margin {
                        println!("  净利率: {:.2}%", nm);
                    }
                    if let Some(ry) = f.revenue_yoy {
                        println!("  营收同比增长: {:.2}%", ry);
                    }
                    if let Some(py) = f.profit_yoy {
                        println!("  净利润同比增长: {:.2}%", py);
                    }
                    if let Some(dr) = f.debt_ratio {
                        println!("  资产负债率: {:.2}%", dr);
                    }
                }
            }
        }

        Commands::Indicator { indicator_type, code, count } => {
            let stock_code = StockCode::from_raw(&code)?;
            let bars = client
                .get_kline(&stock_code, &KlineType::Day, None, None, Some(count), true)
                .await?;

            if bars.is_empty() {
                println!("未获取到K线数据");
                return Ok(());
            }

            match indicator_type.as_str() {
                "all" | "a" => {
                    if let Some(report) = analysis::compute_all_indicators(&bars) {
                        println!("{}", analysis::format_report(&report, &stock_code.display_wind()));
                    }
                }
                "ma" => {
                    let ma5 = analysis::indicators::compute_sma(&bars, 5);
                    let ma10 = analysis::indicators::compute_sma(&bars, 10);
                    let ma20 = analysis::indicators::compute_sma(&bars, 20);
                    let ma60 = analysis::indicators::compute_sma(&bars, 60);
                    println!("{} MA指标:", stock_code.display_wind());
                    for (i, bar) in bars.iter().enumerate() {
                        let s5 = ma5[i].map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let s10 = ma10[i].map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let s20 = ma20[i].map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let s60 = ma60[i].map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        println!("{}  收:{:.2}  MA5:{}  MA10:{}  MA20:{}  MA60:{}",
                            bar.date, bar.close, s5, s10, s20, s60);
                    }
                }
                "macd" => {
                    let results = analysis::macd::compute_macd(&bars, 12, 26, 9);
                    println!("{} MACD指标:", stock_code.display_wind());
                    for (i, bar) in bars.iter().enumerate() {
                        let dif = results[i].dif.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let dea = results[i].dea.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let hist = results[i].histogram.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        println!("{}  收:{:.2}  DIF:{}  DEA:{}  柱:{}", bar.date, bar.close, dif, dea, hist);
                    }
                }
                "rsi" => {
                    let results = analysis::rsi::compute_rsi(&bars, 14);
                    println!("{} RSI指标:", stock_code.display_wind());
                    for (i, bar) in bars.iter().enumerate() {
                        let rsi = results[i].map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        println!("{}  收:{:.2}  RSI:{}", bar.date, bar.close, rsi);
                    }
                }
                "kdj" => {
                    let results = analysis::kdj::compute_kdj(&bars, 9, 3, 3);
                    println!("{} KDJ指标:", stock_code.display_wind());
                    for (i, bar) in bars.iter().enumerate() {
                        let k = results[i].k.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let d = results[i].d.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let j = results[i].j.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        println!("{}  收:{:.2}  K:{}  D:{}  J:{}", bar.date, bar.close, k, d, j);
                    }
                }
                "boll" => {
                    let results = analysis::indicators::compute_boll(&bars, 20, 2.0);
                    println!("{} 布林带指标:", stock_code.display_wind());
                    for (i, bar) in bars.iter().enumerate() {
                        let upper = results[i].upper.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let mid = results[i].middle.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        let lower = results[i].lower.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".into());
                        println!("{}  收:{:.2}  上:{}  中:{}  下:{}", bar.date, bar.close, upper, mid, lower);
                    }
                }
                _ => {
                    println!("不支持的指标类型: {} (支持: all, ma, macd, rsi, kdj, boll)", indicator_type);
                }
            }
        }

        Commands::Backtest { command } => {
            match command {
                BacktestCommands::Run { code, strategy, capital, count } => {
                    let stock_code = StockCode::from_raw(&code)?;
                    let strategy_obj = backtest::strategies::create_strategy(&strategy)
                        .ok_or_else(|| error::AstockError::ParseError(
                            format!("未知策略: {} (使用 backtest list-strategies 查看可用策略)", strategy)
                        ))?;

                    // 获取K线数据
                    let bars = client
                        .get_kline(&stock_code, &KlineType::Day, None, None, Some(count), true)
                        .await?;

                    if bars.is_empty() {
                        println!("未获取到K线数据");
                        return Ok(());
                    }

                    // 缓存到本地数据库
                    if let Ok(conn) = db::open_db() {
                        if let Ok(n) = db::bar_repo::save_bars(&conn, &bars) {
                            eprintln!("已缓存 {} 条K线数据", n);
                        }
                    }

                    // 运行回测
                    let config = backtest::BacktestConfig {
                        initial_capital: rust_decimal::Decimal::from(capital),
                        ..Default::default()
                    };
                    let engine = backtest::BacktestEngine::new(config);
                    let result = engine.run(&bars, strategy_obj.as_ref());
                    println!("{}", result);
                }

                BacktestCommands::ListStrategies => {
                    println!("可用策略:");
                    for (name, desc) in backtest::strategies::list_strategies() {
                        println!("  {:15} {}", name, desc);
                    }
                }
            }
        }

        Commands::Trade { command } => {
            let conn = db::open_db()?;
            match command {
                TradeCommands::Add { code, action, price, shares, date, note } => {
                    let stock_code = StockCode::from_raw(&code)?;
                    let price_val = rust_decimal::Decimal::from_str(&price)
                        .map_err(|_| error::AstockError::ParseError(format!("无效价格: {}", price)))?;
                    let action_upper = action.to_uppercase();
                    if action_upper != "BUY" && action_upper != "SELL" {
                        println!("action 必须是 buy 或 sell");
                        return Ok(());
                    }
                    if shares % 100 != 0 {
                        println!("A股股数必须是100的整数倍");
                        return Ok(());
                    }

                    let id = db::trade_repo::add_trade(
                        &conn, &stock_code, &action_upper, price_val, shares, &date, &note,
                    )?;

                    // 同步更新持仓
                    match action_upper.as_str() {
                        "BUY" => {
                            db::position_repo::upsert_position(&conn, &stock_code, "", shares, price_val)?;
                        }
                        "SELL" => {
                            db::position_repo::reduce_position(&conn, &stock_code, shares)?;
                        }
                        _ => {}
                    }

                    println!("已添加交易记录 #{}: {} {} {}股 @{}", id, action_upper, code, shares, price);
                }

                TradeCommands::List { code, action } => {
                    let stock_code = code.as_ref().and_then(|c| StockCode::from_raw(c).ok());
                    let action_filter = action.as_deref();
                    let trades = db::trade_repo::list_trades(&conn, stock_code.as_ref(), action_filter)?;

                    if trades.is_empty() {
                        println!("暂无交易记录");
                    } else {
                        println!("{:<5} {:<12} {:<6} {:<10} {:<8} {:<12} {:<8} {}",
                            "ID", "日期", "方向", "代码", "价格", "数量", "金额", "费用");
                        for t in &trades {
                            println!("{:<5} {:<12} {:<6} {:<10} {:<8.2} {:<12} {:<8.2} {:.2}",
                                t.id, t.trade_date, t.action, t.code, t.price, t.shares, t.amount, t.total_cost);
                        }
                    }
                }

                TradeCommands::Delete { id } => {
                    if db::trade_repo::delete_trade(&conn, id)? {
                        println!("已删除交易记录 #{}", id);
                    } else {
                        println!("未找到交易记录 #{}", id);
                    }
                }

                TradeCommands::Summary => {
                    let summary = db::trade_repo::trade_summary(&conn)?;
                    println!("交易汇总:");
                    println!("  总交易次数: {}", summary.total_trades);
                    println!("  买入次数:   {}", summary.buy_count);
                    println!("  卖出次数:   {}", summary.sell_count);
                    println!("  总成交金额: {:.2}", summary.total_amount);
                    println!("  累计手续费: {:.2}", summary.total_fees);
                }
            }
        }

        Commands::Position { command } => {
            let conn = db::open_db()?;
            match command {
                PositionCommands::List => {
                    let mut positions = db::position_repo::list_positions(&conn)?;
                    if positions.is_empty() {
                        println!("暂无持仓");
                    } else {
                        // 获取实时行情
                        let codes: Vec<StockCode> = positions.iter()
                            .filter_map(|p| StockCode::from_raw(&p.code).ok())
                            .collect();
                        let quotes = if !codes.is_empty() {
                            client.get_quotes(&codes).await.unwrap_or_default()
                        } else {
                            Vec::new()
                        };

                        // 将实时价格填入
                        for pos in &mut positions {
                            if let Some(q) = quotes.iter().find(|q| q.code.for_api() == pos.code) {
                                pos.current_price = q.price;
                            }
                        }

                        println!("{:<12} {:<8} {:<10} {:<10} {:<12} {:<10} {}",
                            "代码", "股数", "成本价", "现价", "市值", "盈亏", "盈亏%");
                        let mut total_value = rust_decimal::Decimal::ZERO;
                        let mut total_cost = rust_decimal::Decimal::ZERO;
                        for p in &positions {
                            let market_value = p.current_price * rust_decimal::Decimal::from(p.shares);
                            let cost_value = p.avg_cost * rust_decimal::Decimal::from(p.shares);
                            let profit = market_value - cost_value;
                            let profit_pct = if cost_value > rust_decimal::Decimal::ZERO {
                                profit / cost_value * rust_decimal::Decimal::from(100)
                            } else {
                                rust_decimal::Decimal::ZERO
                            };
                            total_value += market_value;
                            total_cost += cost_value;
                            println!("{:<12} {:<8} {:<10.2} {:<10.2} {:<12.2} {:<10.2} {:.2}%",
                                p.code, p.shares, p.avg_cost, p.current_price, market_value, profit, profit_pct);
                        }
                        let total_profit = total_value - total_cost;
                        let total_pct = if total_cost > rust_decimal::Decimal::ZERO {
                            total_profit / total_cost * rust_decimal::Decimal::from(100)
                        } else {
                            rust_decimal::Decimal::ZERO
                        };
                        println!("───────────────────────────────────────────────────────");
                        println!("合计: 市值 {:.2}  成本 {:.2}  盈亏 {:.2} ({:.2}%)",
                            total_value, total_cost, total_profit, total_pct);
                    }
                }

                PositionCommands::Add { code, shares, avg_cost } => {
                    let stock_code = StockCode::from_raw(&code)?;
                    let cost = rust_decimal::Decimal::from_str(&avg_cost)
                        .map_err(|_| error::AstockError::ParseError(format!("无效价格: {}", avg_cost)))?;
                    db::position_repo::upsert_position(&conn, &stock_code, "", shares, cost)?;
                    println!("已添加持仓: {} {}股 @{}", code, shares, avg_cost);
                }

                PositionCommands::Remove { code } => {
                    let stock_code = StockCode::from_raw(&code)?;
                    if db::position_repo::remove_position(&conn, &stock_code)? {
                        println!("已删除持仓: {}", code);
                    } else {
                        println!("未找到持仓: {}", code);
                    }
                }
            }
        }

        Commands::Watch { command } => {
            let conn = db::open_db()?;
            match command {
                WatchCommands::List => {
                    let items = db::watch_repo::list_watch(&conn)?;
                    if items.is_empty() {
                        println!("自选股为空，使用 astock watch add <code> 添加");
                    } else {
                        for w in &items {
                            println!("{} {}", w.code, w.name);
                        }
                    }
                }
                WatchCommands::Add { code } => {
                    let stock_code = StockCode::from_raw(&code)?;
                    // 尝试获取股票名称
                    let name = if let Ok(quotes) = client.get_quotes(&[stock_code.clone()]).await {
                        quotes.first().map(|q| q.name.clone()).unwrap_or_default()
                    } else {
                        String::new()
                    };
                    db::watch_repo::add_watch(&conn, &stock_code, &name)?;
                    println!("已添加自选股: {} {}", stock_code.display_wind(), name);
                }
                WatchCommands::Remove { code } => {
                    let stock_code = StockCode::from_raw(&code)?;
                    if db::watch_repo::remove_watch(&conn, &stock_code)? {
                        println!("已删除自选股: {}", code);
                    } else {
                        println!("未找到自选股: {}", code);
                    }
                }
                WatchCommands::Quote => {
                    let items = db::watch_repo::list_watch(&conn)?;
                    if items.is_empty() {
                        println!("自选股为空");
                    } else {
                        let codes: Vec<StockCode> = items.iter()
                            .filter_map(|w| StockCode::from_raw(&w.code).ok())
                            .collect();
                        let quotes = client.get_quotes(&codes).await?;
                        table::display_quotes(&quotes);
                    }
                }
            }
        }

        Commands::Alert { command } => {
            let conn = db::open_db()?;
            match command {
                AlertCommands::Add { code, signal_type, price } => {
                    let stock_code = StockCode::from_raw(&code)?;
                    let params = match signal_type.as_str() {
                        "price_above" | "price_below" => {
                            let p = price.unwrap_or(0.0);
                            serde_json::json!({"price": p}).to_string()
                        }
                        _ => "{}".to_string(),
                    };
                    let id = db::alert_repo::add_alert_rule(&conn, &stock_code.for_api(), &signal_type, &params)?;
                    println!("已添加预警规则 #{}: {} {} (参数: {})", id, stock_code.display_wind(), signal_type, params);
                }
                AlertCommands::List => {
                    let rules = db::alert_repo::list_all_rules(&conn)?;
                    if rules.is_empty() {
                        println!("暂无预警规则");
                    } else {
                        println!("{:<5} {:<12} {:<15} {:<20} {}", "ID", "代码", "类型", "参数", "状态");
                        for r in &rules {
                            let status = if r.enabled { "启用" } else { "禁用" };
                            println!("{:<5} {:<12} {:<15} {:<20} {}", r.id, r.code, r.signal_type, r.params, status);
                        }
                    }
                }
                AlertCommands::Remove { id } => {
                    if db::alert_repo::delete_alert_rule(&conn, id)? {
                        println!("已删除预警规则 #{}", id);
                    } else {
                        println!("未找到预警规则 #{}", id);
                    }
                }
                AlertCommands::History => {
                    let alerts = db::alert_repo::list_alert_history(&conn, 20)?;
                    if alerts.is_empty() {
                        println!("暂无预警记录");
                    } else {
                        println!("{:<5} {:<12} {:<15} {} {}", "ID", "代码", "类型", "消息", "时间");
                        for (id, code, stype, msg, time) in &alerts {
                            println!("{:<5} {:<12} {:<15} {} {}", id, code, stype, msg, time);
                        }
                    }
                }
            }
        }

        Commands::Monitor { command } => {
            match command {
                MonitorCommands::Start { interval } => {
                    monitor::watcher::start_monitor(interval).await?;
                }
            }
        }

        Commands::Receiver { port } => {
            receiver::start_server(port).await?;
        }
    }

    Ok(())
}
