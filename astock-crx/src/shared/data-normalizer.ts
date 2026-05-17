/**
 * 数据归一化 - 将同花顺API原始响应解析为astock格式
 */
import { toAstockCode, extractCodeFromApiUrl } from './stock-code';
import type {
  NormalizedData,
  QuoteData,
  KlineEntry,
  TickEntry,
  OrderBookData,
  MoneyFlowData,
  TradeDirection,
} from './types';

/** 从JSONP响应中提取JSON */
function extractJsonFromJsonp(body: string): any | null {
  if (!body) return null;
  const text = body.trim();

  // 尝试直接解析
  try { return JSON.parse(text); } catch {}

  // 去除JSONP回调: callback({...})
  const jsonpMatch = text.match(/^[a-zA-Z_$][a-zA-Z0-9_$]*\s*\(([\s\S]*)\)\s*;?\s*$/);
  if (jsonpMatch) {
    try { return JSON.parse(jsonpMatch[1]); } catch {}
  }

  // 找第一个 { 到最后一个 }
  const start = text.indexOf('{');
  const end = text.lastIndexOf('}');
  if (start !== -1 && end > start) {
    try { return JSON.parse(text.substring(start, end + 1)); } catch {}
  }

  return null;
}

/** 解析分时成交明细 (exchangedetail JSONP) */
function parseExchangeDetailData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const items = json.items || [];
    const ticks: TickEntry[] = [];

    for (const item of items) {
      // 字段映射 (来自同花顺 exchangedetail API):
      // 10 = 价格, 49 = 成交量, 12 = 买卖方向, 625295 = 成交额(元), His = 时间
      const price = String(item['10'] ?? '0');
      const volume = parseFloat(String(item['49'] ?? '0')) || 0;
      const hisTime = String(item['His'] ?? '');
      // 12: 1=主动卖, 5=主动买 (可能是红绿标记)
      const dirValue = parseInt(String(item['12'] ?? '0'));
      const direction: TradeDirection = dirValue === 5 ? 'buy' : dirValue === 1 ? 'sell' : 'neutral';

      if (price === '0' || !hisTime) continue;

      // His 格式: "14:56:30"
      const timeParts = hisTime.split(':');
      const trade_time = timeParts.length === 3 ? hisTime : '';

      ticks.push({
        code: astockCode,
        trade_time,
        price,
        volume,
        direction,
      });
    }

    return ticks.length > 0 ? { type: 'tick', data: { ticks } } : null;
  } catch {
    return null;
  }
}

/** 解析分时数据 */
function parseTimeshareData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const ticks: TickEntry[] = [];
    const trends = json?.data?.trends || [];
    for (const t of trends) {
      const parts = t.split(',');
      if (parts.length >= 3) {
        const timeStr = parts[0] as string;
        const hh = timeStr.substring(8, 10);
        const mm = timeStr.substring(10, 12);
        const ss = timeStr.substring(12, 14);
        ticks.push({
          code: astockCode,
          trade_time: `${hh}:${mm}:${ss}`,
          price: parts[1] as string,
          volume: parseInt(parts[2] as string) || 0,
          direction: 'neutral',
        });
      }
    }

    return ticks.length > 0 ? { type: 'tick', data: { ticks } } : null;
  } catch {
    return null;
  }
}

/** 解析K线数据 */
function parseKlineData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const bars: KlineEntry[] = [];
    const klineData = json.data || json;
    const dataKey = Object.keys(klineData).find((k) => Array.isArray((klineData as any)[k]));

    if (dataKey && Array.isArray((klineData as any)[dataKey])) {
      for (const item of (klineData as any)[dataKey]) {
        if (typeof item === 'string') {
          const parts = item.split(',');
          if (parts.length >= 6) {
            bars.push({
              code: astockCode,
              date: parts[0]!,
              open: parts[1]!,
              close: parts[2]!,
              high: parts[3]!,
              low: parts[4]!,
              volume: parseInt(parts[5]!) || 0,
              turnover: parts[6] || '0',
            });
          }
        } else if (Array.isArray(item) && item.length >= 6) {
          bars.push({
            code: astockCode,
            date: String(item[0]),
            open: String(item[1]),
            close: String(item[2]),
            high: String(item[3]),
            low: String(item[4]),
            volume: parseInt(item[5]) || 0,
            turnover: item[6] ? String(item[6]) : '0',
          });
        }
      }
    }

    return bars.length > 0 ? { type: 'kline', data: { bars } } : null;
  } catch {
    return null;
  }
}

