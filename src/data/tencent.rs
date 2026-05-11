/// 腾讯实时行情API客户端
///
/// API: http://qt.gtimg.cn/q=sh600519,sz000001
/// 返回GBK编码的文本，格式: v_sh600519="1~贵州茅台~600519~1372.99~..."
/// 字段用波浪号(~)分隔，约48个字段
use crate::models::quote::Quote;
use crate::models::stock::StockCode;
use rust_decimal::Decimal;
use std::str::FromStr;

/// 解析腾讯实时行情响应
///
/// 响应格式(每行一个股票):
/// v_sh600519="1~贵州茅台~600519~1372.99~1371.05~1371.66~33369~13481~19888~1372.60~...";
///
/// 关键字段索引(波浪号分隔后):
/// 1  = 股票名称
/// 2  = 股票代码
/// 3  = 当前价格
/// 4  = 昨收价
/// 5  = 今开
/// 33 = 最高
/// 34 = 最低
/// 6  = 成交量(手)
/// 37 = 成交额(万)
/// 31 = 涨跌额
/// 32 = 涨跌幅
/// 9  = 买一价
/// 19 = 卖一价(需确认)
/// 38 = 时间
pub fn parse_tencent_quotes(raw: &str, _codes: &[StockCode]) -> Vec<Quote> {
    let mut quotes = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // 格式: v_sh600519="..."; 或 v_sz000001="...";
        // 提取引号内的内容
        if let Some(start) = line.find('"') {
            if let Some(end) = line.rfind('"') {
                let content = &line[start + 1..end];
                let fields: Vec<&str> = content.split('~').collect();

                if fields.len() < 39 {
                    continue;
                }

                // 从第一行提取股票代码前缀和代码
                // line的开头有 v_sh600519= ，可以提取市场前缀
                let prefix_and_code = if let Some(eq_pos) = line.find('=') {
                    let var_name = &line[..eq_pos];
                    // var_name = "v_sh600519" → 提取 "sh600519"
                    var_name.strip_prefix('v').unwrap_or(var_name).trim_start_matches('_')
                } else {
                    continue;
                };

                let code = match StockCode::from_raw(prefix_and_code) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let name = fields[1].to_string();
                let price = parse_decimal(fields[3]);
                let prev_close = parse_decimal(fields[4]);
                let open = parse_decimal(fields[5]);
                let high = parse_decimal(fields[33]);
                let low = parse_decimal(fields[34]);
                // 成交量: 字段6是手(100股/手)，转成股
                let volume = fields[6].parse::<i64>().unwrap_or(0) * 100;
                // 成交额: 字段37是万元
                let turnover = parse_decimal(fields[37]) * Decimal::from(10000);
                let change = parse_decimal(fields[31]);
                let change_pct = parse_decimal(fields[32]);
                let bid = parse_decimal(fields[9]);
                let ask = parse_decimal(fields[19]);

                quotes.push(Quote {
                    code,
                    name,
                    price,
                    prev_close,
                    open,
                    high,
                    low,
                    volume,
                    turnover,
                    bid,
                    ask,
                    change,
                    change_pct,
                    time: String::new(), // 腾讯API时间字段不直观，暂留空
                });
            }
        }
    }

    quotes
}

/// 安全解析Decimal，解析失败返回0
fn parse_decimal(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap_or(Decimal::ZERO)
}

/// 构建腾讯实时行情请求URL
pub fn build_quote_url(codes: &[StockCode]) -> String {
    let codes_str: Vec<String> = codes.iter().map(|c| c.for_api()).collect();
    format!("http://qt.gtimg.cn/q={}", codes_str.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tencent_response() {
        // 模拟腾讯API返回数据(GBK解码后的UTF-8)
        let raw = r#"v_sh600519="1~贵州茅台~600519~1372.99~1371.05~1371.66~33369~13481~19888~1372.60~1~1372.50~3~1372.11~1~1372.10~30~1372.08~423~1372.99~4000~1373.00~200~1373.03~200~1373.05~100~1373.09~20260508~1.94~0.14~1382.77~1370.00~1372.99/33369/4582855939~33369~458286~0.27~20.79~~1382.77~1370.00~0.93~17193.54~17193.54~6.35~1508.16~1233.95~0.80~-13~1373.41~15.78~20.89~~~0.45~458285.5939~0.0000~0~ ~GP-A~-0.30~-2.28~3.76~30.53~26.78~1593.44~1322.01~-2.76~-4.65~-2.00~1252270215~1252270215~-15.29~-2.19~1252270215~~~-10.06~0.12~~CNY~0~___D__F__N~1373.20~-81~";"#;

        let codes = vec![StockCode::from_raw("600519").unwrap()];
        let quotes = parse_tencent_quotes(raw, &codes);

        assert_eq!(quotes.len(), 1);
        let q = &quotes[0];
        assert_eq!(q.code.for_api(), "sh600519");
        assert_eq!(q.name, "贵州茅台");
        assert!(q.price > Decimal::ZERO);
    }

    #[test]
    fn test_build_quote_url() {
        let codes = vec![
            StockCode::from_raw("600519").unwrap(),
            StockCode::from_raw("000001").unwrap(),
        ];
        assert_eq!(build_quote_url(&codes), "http://qt.gtimg.cn/q=sh600519,sz000001");
    }
}
