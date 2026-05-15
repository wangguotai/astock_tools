/**
 * Background Service Worker - 数据流编排
 * 接收拦截数据 → 归一化 → 推送到astock接收端
 */
import { normalizeData } from '@shared/data-normalizer';
import { pushQuote, pushKline, pushTicks, pushOrderBook, pushMoneyFlow, checkStatus } from '@shared/api-client';
import { appendPushLog, setCurrentCode, getCurrentCode } from '@shared/storage';
import { THROTTLE } from '@shared/constants';
import type { CapturedDataMessage, PageStockCodeMessage, PopupMessage, PushLogEntry } from '@shared/types';

// 推送节流状态
const pushState = {
  lastQuotePush: {} as Record<string, number>,
  lastOrderbookPush: {} as Record<string, number>,
  pendingTicks: {} as Record<string, any[]>,
  tickTimer: {} as Record<string, ReturnType<typeof setTimeout> | null>,
  pushedKline: new Set<string>(),
  pushedMoneyflow: new Set<string>(),
};

// 内存推送日志 (最近20条，快速查询)
const memoryLog: PushLogEntry[] = [];

function logPush(entry: PushLogEntry) {
  memoryLog.unshift(entry);
  if (memoryLog.length > 20) memoryLog.pop();
  appendPushLog(entry); // 持久化
}

// 监听content script消息
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  const msg = message as CapturedDataMessage | PageStockCodeMessage | PopupMessage;
  if (msg.type === 'CAPTURED_DATA') {
    handleCapturedData(msg.url, msg.body, msg.status);
  } else if (msg.type === 'PAGE_STOCK_CODE') {
    setCurrentCode(msg.code);
    pushState.pushedKline.delete(msg.code);
    pushState.pushedMoneyflow.delete(msg.code);
  } else if (msg.type === 'GET_PUSH_LOG') {
    sendResponse(memoryLog);
  } else if (msg.type === 'GET_STATUS') {
    checkStatus().then(sendResponse);
    return true; // 异步响应
  } else if (msg.type === 'GET_CURRENT_CODE') {
    getCurrentCode().then(sendResponse);
    return true;
  } else if (msg.type === 'FORCE_PUSH') {
    pushState.lastQuotePush = {};
    pushState.lastOrderbookPush = {};
    pushState.pushedKline.clear();
    pushState.pushedMoneyflow.clear();
    sendResponse({ status: 'ok' });
  }

  return false;
});

/** 处理拦截到的数据 */
function handleCapturedData(url: string, body: string, status: number) {
  debugger;
  if (status !== 200 || !body) return;

  const result = normalizeData(url, body);
  if (!result) return;

  let code = '';
  if (result.type === 'kline') code = result.data.bars[0]?.code || '';
  else code = (result.data as any).code || '';
  const now = Date.now();

  switch (result.type) {
    case 'quote': {
      const lastPush = pushState.lastQuotePush[code] || 0;
      if (now - lastPush >= THROTTLE.quote) {
        pushState.lastQuotePush[code] = now;
        pushQuote(result.data).then((r) => {
          logPush({ time: new Date().toISOString(), type: 'quote', code, success: r.status === 'ok' });
          updateBadge(code, result.data.price);
        }).catch(() => {
          logPush({ time: new Date().toISOString(), type: 'quote', code, success: false });
        });
      }
      break;
    }

    case 'tick': {
      if (!pushState.pendingTicks[code]) pushState.pendingTicks[code] = [];
      pushState.pendingTicks[code].push(...result.data.ticks);

      if (!pushState.tickTimer[code]) {
        pushState.tickTimer[code] = setTimeout(() => {
          const pending = pushState.pendingTicks[code] || [];
          if (pending.length > 0) {
            pushTicks({ ticks: pending }).then((r) => {
              logPush({ time: new Date().toISOString(), type: 'tick', code, count: pending.length, success: r.status === 'ok' });
            }).catch(() => {
              logPush({ time: new Date().toISOString(), type: 'tick', code, count: pending.length, success: false });
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
        pushKline(result.data).then((r) => {
          logPush({ time: new Date().toISOString(), type: 'kline', code, count: result.data.bars.length, success: r.status === 'ok' });
        }).catch(() => {
          logPush({ time: new Date().toISOString(), type: 'kline', code, count: result.data.bars.length, success: false });
        });
      }
      break;
    }

    case 'orderbook': {
      const lastPush = pushState.lastOrderbookPush[code] || 0;
      if (now - lastPush >= THROTTLE.orderbook) {
        pushState.lastOrderbookPush[code] = now;
        pushOrderBook(result.data).then((r) => {
          logPush({ time: new Date().toISOString(), type: 'orderbook', code, success: r.status === 'ok' });
        }).catch(() => {
          logPush({ time: new Date().toISOString(), type: 'orderbook', code, success: false });
        });
      }
      break;
    }

    case 'moneyflow': {
      if (!pushState.pushedMoneyflow.has(code)) {
        pushState.pushedMoneyflow.add(code);
        pushMoneyFlow(result.data).then((r) => {
          logPush({ time: new Date().toISOString(), type: 'moneyflow', code, success: r.status === 'ok' });
        }).catch(() => {
          logPush({ time: new Date().toISOString(), type: 'moneyflow', code, success: false });
        });
      }
      break;
    }
  }
}

/** 更新插件Badge */
function updateBadge(code: string, _price?: string) {
  const short = code.replace(/^(sh|sz|bj)/, '').substring(0, 2);
  chrome.action.setBadgeText({ text: short });
  chrome.action.setBadgeBackgroundColor({ color: '#4CAF50' });
}

// 定时检查连接状态
setInterval(async () => {
  const status = await checkStatus();
  chrome.action.setBadgeBackgroundColor({
    color: status.connected ? '#4CAF50' : '#F44336',
  });
}, 30_000);

// 初始检查
checkStatus();
