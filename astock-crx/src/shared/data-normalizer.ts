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

/** 解析五档数据 (fiverange JSONP) */
function parseFiverangeData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const items = json.items || json;
    // 同花顺字段映射 (fiverange):
    // 24=买一价, 25=买一量, 26=买二价, 27=买二量, 28=买三价, 29=买三量
    // 150=买四价, 151=买四量, 154=买五价, 155=买五量
    // 30=卖一价, 31=卖一量, 32=卖二价, 33=卖二量, 34=卖三价, 35=卖三量
    // 152=卖四价, 153=卖四量, 156=卖五价, 157=卖五量
    const bidPrices = [
      String(items['24'] ?? '0'),
      String(items['26'] ?? '0'),
      String(items['28'] ?? '0'),
      String(items['150'] ?? '0'),
      String(items['154'] ?? '0'),
    ];
    const bidVolumes = [
      parseFloat(String(items['25'] ?? '0')) || 0,
      parseFloat(String(items['27'] ?? '0')) || 0,
      parseFloat(String(items['29'] ?? '0')) || 0,
      parseFloat(String(items['151'] ?? '0')) || 0,
      parseFloat(String(items['155'] ?? '0')) || 0,
    ];
    const askPrices = [
      String(items['30'] ?? '0'),
      String(items['32'] ?? '0'),
      String(items['34'] ?? '0'),
      String(items['152'] ?? '0'),
      String(items['156'] ?? '0'),
    ];
    const askVolumes = [
      parseFloat(String(items['31'] ?? '0')) || 0,
      parseFloat(String(items['33'] ?? '0')) || 0,
      parseFloat(String(items['35'] ?? '0')) || 0,
      parseFloat(String(items['153'] ?? '0')) || 0,
      parseFloat(String(items['157'] ?? '0')) || 0,
    ];

    // 全部为0说明解析失败
    const allZero = [...bidPrices, ...askPrices].every(p => p === '0');
    if (allZero) return null;

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

