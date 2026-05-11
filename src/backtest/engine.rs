/// 回测引擎 - 涨跌循环模拟
///
/// 核心逻辑:
/// 1. 逐根K线遍历历史数据
/// 2. 策略在每个bar生成信号 (Buy/Sell/Hold)
/// 3. 信号在下一个bar的开盘价执行
/// 4. 遵守T+1规则: 当日买入次日才可卖出
/// 5. 计算交易成本
use crate::backtest::config::BacktestConfig;
use crate::backtest::costs::{calc_buy_cost, calc_sell_cost, round_lot};
use crate::models::bar::Bar;
use crate::models::stock::StockCode;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt;

/// 交易信号
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// 策略 trait - 所有回测策略必须实现
pub trait Strategy: Send + Sync {
    /// 策略名称
    fn name(&self) -> &str;
    /// 根据历史K线生成交易信号
    /// bars: 所有K线数据, index: 当前bar的位置 (0..bars.len())
    /// 只能使用 bars[..=index] 的数据，不能偷看未来
    fn generate_signal(&self, bars: &[Bar], index: usize) -> Signal;
}

/// 单笔交易记录
#[derive(Debug, Clone)]
pub struct BacktestTrade {
    pub code: StockCode,
    pub action: String,     // "BUY" or "SELL"
    pub price: Decimal,
    pub shares: i64,
    pub amount: Decimal,
    pub cost: Decimal,      // 手续费
    pub date: NaiveDate,
}

/// 回测结果
#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub strategy_name: String,
    pub initial_capital: Decimal,
    pub final_capital: Decimal,
    pub total_return_pct: Decimal,
    pub max_drawdown_pct: Decimal,
    pub win_rate: Decimal,
    pub profit_loss_ratio: Decimal,
    pub trade_count: usize,
    pub total_cost: Decimal,
    pub trades: Vec<BacktestTrade>,
}

impl fmt::Display for BacktestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "策略: {}", self.strategy_name)?;
        writeln!(f, "───────────────────────────────────")?;
        writeln!(f, "初始资金:  {:.2}", self.initial_capital)?;
        writeln!(f, "最终资金:  {:.2}", self.final_capital)?;
        writeln!(f, "总收益率:  {:.2}%", self.total_return_pct)?;
        writeln!(f, "最大回撤:  {:.2}%", self.max_drawdown_pct)?;
        writeln!(f, "胜率:      {:.2}%", self.win_rate)?;
        writeln!(f, "盈亏比:    {:.2}", self.profit_loss_ratio)?;
        writeln!(f, "交易次数:  {}", self.trade_count)?;
        writeln!(f, "累计费用:  {:.2}", self.total_cost)?;
        Ok(())
    }
}

/// 回测引擎
pub struct BacktestEngine {
    pub config: BacktestConfig,
}

impl BacktestEngine {
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// 运行回测
    pub fn run(&self, bars: &[Bar], strategy: &dyn Strategy) -> BacktestResult {
        let mut cash = self.config.initial_capital;
        let mut position_shares: i64 = 0;       // 持仓股数
        let mut position_cost: Decimal = dec!(0); // 持仓成本总额
        let mut buy_date: Option<NaiveDate> = None; // 最近买入日期 (T+1检查)
        let mut trades: Vec<BacktestTrade> = Vec::new();
        let mut total_cost = Decimal::ZERO;

        // 用于计算最大回撤
        let mut peak_value = self.config.initial_capital;
        let mut max_drawdown = Decimal::ZERO;

        // 需要至少2根K线才能回测
        if bars.len() < 2 {
            return BacktestResult {
                strategy_name: strategy.name().to_string(),
                initial_capital: self.config.initial_capital,
                final_capital: cash,
                total_return_pct: Decimal::ZERO,
                max_drawdown_pct: Decimal::ZERO,
                win_rate: Decimal::ZERO,
                profit_loss_ratio: dec!(0),
                trade_count: 0,
                total_cost: Decimal::ZERO,
                trades: Vec::new(),
            };
        }

        let mut pending_signal: Option<Signal> = None;

        for i in 0..bars.len() {
            let bar = &bars[i];

            // 执行上一个bar的信号
            if let Some(signal) = pending_signal.take() {
                // 加入滑点后的执行价格
                let exec_price = match signal {
                    Signal::Buy => bar.open * (Decimal::ONE + self.config.slippage),
                    Signal::Sell => bar.open * (Decimal::ONE - self.config.slippage),
                    Signal::Hold => bar.open,
                };

                match signal {
                    Signal::Buy if position_shares == 0 => {
                        // 计算可买股数 (向下取整到100的倍数)
                        let max_shares = round_lot(
                            (cash / exec_price).to_i64().unwrap_or(0)
                        );
                        if max_shares >= 100 {
                            let buy_cost = calc_buy_cost(exec_price, max_shares);
                            let total_amount = exec_price * Decimal::from(max_shares);
                            if cash >= total_amount + buy_cost {
                                cash -= total_amount + buy_cost;
                                position_shares = max_shares;
                                position_cost = total_amount + buy_cost;
                                buy_date = Some(bar.date);
                                total_cost += buy_cost;

                                trades.push(BacktestTrade {
                                    code: bar.code.clone(),
                                    action: "BUY".into(),
                                    price: exec_price,
                                    shares: max_shares,
                                    amount: total_amount,
                                    cost: buy_cost,
                                    date: bar.date,
                                });
                            }
                        }
                    }
                    Signal::Sell if position_shares > 0 => {
                        // T+1检查: 当日买入不能当日卖出
                        let can_sell = if self.config.t_plus_1 {
                            buy_date.map_or(true, |d| bar.date > d)
                        } else {
                            true
                        };

                        if can_sell {
                            let sell_cost = calc_sell_cost(exec_price, position_shares);
                            let total_amount = exec_price * Decimal::from(position_shares);
                            cash += total_amount - sell_cost;
                            total_cost += sell_cost;

                            trades.push(BacktestTrade {
                                code: bar.code.clone(),
                                action: "SELL".into(),
                                price: exec_price,
                                shares: position_shares,
                                amount: total_amount,
                                cost: sell_cost,
                                date: bar.date,
                            });

                            position_shares = 0;
                            position_cost = Decimal::ZERO;
                            buy_date = None;
                        }
                    }
                    _ => {}
                }
            }

            // 计算当前总资产 (用于回撤)
            let current_value = cash + exec_price_if_held(position_shares, bar.close);
            if current_value > peak_value {
                peak_value = current_value;
            }
            let drawdown = (peak_value - current_value) / peak_value * dec!(100);
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }

            // 生成下一个bar的信号
            if i + 1 < bars.len() {
                let signal = strategy.generate_signal(bars, i);
                if signal != Signal::Hold {
                    pending_signal = Some(signal);
                }
            }
        }

