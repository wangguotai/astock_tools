/// A股交易成本模型
///
/// 佣金: 万三(0.03%)，最低5元
/// 印花税: 0.05% (2023年8月减半，仅卖出)
/// 过户费: 0.001% (买卖都收)
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// 计算买入总成本 (含佣金+过户费)
pub fn calc_buy_cost(price: Decimal, shares: i64) -> Decimal {
    let amount = price * Decimal::from(shares);
    let commission = calc_commission(amount);
    let transfer_fee = amount * dec!(0.00001); // 0.001%
    commission + transfer_fee
}

/// 计算卖出总成本 (含佣金+印花税+过户费)
pub fn calc_sell_cost(price: Decimal, shares: i64) -> Decimal {
    let amount = price * Decimal::from(shares);
    let commission = calc_commission(amount);
    let stamp_tax = amount * dec!(0.0005); // 0.05% (2023年8月减半)
    let transfer_fee = amount * dec!(0.00001);
    commission + stamp_tax + transfer_fee
}

/// 计算佣金 (万三，最低5元)
fn calc_commission(amount: Decimal) -> Decimal {
    let commission = amount * dec!(0.0003);
    if commission < dec!(5) {
        dec!(5)
    } else {
        commission
    }
}

/// 将股数向下取整到100的整数倍 (A股1手=100股)
pub fn round_lot(shares: i64) -> i64 {
    (shares / 100) * 100
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_buy_cost() {
        // 10元买入1000股
        let cost = calc_buy_cost(dec!(10), 1000);
        // 金额=10000, 佣金=3→5(最低), 过户费=0.1, 总=5.1
        assert_eq!(cost, dec!(5.1));
    }

    #[test]
    fn test_sell_cost() {
        // 10元卖出1000股
        let cost = calc_sell_cost(dec!(10), 1000);
        // 金额=10000, 佣金=3→5, 印花税=5, 过户费=0.1, 总=10.1
        assert_eq!(cost, dec!(10.1));
    }

    #[test]
    fn test_round_lot() {
        assert_eq!(round_lot(150), 100);
        assert_eq!(round_lot(250), 200);
        assert_eq!(round_lot(100), 100);
    }
}
