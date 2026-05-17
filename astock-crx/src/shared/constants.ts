/**
 * 常量定义 - 同花顺API域名和URL模式
 */

/** 需要拦截的API域名 */
export const TARGET_DOMAINS = [
  'd.10jqka.com.cn',
  'push2his.eastmoney.com',
  'push2.eastmoney.com',
  'qt.gtimg.cn',
];

/** 判断URL是否为目标数据API */
export function isTargetUrl(url: string): boolean {
  if (!url) return false;
  return TARGET_DOMAINS.some((d) => url.includes(d));
}

/** 同花顺股票页面URL正则 */
export const STOCK_PAGE_REGEX = /10jqka\.com\.cn\/(\d{6})/;

/** 同花顺API中的股票代码正则 */
export const API_CODE_REGEX = /hs_(\d{6})/;

/** 节流配置 (毫秒) */
export const THROTTLE = {
  quote: 10_000,
} as const;

/** 默认astock接收端地址 */
export const DEFAULT_SERVER_URL = 'http://127.0.0.1:17320';
