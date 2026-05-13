/**
 * background.js - Service Worker
 * 编排数据流: 接收拦截数据 → 归一化 → 推送到astock
 */

// 导入模块 (service worker支持ES modules via manifest type:module)
importScripts(
  'lib/stock-code.js',
  'lib/data-normalizer.js',
  'lib/api-client.js'
);

// 推送节流状态
const pushState = {
  lastQuotePush: {},      // code -> timestamp
  lastOrderbookPush: {},  // code -> timestamp
  pendingTicks: {},       // code -> [tick, ...]
  tickTimer: {},          // code -> timerId
  pushedKline: new Set(), // code -> 已推送
  pushedMoneyflow: new Set(),
};

// 节流配置 (毫秒)
const THROTTLE = {
  quote: 10000,     // 行情: 10秒
  orderbook: 5000,  // 盘口: 5秒
  tickBatch: 5000,  // 成交: 5秒批量
};

// 当前活跃的股票页面
let currentPageCode = null;

// 推送日志 (最近20条)
const pushLog = [];
function logPush(entry) {
  pushLog.unshift(entry);
  if (pushLog.length > 20) pushLog.pop();
}

// 监听content script消息
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'CAPTURED_DATA') {
    handleCapturedData(message.url, message.body, message.status);
  } else if (message.type === 'PAGE_STOCK_CODE') {
    currentPageCode = message.code;
    // 切换股票时重置节流状态
    pushState.pushedKline.delete(message.code);
    pushState.pushedMoneyflow.delete(message.code);
  }
  return true;
});

// 处理拦截到的数据
function handleCapturedData(url, body, status) {
  if (status !== 200 || !body) return;

  // 使用data-normalizer解析
  const result = normalizeData(url, body);
  if (!result) return;

  const code = result.data?.code || currentPageCode;
  const now = Date.now();

  switch (result.type) {
    case 'quote': {
      const lastPush = pushState.lastQuotePush[code] || 0;
      if (now - lastPush >= THROTTLE.quote) {
        pushState.lastQuotePush[code] = now;
        pushQuote(result.data).then(r => {
          logPush({ time: new Date().toISOString(), type: 'quote', code, success: r.success });
          if (r.success) updateBadge(code, result.data.price);
        });
      }
      break;
    }

    case 'tick': {
      const ticks = result.data.ticks;
      if (!pushState.pendingTicks[code]) {
        pushState.pendingTicks[code] = [];
      }
      pushState.pendingTicks[code].push(...ticks);

      // 批量推送
      if (!pushState.tickTimer[code]) {
        pushState.tickTimer[code] = setTimeout(() => {
          const pending = pushState.pendingTicks[code] || [];
          if (pending.length > 0) {
            pushTicks(pending).then(r => {
              logPush({ time: new Date().toISOString(), type: 'tick', code, count: pending.length, success: r.success });
            });
          }
          pushState.pendingTicks[code] = [];
          pushState.tickTimer[code] = null;
        }, THROTTLE.tickBatch);
      }
      break;
    }

    case 'kline': {
      if (!pushState.pushedKline.has(code)) {
        pushState.pushedKline.add(code);
        pushKline(result.data.bars).then(r => {
          logPush({ time: new Date().toISOString(), type: 'kline', code, count: result.data.bars.length, success: r.success });
        });
      }
      break;
    }

    case 'orderbook': {
      const lastPush = pushState.lastOrderbookPush[code] || 0;
      if (now - lastPush >= THROTTLE.orderbook) {
        pushState.lastOrderbookPush[code] = now;
        pushOrderBook(result.data).then(r => {
          logPush({ time: new Date().toISOString(), type: 'orderbook', code, success: r.success });
        });
      }
      break;
    }

    case 'moneyflow': {
      if (!pushState.pushedMoneyflow.has(code)) {
        pushState.pushedMoneyflow.add(code);
        pushMoneyFlow(result.data).then(r => {
          logPush({ time: new Date().toISOString(), type: 'moneyflow', code, success: r.success });
        });
      }
      break;
    }
  }
}

// 更新插件图标Badge
function updateBadge(code, price) {
  const short = code.replace(/^(sh|sz|bj)/, '').substring(0, 2);
  chrome.action.setBadgeText({ text: short });
  chrome.action.setBadgeBackgroundColor({ color: '#4CAF50' });
}

// 定时检查连接状态
setInterval(async () => {
  const status = await checkStatus();
  if (status.connected) {
    chrome.action.setBadgeBackgroundColor({ color: '#4CAF50' });
  } else {
    chrome.action.setBadgeBackgroundColor({ color: '#F44336' });
  }
}, 30000);

// 暴露pushLog给popup查询
chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.type === 'GET_PUSH_LOG') {
    sendResponse(pushLog);
  } else if (msg.type === 'GET_STATUS') {
    checkStatus().then(sendResponse);
    return true;  // 异步响应
  } else if (msg.type === 'GET_CURRENT_CODE') {
    sendResponse({ code: currentPageCode });
  }
});
