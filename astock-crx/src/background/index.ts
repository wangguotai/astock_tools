/**
 * Background Service Worker - 数据流编排
 * 接收拦截数据 → 归一化 → 推送到astock接收端
 */
import { normalizeData } from '@shared/data-normalizer';
import { pushQuote, pushKline, pushTicks, pushTimeshare, pushOrderBook, pushMoneyFlow, checkStatus } from '@shared/api-client';
import { appendPushLog, setCurrentCode, getCurrentCode } from '@shared/storage';
import { THROTTLE } from '@shared/constants';
import type { CapturedDataMessage, PageStockCodeMessage, PopupMessage, PushLogEntry } from '@shared/types';
import { getAlertRules, setAlertRules, getAlertHistory, setAlertHistory } from '@shared/storage';

// 推送节流状态
const pushState = {
  lastQuotePush: {} as Record<string, number>,
  lastOrderbookSnapshot: {} as Record<string, string>,
  sentTickIds: {} as Record<string, Set<string>>,
  pushedKline: new Set<string>(),
  pushedMoneyflow: new Set<string>(),
  pushedTimeshare: new Set<string>(),
};

// 内存推送日志 (最近20条，快速查询)
const memoryLog: PushLogEntry[] = [];

/** 北京时间字符串 (格式: YYYY-MM-DD HH:mm:ss) */
function beijingNow(): string {
  const now = new Date();
  const offset = 8 * 60; // 北京时间 UTC+8
  const localTime = new Date(now.getTime() + offset * 60 * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${localTime.getUTCFullYear()}-${pad(localTime.getUTCMonth() + 1)}-${pad(localTime.getUTCDate())} ${pad(localTime.getUTCHours())}:${pad(localTime.getUTCMinutes())}:${pad(localTime.getUTCSeconds())}`;
}

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
    pushState.pushedTimeshare.delete(msg.code);
    pushState.lastQuotePush[msg.code] = 0;
    pushState.lastOrderbookSnapshot[msg.code] = '';
    pushState.sentTickIds[msg.code] = new Set();
  } else if (msg.type === 'GET_PUSH_LOG') {
    sendResponse(memoryLog);
  } else if (msg.type === 'GET_STATUS') {
    checkStatus().then(sendResponse);
    return true;
  } else if (msg.type === 'GET_CURRENT_CODE') {
    getCurrentCode().then(sendResponse);
    return true;
  } else if (msg.type === 'FORCE_PUSH') {
    pushState.lastQuotePush = {};
    pushState.lastOrderbookSnapshot = {};
    pushState.sentTickIds = {};
    pushState.pushedKline.clear();
    pushState.pushedMoneyflow.clear();
    sendResponse({ status: 'ok' });
  } else if (msg.type === 'GET_ALERT_RULES') {
    getAlertRules().then(sendResponse);
    return true;
  } else if (msg.type === 'ADD_ALERT_RULE') {
    getAlertRules().then(rules => {
      const newRule = {
        id: Date.now(),
        code: msg.code,
        signal_type: msg.signal_type,
        params: msg.params,
        enabled: true,
      };
      rules.push(newRule);
      setAlertRules(rules).then(() => sendResponse({ status: 'ok', rule: newRule }));
    });
    return true;
  } else if (msg.type === 'DELETE_ALERT_RULE') {
    getAlertRules().then(rules => {
      const filtered = rules.filter(r => r.id !== msg.id);
      setAlertRules(filtered).then(() => sendResponse({ status: 'ok' }));
    });
    return true;
  } else if (msg.type === 'GET_ALERT_HISTORY') {
    getAlertHistory().then(sendResponse);
    return true;
  }

  return false;
});

/** 处理拦截到的数据 */
function handleCapturedData(url: string, body: string, status: number) {
  if (status !== 200 || !body) return;

  const result = normalizeData(url, body);
  if (!result) return;

  let code = '';
  if (result.type === 'kline') code = result.data.bars[0]?.code || '';
  else code = (result.data as any).code || '';
  const now = Date.now();

  switch (result.type) {
    case 'quote': {
      // 非交易时段（闭市、停牌等）跳过
      const status = (result.data as any).stock_status;
      if (status === '闭市' || status === '停牌' || !status) {
        break;
      }
      const lastPush = pushState.lastQuotePush[code] || 0;
      if (now - lastPush >= THROTTLE.quote) {
        pushState.lastQuotePush[code] = now;
        pushQuote(result.data).then((r) => {
          logPush({ time: beijingNow(), type: 'quote', code, success: r.status === 'ok' });
          updateBadge(code, result.data.price);
        }).catch(() => {
          logPush({ time: beijingNow(), type: 'quote', code, success: false });
        });
      }
      break;
    }

    case 'tick': {
      if (!pushState.sentTickIds[code]) pushState.sentTickIds[code] = new Set();

      // 过滤已发送的 tick（按 id 去重）
      const newTicks = result.data.ticks.filter((t: any) => {
        if (!t.id) return true; // 无 id 的原始 tick 不过滤
        if (pushState.sentTickIds[code].has(t.id)) return false;
        pushState.sentTickIds[code].add(t.id);
        return true;
      });

      if (newTicks.length === 0) break;

      // 限制 sentTickIds 集合大小（保留最近 500 条）
      const ids = Array.from(pushState.sentTickIds[code]);
      if (ids.length > 500) {
        pushState.sentTickIds[code] = new Set(ids.slice(-500));
      }

      // 实时发送：不使用批量延迟，直接推送
      pushTicks({ ticks: newTicks }).then((r) => {
        logPush({ time: beijingNow(), type: 'tick', code, count: newTicks.length, success: r.status === 'ok' });
      }).catch(() => {
        logPush({ time: beijingNow(), type: 'tick', code, count: newTicks.length, success: false });
      });
      break;
    }

    case 'timeshare': {
      if (!pushState.pushedTimeshare.has(code)) {
        pushState.pushedTimeshare.add(code);
        pushTimeshare(result.data).then((r) => {
          logPush({ time: beijingNow(), type: 'timeshare', code, count: result.data.points.length, success: r.status === 'ok' });
        }).catch(() => {
          logPush({ time: beijingNow(), type: 'timeshare', code, count: result.data.points.length, success: false });
        });
      }
      break;
    }

    case 'kline': {
      if (!pushState.pushedKline.has(code)) {
        pushState.pushedKline.add(code);
        pushKline(result.data).then((r) => {
          logPush({ time: beijingNow(), type: 'kline', code, count: result.data.bars.length, success: r.status === 'ok' });
        }).catch(() => {
          logPush({ time: beijingNow(), type: 'kline', code, count: result.data.bars.length, success: false });
        });
      }
      break;
    }

    case 'orderbook': {
      // 变化检测：只有五档数据变化时才推送
      const snapshot = [
        ...result.data.bid_prices,
        ...result.data.bid_volumes.map(String),
        ...result.data.ask_prices,
        ...result.data.ask_volumes.map(String),
      ].join(',');
      if (snapshot === pushState.lastOrderbookSnapshot[code]) {
        break; // 数据未变，跳过推送
      }
      pushState.lastOrderbookSnapshot[code] = snapshot;
      // 实时发送：不使用节流
      pushOrderBook(result.data).then((r) => {
        logPush({ time: beijingNow(), type: 'orderbook', code, success: r.status === 'ok' });
      }).catch(() => {
        logPush({ time: beijingNow(), type: 'orderbook', code, success: false });
      });
      break;
    }

    case 'moneyflow': {
      if (!pushState.pushedMoneyflow.has(code)) {
        pushState.pushedMoneyflow.add(code);
        pushMoneyFlow(result.data).then((r) => {
          logPush({ time: beijingNow(), type: 'moneyflow', code, success: r.status === 'ok' });
        }).catch(() => {
          logPush({ time: beijingNow(), type: 'moneyflow', code, success: false });
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
