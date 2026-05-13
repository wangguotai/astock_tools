/**
 * data-normalizer.js - 将同花顺API原始数据归一化为astock格式
 */

/**
 * 解析同花顺分时数据API响应
 * URL模式: d.10jqka.com.cn/v6/line/hs_{code}/01/last.js
 * 返回JSONP格式，需要提取JSON部分
 */
function parseTimeshareData(url, body, code) {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;

    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    // 解析分时数据
    const ticks = [];
    if (json.data && json.data.trends) {
      for (const t of json.data.trends) {
        // 格式: "20260513100512,12.34,12345,150000.00"
        const parts = t.split(',');
        if (parts.length >= 3) {
          const timeStr = parts[0];
          const hh = timeStr.substring(8, 10);
          const mm = timeStr.substring(10, 12);
          const ss = timeStr.substring(12, 14);
          ticks.push({
            code: astockCode,
            trade_time: `${hh}:${mm}:${ss}`,
            price: parts[1],
            volume: parseInt(parts[2]) || 0,
            direction: 'neutral',
          });
        }
      }
    }

    return ticks.length > 0 ? { type: 'tick', data: { ticks } } : null;
  } catch (e) {
    console.warn('[astock] 解析分时数据失败:', e);
    return null;
  }
}

/**
 * 解析同花顺K线数据API响应
 * URL模式: d.10jqka.com.cn/v6/line/hs_{code}/11/*.js
 */
function parseKlineData(url, body, code) {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;

    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const bars = [];
    // K线数据在json.data中
    const klineData = json.data || json;
    const dataKey = Object.keys(klineData).find(k => Array.isArray(klineData[k]));
    if (dataKey && Array.isArray(klineData[dataKey])) {
      for (const item of klineData[dataKey]) {
        if (Array.isArray(item) && item.length >= 6) {
          // [date, open, close, high, low, volume] 或 [date, open, high, low, close, volume]
          bars.push({
            code: astockCode,
            date: item[0],
            open: String(item[1]),
            close: String(item[2]),
            high: String(item[3]),
            low: String(item[4]),
            volume: parseInt(item[5]) || 0,
            turnover: item[6] ? String(item[6]) : '0',
          });
        } else if (typeof item === 'string') {
          // 字符串格式: "2026-05-13,12.40,12.34,12.60,12.30,123456"
          const parts = item.split(',');
          if (parts.length >= 6) {
            bars.push({
              code: astockCode,
              date: parts[0],
              open: parts[1],
              close: parts[2],
              high: parts[3],
              low: parts[4],
              volume: parseInt(parts[5]) || 0,
              turnover: parts[6] || '0',
            });
          }
        }
      }
    }

    return bars.length > 0 ? { type: 'kline', data: { bars } } : null;
  } catch (e) {
    console.warn('[astock] 解析K线数据失败:', e);
    return null;
  }
}

/**
 * 解析盘口数据
 */
function parseOrderBookData(url, body, code) {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;

    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    // 盘口数据格式可能是 json.data.bids / json.data.asks
    const data = json.data || json;
    const bids = data.bids || data.buy || [];
    const asks = data.asks || data.sell || [];

    const bidPrices = [];
    const bidVolumes = [];
    const askPrices = [];
    const askVolumes = [];

    for (let i = 0; i < 5 && i < bids.length; i++) {
      bidPrices.push(String(bids[i].price || bids[i][0] || '0'));
      bidVolumes.push(parseInt(bids[i].volume || bids[i][1] || 0));
    }
    for (let i = 0; i < 5 && i < asks.length; i++) {
      askPrices.push(String(asks[i].price || asks[i][0] || '0'));
      askVolumes.push(parseInt(asks[i].volume || asks[i][1] || 0));
    }

    // 补齐5档
    while (bidPrices.length < 5) { bidPrices.push('0'); bidVolumes.push(0); }
    while (askPrices.length < 5) { askPrices.push('0'); askVolumes.push(0); }

    if (bidPrices[0] === '0' && askPrices[0] === '0') return null;

    return {
      type: 'orderbook',
      data: {
        code: astockCode,
        snap_time: new Date().toISOString().replace('T', ' ').substring(0, 19),
        bid_prices: bidPrices,
        bid_volumes: bidVolumes,
        ask_prices: askPrices,
        ask_volumes: askVolumes,
      },
    };
  } catch (e) {
    console.warn('[astock] 解析盘口数据失败:', e);
    return null;
  }
}

