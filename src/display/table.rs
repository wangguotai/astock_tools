/// 表格和颜色展示工具
///
/// A股惯例: 红涨绿跌 (与国际市场相反)
use crate::models::bar::Bar;
use crate::models::quote::Quote;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use rust_decimal::Decimal;

/// 判断涨跌颜色
/// A股: 红涨绿跌
fn price_color(change: Decimal) -> Color {
    if change > Decimal::ZERO {
        Color::Red
    } else if change < Decimal::ZERO {
        Color::Green
    } else {
        Color::White
    }
}

/// 格式化涨跌幅显示
fn fmt_change_pct(pct: Decimal) -> String {
    if pct > Decimal::ZERO {
        format!("+{:.2}%", pct)
    } else {
        format!("{:.2}%", pct)
    }
}

/// 格式化涨跌额显示
fn fmt_change(change: Decimal) -> String {
    if change > Decimal::ZERO {
        format!("+{:.2}", change)
    } else {
        format!("{:.2}", change)
    }
}

/// 格式化成交量(手)
fn fmt_volume(volume: i64) -> String {
    if volume >= 100_000_000 {
        format!("{:.2}亿手", volume as f64 / 100_000_000.0)
    } else if volume >= 10_000 {
        format!("{:.2}万手", volume as f64 / 10_000.0)
    } else {
        format!("{}手", volume)
    }
}

/// 格式化金额(元)
fn fmt_amount(amount: Decimal) -> String {
    if amount >= Decimal::from(100_000_000) {
        format!("{:.2}亿", amount / Decimal::from(100_000_000))
    } else if amount >= Decimal::from(10_000) {
        format!("{:.2}万", amount / Decimal::from(10_000))
    } else {
        format!("{:.2}", amount)
    }
}

/// 显示实时行情
pub fn display_quotes(quotes: &[Quote]) {
    if quotes.is_empty() {
        println!("未获取到行情数据");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "代码", "名称", "现价", "涨跌", "涨跌幅", "今开", "最高", "最低", "成交量", "成交额",
    ]);

    for q in quotes {
        let color = price_color(q.change);
        table.add_row(vec![
            Cell::new(q.code.display_wind()),
            Cell::new(&q.name),
            Cell::new(format!("{:.2}", q.price)).fg(color),
            Cell::new(fmt_change(q.change)).fg(color),
            Cell::new(fmt_change_pct(q.change_pct)).fg(color),
            Cell::new(format!("{:.2}", q.open)),
            Cell::new(format!("{:.2}", q.high)),
            Cell::new(format!("{:.2}", q.low)),
            Cell::new(fmt_volume(q.volume / 100)),
            Cell::new(fmt_amount(q.turnover)),
        ]);
    }

    println!("{table}");
}

/// 显示K线数据(最近N条)
pub fn display_kline(bars: &[Bar], count: usize) {
    if bars.is_empty() {
        println!("未获取到K线数据");
        return;
    }

    let start = bars.len().saturating_sub(count);
    let display_bars = &bars[start..];

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "日期", "开盘", "收盘", "最高", "最低", "涨跌幅", "成交量", "成交额",
    ]);

    for (i, bar) in display_bars.iter().enumerate() {
        let change = if i > 0 {
            bar.close - display_bars[i - 1].close
        } else {
            Decimal::ZERO
        };
        let change_pct = if i > 0 && display_bars[i - 1].close > Decimal::ZERO {
            change / display_bars[i - 1].close * Decimal::from(100)
        } else {
            Decimal::ZERO
        };
        let color = price_color(change);

        table.add_row(vec![
            Cell::new(bar.date.to_string()),
            Cell::new(format!("{:.2}", bar.open)),
            Cell::new(format!("{:.2}", bar.close)).fg(color),
            Cell::new(format!("{:.2}", bar.high)),
            Cell::new(format!("{:.2}", bar.low)),
            Cell::new(fmt_change_pct(change_pct)).fg(color),
            Cell::new(fmt_volume(bar.volume / 100)),
            Cell::new(fmt_amount(bar.turnover)),
        ]);
    }

    println!("{table}");
}
