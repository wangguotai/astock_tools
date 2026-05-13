/**
 * stock-code.js - 股票代码映射
 * 将6位数字代码转换为astock内部格式 (如 002202 -> sz002202)
 */

function toAstockCode(plain6) {
  if (!plain6 || plain6.length !== 6) return null;
  const first = plain6[0];
  let prefix;
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

function extractCodeFromUrl(url) {
  const match = url.match(/10jqka\.com\.cn\/(\d{6})/);
  return match ? match[1] : null;
}

function extractCodeFromApiUrl(url) {
  // 同花顺API URL中可能包含股票代码
  // 如 d.10jqka.com.cn/v6/line/hs_002202/01/last.js
  const match = url.match(/hs_(\d{6})/);
  if (match) return match[1];
  return null;
}
