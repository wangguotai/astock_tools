/**
 * 共享类型定义
 */

/** 股票代码格式 (astock内部: sz002202) */
export type AstockCode = string;

/** 数据来源 */
export type DataSource = '10jqka' | 'eastmoney' | 'tencent';

/** 交易方向 */
export type TradeDirection = 'buy' | 'sell' | 'neutral';

/** 实时行情数据 */
export interface QuoteData {
  code: AstockCode;
  name?: string;
  price: string;
  prev_close?: string;
  open?: string;
  high?: string;
  low?: string;
  volume?: number;
  turnover?: string;
  bid?: string;
  ask?: string;
  change?: string;
  change_pct?: string;
  time?: string;
}

/** K线条目 */
export interface KlineEntry {
  code: AstockCode;
  date: string;
  open: string;
  close: string;
  high: string;
  low: string;
  volume?: number;
  turnover?: string;
}

/** K线批量 */
export interface KlineBatchData {
  bars: KlineEntry[];
}

/** 分时成交条目 */
export interface TickEntry {
  code: AstockCode;
  trade_time: string;
  price: string;
  volume: number;
  direction?: TradeDirection;
}

/** 分时成交批量 */
export interface TickBatchData {
  ticks: TickEntry[];
}

/** 盘口5档快照 */
export interface OrderBookData {
  code: AstockCode;
  snap_time: string;
  bid_prices: string[];
  bid_volumes: number[];
  ask_prices: string[];
  ask_volumes: number[];
}

/** 资金流向 */
export interface MoneyFlowData {
  code: AstockCode;
  trade_date: string;
  main_inflow: string;
  main_outflow: string;
  main_net: string;
  retail_inflow: string;
  retail_outflow: string;
  retail_net: string;
}

/** 推送请求通用结构 */
export interface PushRequest<T> {
  source: DataSource;
  timestamp?: string;
  data: T;
}

/** API响应 */
export interface ApiResponse {
  status: string;
  message?: string;
}

/** 状态响应 */
export interface StatusResponse {
  status: string;
  version: string;
  uptime_secs: number;
}

/** 自选股条目 */
export interface WatchlistEntry {
  code: string;
  name: string;
}

/** 推送日志条目 */
export interface PushLogEntry {
  time: string;
  type: string;
  code?: string;
  count?: number;
  success: boolean;
}

/** 归一化后的数据包 */
export type NormalizedData =
  | { type: 'quote'; data: QuoteData }
  | { type: 'kline'; data: KlineBatchData }
  | { type: 'tick'; data: TickBatchData }
  | { type: 'orderbook'; data: OrderBookData }
  | { type: 'moneyflow'; data: MoneyFlowData };

/** Content Script → Background 消息 */
export interface CapturedDataMessage {
  type: 'CAPTURED_DATA';
  url: string;
  body: string;
  status: number;
}

/** 页面股票代码消息 */
export interface PageStockCodeMessage {
  type: 'PAGE_STOCK_CODE';
  code: string;
  url: string;
}

/** Popup → Background 消息 */
export type PopupMessage =
  | { type: 'GET_PUSH_LOG' }
  | { type: 'GET_STATUS' }
  | { type: 'GET_CURRENT_CODE' }
  | { type: 'FORCE_PUSH' };

/** 存储键 */
export interface StorageSchema {
  astock_server_url: string;
  astock_push_log: PushLogEntry[];
  astock_current_code: string | null;
}
