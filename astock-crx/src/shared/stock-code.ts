/**
 * 股票代码映射 - 6位数字转astock内部格式
 */

/** 将6位数字代码转为astock格式 (如 002202 -> sz002202) */
export function toAstockCode(plain6: string): string | null {
  if (!plain6 || plain6.length !== 6) return null;
  const first = plain6[0];
  let prefix: string;
  switch (first) {
    case '6': prefix = 'sh'; break;
    case '0':
    case '3': prefix = 'sz'; break;
    case '8':
    case '4': prefix = 'bj'; break;
    default: return null;
  }
  return prefix + plain6;
}

/** 从同花顺股票页面URL提取6位代码 */
export function extractCodeFromUrl(url: string): string | null {
  const match = url.match(/10jqka\.com\.cn\/(\d{6})/);
  return match ? match[1] : null;
}

/** 从同花顺API URL提取6位代码 */
export function extractCodeFromApiUrl(url: string): string | null {
  const match = url.match(/hs_(\d{6})/);
  return match ? match[1] : null;
}
