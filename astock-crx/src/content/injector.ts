/**
 * 内容脚本注入器 - 在隔离世界接收拦截数据并转发给background
 * interceptor.ts 通过 manifest.json 的 world: "MAIN" 独立注入到页面上下文
 */
import { STOCK_PAGE_REGEX } from '@shared/constants';

// 从页面接收拦截到的数据，转发给background
window.addEventListener('message', (event) => {
  if (event.source !== window) return;
  if (event.data.type !== 'ASTOCK_XHR_DATA' && event.data.type !== 'ASTOCK_FETCH_DATA' && event.data.type !== 'ASTOCK_JSONP_DATA' && event.data.type !== 'ASTOCK_SCRIPT_LOAD') return;
  chrome.runtime.sendMessage({
    type: 'CAPTURED_DATA',
    url: event.data.url,
    body: event.data.response,
    status: event.data.status,
  });
});

// 通知background当前页面的股票代码
function notifyStockCode() {
  const match = window.location.pathname.match(/\/(\d{6})\/?/);
  if (match) {
    chrome.runtime.sendMessage({
      type: 'PAGE_STOCK_CODE',
      code: match[1],
      url: window.location.href,
    });
  }
}

notifyStockCode();
debugger;
// 监听URL变化（SPA导航）
let lastUrl = window.location.href;
const urlObserver = new MutationObserver(() => {
  if (window.location.href !== lastUrl) {
    lastUrl = window.location.href;
    notifyStockCode();
  }
});
urlObserver.observe(document.body || document.documentElement, {
  childList: true,
  subtree: true,
});
console.log()
