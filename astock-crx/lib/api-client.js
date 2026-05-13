/**
 * api-client.js - 向astock本地接收端推送数据
 */

const DEFAULT_SERVER = 'http://127.0.0.1:17320';

async function getServerUrl() {
  const result = await chrome.storage.local.get('serverUrl');
  return result.serverUrl || DEFAULT_SERVER;
}

async function pushData(endpoint, payload) {
  const serverUrl = await getServerUrl();
  const url = `${serverUrl}${endpoint}`;

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      console.warn(`[astock] 推送失败 ${endpoint}: ${response.status}`);
      return { success: false, status: response.status };
    }

    const result = await response.json();
    return { success: true, ...result };
  } catch (e) {
    console.warn(`[astock] 推送失败 ${endpoint}:`, e.message);
    return { success: false, error: e.message };
  }
}

async function pushQuote(data) {
  return pushData('/api/v1/quote', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data,
  });
}

async function pushKline(bars) {
  return pushData('/api/v1/kline', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data: { bars },
  });
}

async function pushTicks(ticks) {
  return pushData('/api/v1/tick', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data: { ticks },
  });
}

async function pushOrderBook(data) {
  return pushData('/api/v1/orderbook', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data,
  });
}

async function pushMoneyFlow(data) {
  return pushData('/api/v1/moneyflow', {
    source: '10jqka',
    timestamp: new Date().toISOString(),
    data,
  });
}

async function checkStatus() {
  const serverUrl = await getServerUrl();
  try {
    const response = await fetch(`${serverUrl}/api/v1/status`);
    if (!response.ok) return { connected: false };
    const result = await response.json();
    return { connected: true, ...result };
  } catch (e) {
    return { connected: false, error: e.message };
  }
}

async function getWatchlist() {
  const serverUrl = await getServerUrl();
  try {
    const response = await fetch(`${serverUrl}/api/v1/watchlist`);
    if (!response.ok) return [];
    return await response.json();
  } catch (e) {
    return [];
  }
}