        // 计算最终结果
        let last_price = bars.last().map(|b| b.close).unwrap_or(Decimal::ZERO);
        let final_value = cash + Decimal::from(position_shares) * last_price;
        let total_return_pct = (final_value - self.config.initial_capital)
            / self.config.initial_capital * dec!(100);

        // 计算胜率和盈亏比
        let (win_rate, pl_ratio) = calc_win_rate_pl_ratio(&trades);

        BacktestResult {
            strategy_name: strategy.name().to_string(),
            initial_capital: self.config.initial_capital,
            final_capital: final_value,
            total_return_pct,
            max_drawdown_pct: max_drawdown,
            win_rate,
            profit_loss_ratio: pl_ratio,
            trade_count: trades.len(),
            total_cost,
            trades,
        }
    }
}

/// 计算持仓市值
fn exec_price_if_held(shares: i64, price: Decimal) -> Decimal {
    Decimal::from(shares) * price
}

/// 计算胜率和盈亏比
fn calc_win_rate_pl_ratio(trades: &[BacktestTrade]) -> (Decimal, Decimal) {
    // 配对买卖: 一次买入+一次卖出为一笔完整交易
    let mut profits: Vec<Decimal> = Vec::new();
    let mut buy_amount = Decimal::ZERO;
    let mut buy_shares: i64 = 0;

    for t in trades {
        match t.action.as_str() {
            "BUY" => {
                buy_amount = t.amount + t.cost;
                buy_shares = t.shares;
            }
            "SELL" => {
                if buy_shares > 0 {
                    let sell_net = t.amount - t.cost;
                    let profit = sell_net - buy_amount;
                    profits.push(profit);
                    buy_amount = Decimal::ZERO;
                    buy_shares = 0;
                }
            }
            _ => {}
        }
    }

    if profits.is_empty() {
        return (Decimal::ZERO, dec!(0));
    }

    let wins = profits.iter().filter(|&&p| p > Decimal::ZERO).count();
    let win_rate = Decimal::from(wins) / Decimal::from(profits.len()) * dec!(100);

    let total_win: Decimal = profits.iter().filter(|&&p| p > Decimal::ZERO).sum();
    let total_loss: Decimal = profits.iter().filter(|&&p| p < Decimal::ZERO).map(|p| p.abs()).sum();
    let win_count = profits.iter().filter(|&&p| p > Decimal::ZERO).count();
    let loss_count = profits.iter().filter(|&&p| p < Decimal::ZERO).count();

    let pl_ratio = if loss_count > 0 && total_loss > Decimal::ZERO {
        (total_win / Decimal::from(win_count.max(1))) / (total_loss / Decimal::from(loss_count))
    } else if win_count > 0 {
        dec!(99) // 全胜
    } else {
        dec!(0)
    };

    (win_rate, pl_ratio)
}

use rust_decimal::prelude::ToPrimitive;
