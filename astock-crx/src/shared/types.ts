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
  price: string;          // 当前价
  prev_close?: string;     // 昨收价
  open?: string;           // 开盘价
  high?: string;           // 最高价
  low?: string;            // 最低价
  volume?: number;         // 成交量
  turnover?: string;       // 成交额
  bid?: string;            // 买一价
  ask?: string;            // 卖一价
  bid_vol?: number;        // 买一量
  ask_vol?: number;        // 卖一量
  change?: string;         // 涨跌额
  change_pct?: string;    // 涨跌幅
  high_limit?: string;     // 涨停价
  low_limit?: string;      // 跌停价
  inner_vol?: number;      // 内盘（主动卖出量）
  outer_vol?: number;      // 外盘（主动买入量）
  open_vol?: number;      // 开盘成交量
  time?: string;           // 更新时间
  update_time?: string;   // 同花顺 updateTime 字段
  stock_status?: string;   // 交易状态：连续竞价/闭市/休市
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

/** 分时历史数据批量 */
export interface TimeshareBatchData {
  points: TimesharePoint[];
}

/** 分时历史数据条目 */
export interface TimesharePoint {
  code: AstockCode;
  trade_time: string;
  price: string;
  volume: number;
  avg_price?: string;
  cum_volume?: number;
}

/** 分时成交条目 */
export interface TickEntry {
  id?: string; // 成交单号 (exchangedetail 的 1 字段)
  code: AstockCode;
  trade_time: string;
  price: string;
  volume: number;
  avg_price?: string;      // 成交均价 (v6/time parts[3])
  cum_volume?: number;      // 累计成交量 (v6/time parts[2])
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
  | { type: 'timeshare'; data: TimeshareBatchData }
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
  | { type: 'FORCE_PUSH' }
  | { type: 'GET_ALERT_RULES' }
  | { type: 'ADD_ALERT_RULE'; code: string; signal_type: AlertSignalType; params: AlertRuleParams }
  | { type: 'DELETE_ALERT_RULE'; id: number }
  | { type: 'GET_ALERT_HISTORY' }
  | { type: 'GET_WATCHLIST' };

/** 存储键 */
export interface StorageSchema {
  astock_server_url: string;
  astock_push_log: PushLogEntry[];
  astock_current_code: string | null;
}

/** 告警规则类型 */
export type AlertSignalType =
  | 'price_above'   // 价格突破上限
  | 'price_below'   // 价格跌破下限
  | 'MA_GOLDEN_CROSS'  // MA5 上穿 MA10 金叉
  | 'MA_DEAD_CROSS'    // MA5 下穿 MA10 死叉
  | 'RSI_OVERBOUGHT'   // RSI 超买
  | 'RSI_OVERSOLD'     // RSI 超卖
  | 'MACD_GOLDEN_CROSS' // MACD 金叉
  | 'MACD_DEAD_CROSS'   // MACD 死叉
  | 'MONEYFLOW_IN'     // 主力净流入
  | 'MONEYFLOW_OUT';   // 主力净流出

/** 告警规则参数 */
export interface AlertRuleParams {
  price?: number;        // price_above/below 阈值
  value?: number;         // RSI 阈值
  threshold?: number;     // 资金流向阈值（万元）
}

/** 告警规则 */
export interface AlertRule {
  id: number;
  code: string;
  signal_type: AlertSignalType;
  params: AlertRuleParams;
  enabled: boolean;
}

/** 告警历史 */
export interface AlertHistory {
  id: number;
  code: string;
  signal_type: string;
  message: string;
  triggered_at: string;
}
