/**
 * inject.js - 注入到同花顺页面的桥接脚本
 * 将interceptor.js注入MAIN world，同时转发消息给background
 */

// 注入interceptor.js到MAIN world
const script = document.createElement('script');
script.src = chrome.runtime.getURL('content_scripts/interceptor.js');
script.onload = () => script.remove();
(document.head || document.documentElement).appendChild(script);

// 从页面接收拦截到的数据，转发给background service worker
window.addEventListener('message', (event) => {
  if (event.source !== window) return;
  if (event.data.type !== 'ASTOCK_XHR_DATA' && event.data.type !== 'ASTOCK_FETCH_DATA') return;

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