/** 解析盘口数据 */
function parseOrderBookData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const data = json.data || json;
    const bids = data.bids || data.buy || [];
    const asks = data.asks || data.sell || [];

    const bidPrices: string[] = [];
    const bidVolumes: number[] = [];
    const askPrices: string[] = [];
    const askVolumes: number[] = [];

    for (let i = 0; i < 5 && i < bids.length; i++) {
      bidPrices.push(String(bids[i]?.price ?? bids[i]?.[0] ?? '0'));
      bidVolumes.push(parseInt(bids[i]?.volume ?? bids[i]?.[1] ?? 0));
    }
    for (let i = 0; i < 5 && i < asks.length; i++) {
      askPrices.push(String(asks[i]?.price ?? asks[i]?.[0] ?? '0'));
      askVolumes.push(parseInt(asks[i]?.volume ?? asks[i]?.[1] ?? 0));
    }

    while (bidPrices.length < 5) { bidPrices.push('0'); bidVolumes.push(0); }
    while (askPrices.length < 5) { askPrices.push('0'); askVolumes.push(0); }

    if (bidPrices[0] === '0' && askPrices[0] === '0') return null;

    const obData: OrderBookData = {
      code: astockCode,
      snap_time: new Date().toISOString().replace('T', ' ').substring(0, 19),
      bid_prices: bidPrices,
      bid_volumes: bidVolumes,
      ask_prices: askPrices,
      ask_volumes: askVolumes,
    };

    return { type: 'orderbook', data: obData };
  } catch {
    return null;
  }
}

/** 解析资金流向 */
function parseMoneyFlowData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const data = json.data || json;
    const mfData: MoneyFlowData = {
      code: astockCode,
      trade_date: new Date().toISOString().substring(0, 10),
      main_inflow: String(data.mainInflow ?? data.main_inflow ?? '0'),
      main_outflow: String(data.mainOutflow ?? data.main_outflow ?? '0'),
      main_net: String(data.mainNet ?? data.main_net ?? '0'),
      retail_inflow: String(data.retailInflow ?? data.retail_inflow ?? '0'),
      retail_outflow: String(data.retailOutflow ?? data.retail_outflow ?? '0'),
      retail_net: String(data.retailNet ?? data.retail_net ?? '0'),
    };

    if (mfData.main_inflow === '0' && mfData.main_outflow === '0') return null;
    return { type: 'moneyflow', data: mfData };
  } catch {
    return null;
  }
}

/** 解析实时行情 */
function parseQuoteData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const data = json.data || json;
    const quoteData: QuoteData = {
      code: astockCode,
      name: data.name ?? data.secname ?? '',
      price: String(data.price ?? data.currentPrice ?? data.newprice ?? '0'),
      prev_close: String(data.prevClose ?? data.yesterdayClose ?? data.close ?? '0'),
      open: String(data.open ?? data.openPrice ?? '0'),
      high: String(data.high ?? data.highPrice ?? '0'),
      low: String(data.low ?? data.lowPrice ?? '0'),
      volume: parseInt(data.volume ?? data.totalVolume ?? 0),
      turnover: String(data.turnover ?? data.amount ?? data.totalAmount ?? '0'),
      bid: String(data.bid1 ?? data.buy1Price ?? '0'),
      ask: String(data.ask1 ?? data.sell1Price ?? '0'),
      change: String(data.change ?? data.priceChange ?? '0'),
      change_pct: String(data.changePercent ?? data.priceChangePercent ?? '0'),
      time: data.time ?? data.updateTime ?? new Date().toISOString().replace('T', ' ').substring(0, 19),
    };

    if (quoteData.price === '0') return null;
    return { type: 'quote', data: quoteData };
  } catch {
    return null;
  }
}

/** 根据URL判断数据类型并解析 */
export function normalizeData(url: string, body: string): NormalizedData | null {
  const code = extractCodeFromApiUrl(url);
  if (!code) return null;

  if (url.includes('/exchangedetail/')) {
    return parseExchangeDetailData(url, body, code);
  }
  if (url.includes('/line/') && url.includes('/01/')) {
    return parseTimeshareData(url, body, code);
  }
  if (url.includes('/line/') && (url.includes('/11/') || url.includes('/21/'))) {
    return parseKlineData(url, body, code);
  }
  if (url.includes('/line/')) {
    return parseKlineData(url, body, code);
  }
  if (url.includes('/moneyflow/')) {
    return parseMoneyFlowData(url, body, code);
  }
  if (url.includes('/trade/') || url.includes('/detail')) {
    return parseOrderBookData(url, body, code);
  }

  return parseQuoteData(url, body, code);
}