/**
 * 解析资金流向数据
 * URL模式: d.10jqka.com.cn/v6/moneyflow/hs_{code}/detail.js
 */
function parseMoneyFlowData(url, body, code) {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;

    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const data = json.data || json;

    // 同花顺资金流向数据格式
    const mainInflow = String(data.mainInflow || data.main_inflow || '0');
    const mainOutflow = String(data.mainOutflow || data.main_outflow || '0');
    const mainNet = String(data.mainNet || data.main_net || '0');
    const retailInflow = String(data.retailInflow || data.retail_inflow || '0');
    const retailOutflow = String(data.retailOutflow || data.retail_outflow || '0');
    const retailNet = String(data.retailNet || data.retail_net || '0');

    if (mainInflow === '0' && mainOutflow === '0') return null;

    return {
      type: 'moneyflow',
      data: {
        code: astockCode,
        trade_date: new Date().toISOString().substring(0, 10),
        main_inflow: mainInflow,
        main_outflow: mainOutflow,
        main_net: mainNet,
        retail_inflow: retailInflow,
        retail_outflow: retailOutflow,
        retail_net: retailNet,
      },
    };
  } catch (e) {
    console.warn('[astock] 解析资金流向失败:', e);
    return null;
  }
}

/**
 * 解析实时行情数据
 * 从同花顺页面API提取实时报价
 */
function parseQuoteData(url, body, code) {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;

    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const data = json.data || json;

    // 尝试从不同格式的响应中提取行情数据
    const quote = {
      code: astockCode,
      name: data.name || data.secname || '',
      price: String(data.price || data.currentPrice || data.newprice || '0'),
      prev_close: String(data.prevClose || data.yesterdayClose || data.close || '0'),
      open: String(data.open || data.openPrice || '0'),
      high: String(data.high || data.highPrice || '0'),
      low: String(data.low || data.lowPrice || '0'),
      volume: parseInt(data.volume || data.totalVolume || 0),
      turnover: String(data.turnover || data.amount || data.totalAmount || '0'),
      bid: String(data.bid1 || data.buy1Price || '0'),
      ask: String(data.ask1 || data.sell1Price || '0'),
      change: String(data.change || data.priceChange || '0'),
      change_pct: String(data.changePercent || data.priceChangePercent || '0'),
      time: data.time || data.updateTime || new Date().toISOString().replace('T', ' ').substring(0, 19),
    };

    if (quote.price === '0') return null;

    return { type: 'quote', data: quote };
  } catch (e) {
    console.warn('[astock] 解析行情数据失败:', e);
    return null;
  }
}

/**
 * 从JSONP响应中提取JSON
 * 如: jQuery123({...})  => {...}
 */
function extractJsonFromJsonp(body) {
  if (!body) return null;

  let text = body.trim();

  // 尝试直接解析
  try {
    return JSON.parse(text);
  } catch (_) {}

  // 尝试去除JSONP回调
  const jsonpMatch = text.match(/^[a-zA-Z_$][a-zA-Z0-9_$]*\s*\(([\s\S]*)\)\s*;?\s*$/);
  if (jsonpMatch) {
    try {
      return JSON.parse(jsonpMatch[1]);
    } catch (_) {}
  }

  // 尝试找第一个 { 到最后一个 }
  const start = text.indexOf('{');
  const end = text.lastIndexOf('}');
  if (start !== -1 && end > start) {
    try {
      return JSON.parse(text.substring(start, end + 1));
    } catch (_) {}
  }

  return null;
}

/**
 * 根据URL判断数据类型并解析
 */
function normalizeData(url, body) {
  const code = extractCodeFromApiUrl(url);
  if (!code) return null;

  // 根据URL路径判断数据类型
  if (url.includes('/line/') && url.includes('/01/')) {
    // 分时数据
    return parseTimeshareData(url, body, code);
  }
  if (url.includes('/line/') && (url.includes('/11/') || url.includes('/21/'))) {
    // K线数据 (周线/月线)
    return parseKlineData(url, body, code);
  }
  if (url.includes('/line/') && !url.includes('/01/')) {
    // 日K线
    return parseKlineData(url, body, code);
  }
  if (url.includes('/moneyflow/')) {
    return parseMoneyFlowData(url, body, code);
  }
  if (url.includes('/trade/') || url.includes('/detail')) {
    // 成交明细/盘口
    return parseOrderBookData(url, body, code);
  }

  // 尝试作为行情数据解析
  return parseQuoteData(url, body, code);
}
