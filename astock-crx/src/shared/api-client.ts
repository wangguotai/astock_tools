/**
 * HTTP客户端 - 向astock接收端推送数据
 */
import axios from 'axios';
import type {
  QuoteData,
  KlineBatchData,
  TickBatchData,
  OrderBookData,
  MoneyFlowData,
  ApiResponse,
  StatusResponse,
  WatchlistEntry,
} from './types';
import { getServerUrl } from './storage';

/** 获取已配置的axios实例 */
async function getClient() {
  const serverUrl = await getServerUrl();
  return axios.create({
    baseURL: serverUrl,
    headers: { 'Content-Type': 'application/json' },
    timeout: 5000,
  });
}

/** 推送实时行情 */
export async function pushQuote(data: QuoteData): Promise<ApiResponse> {
  const client = await getClient();
  const res = await client.post('/api/v1/quote', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data,
  });
  return res.data;
}

/** 推送K线批量 */
export async function pushKline(data: KlineBatchData): Promise<ApiResponse> {
  const client = await getClient();
  const res = await client.post('/api/v1/kline', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data,
  });
  return res.data;
}

/** 推送分时成交批量 */
export async function pushTicks(data: TickBatchData): Promise<ApiResponse> {
  const client = await getClient();
  const res = await client.post('/api/v1/tick', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data,
  });
  return res.data;
}

/** 推送盘口快照 */
export async function pushOrderBook(data: OrderBookData): Promise<ApiResponse> {
  const client = await getClient();
  const res = await client.post('/api/v1/orderbook', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data,
  });
  return res.data;
}

/** 推送资金流向 */
export async function pushMoneyFlow(data: MoneyFlowData): Promise<ApiResponse> {
  const client = await getClient();
  const res = await client.post('/api/v1/moneyflow', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data,
  });
  return res.data;
}

/** 检查接收端状态 */
export async function checkStatus(): Promise<StatusResponse & { connected: boolean }> {
  try {
    const serverUrl = await getServerUrl();
    const res = await axios.get(`${serverUrl}/api/v1/status`, { timeout: 3000 });
    return { connected: true, ...res.data };
  } catch {
    return { connected: false, status: 'error', version: '', uptime_secs: 0 };
  }
}

/** 获取自选股列表 */
export async function getWatchlist(): Promise<WatchlistEntry[]> {
  try {
    const serverUrl = await getServerUrl();
    const res = await axios.get(`${serverUrl}/api/v1/watchlist`, { timeout: 3000 });
    return res.data;
  } catch {
    return [];
  }
}