/** 解析分时成交明细 (exchangedetail JSONP) */
function parseExchangeDetailData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    const rawItems = json.items;
    if (!rawItems || !Array.isArray(rawItems) || rawItems.length === 0) return null;

    const ticks: TickEntry[] = [];

    for (const item of rawItems) {
      const tickId = String(item['1'] ?? '');
      const price = String(item['10'] ?? '0');
      const volume = parseFloat(String(item['49'] ?? '0')) || 0;
      const hisTime = String(item['His'] ?? '');
      const dirValue = parseInt(String(item['12'] ?? '0'));
      const direction: TradeDirection = dirValue === 5 ? 'buy' : dirValue === 1 ? 'sell' : 'neutral';

      if (price === '0' || !hisTime) continue;

      ticks.push({
        id: tickId || undefined,
        code: astockCode,
        trade_time: hisTime,
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

/** 解析分时数据 (v6/time 接口) */
function parseTimeData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    // v6/time 返回结构: { "hs_002202": { data: "0930,25.00,21022500,...", ... } }
    const key = `hs_${code}`;
    const stockData = json[key] || json;
    const rawData = String(stockData.data || '');
    if (!rawData) return null;

    const ticks: TickEntry[] = [];
    const records = rawData.split(';');
    for (const rec of records) {
      if (!rec) continue;
      const parts = rec.split(',');
      if (parts.length >= 5) {
        const timeStr = parts[0] as string;
        const price = parts[1] as string;
        const cumVolume = parseInt(parts[2] as string) || 0;
        const avgPrice = parts[3] as string;
        const volume = parseInt(parts[4] as string) || 0;
        const hh = timeStr.substring(0, 2);
        const mm = timeStr.substring(2, 4);
        ticks.push({
          code: astockCode,
          trade_time: `${hh}:${mm}:00`,
          price,
          volume,
          avg_price: avgPrice,
          cum_volume: cumVolume,
          direction: 'neutral',
        });
      }
    }

    return ticks.length > 0 ? { type: 'timeshare', data: { points: ticks } } : null;
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

/** 解析资金流向 (realFunds API 或旧 moneyflow JSONP) */
function parseMoneyFlowData(url: string, body: string, code: string): NormalizedData | null {
  try {
    const json = extractJsonFromJsonp(body);
    if (!json) return null;
    const astockCode = toAstockCode(code);
    if (!astockCode) return null;

    // realFunds API: { flash: [...], title: { zlr, zlc, je }, field: {...} }
    if (json.title) {
      const flash = json.flash || [];
      // flash 顺序: 大单流出, 中单流出, 小单流出, 小单流入, 中单流入, 大单流入
      const bigOut  = flash.find((f: any) => f.name === '大单流出');
      const bigIn   = flash.find((f: any) => f.name === '大单流入');
      const midOut  = flash.find((f: any) => f.name === '中单流出');
      const midIn   = flash.find((f: any) => f.name === '中单流入');
      const smlOut  = flash.find((f: any) => f.name === '小单流出');
      const smlIn   = flash.find((f: any) => f.name === '小单流入');

      const mainIn  = parseFloat(bigIn?.sr ?? '0') || 0;
      const mainOut = parseFloat(bigOut?.sr ?? '0') || 0;
      // 散户 = 小单
      const retailIn  = parseFloat(smlIn?.sr ?? '0') || 0;
      const retailOut = parseFloat(smlOut?.sr ?? '0') || 0;

      const mfData: MoneyFlowData = {
        code: astockCode,
        trade_date: new Date().toISOString().substring(0, 10),
        main_inflow: String(mainIn),
        main_outflow: String(mainOut),
        main_net: String(mainIn - mainOut),
        retail_inflow: String(retailIn),
        retail_outflow: String(retailOut),
        retail_net: String(retailIn - retailOut),
      };

      if (mfData.main_inflow === '0' && mfData.main_outflow === '0') return null;
      return { type: 'moneyflow', data: mfData };
    }

    // 旧 moneyflow JSONP: { "hs_002202": { data: {...} } }
    const mfKey = `hs_${code}`;
    const mfStockData = json[mfKey] || json;
    const data = mfStockData.data || mfStockData;
    if (!data || typeof data === 'string') return null;
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

    // realhead 返回结构: { items: { "10": "25.79", "6": "25.09", ... }, time: "...", name: "..." }
    // items 是扁平 key-value，其他字段在根上
    const root = json;
    const it = root.items || root;

    const quoteData: QuoteData = {
      code: astockCode,
      name: String(root.name ?? it.name ?? ''),
      price: String(it['10'] ?? '0'),
      prev_close: String(it['6'] ?? '0'),
      open: String(it['7'] ?? '0'),
      high: String(it['8'] ?? '0'),
      low: String(it['9'] ?? '0'),
      volume: parseInt(it['13'] ?? '0') || 0,
      turnover: String(it['19'] ?? '0'),
      bid: String(it['24'] ?? '0'),
      ask: String(it['30'] ?? '0'),
      bid_vol: parseInt(it['25'] ?? '0') || undefined,
      ask_vol: parseInt(it['31'] ?? '0') || undefined,
      change: String(it['199112'] ?? '0'),
      change_pct: String(it['264648'] ?? '0'),
      high_limit: String(it['69'] ?? '0'),
      low_limit: String(it['70'] ?? '0'),
      inner_vol: parseInt(it['223'] ?? '0') || undefined,
      outer_vol: parseInt(it['224'] ?? '0') || undefined,
      open_vol: parseInt(it['15'] ?? '0') || undefined,
      time: String(root.time ?? it.time ?? ''),
      update_time: String(root.updateTime ?? it.updateTime ?? ''),
      stock_status: String(root.stockStatus ?? it.stockStatus ?? ''),
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

  if (url.includes('/fiverange/')) {
    return parseFiverangeData(url, body, code);
  }
  if (url.includes('/exchangedetail/')) {
    return parseExchangeDetailData(url, body, code);
  }
  if (url.includes('/v6/time/')) {
    return parseTimeData(url, body, code);
  }
  if (url.includes('/Funds/realFunds')) {
    return parseMoneyFlowData(url, body, code);
  }
  return parseQuoteData(url, body, code);
}
