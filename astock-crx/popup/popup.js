/**
 * popup.js - 插件弹窗逻辑
 */

document.addEventListener('DOMContentLoaded', async () => {
  // 加载服务器地址
  const result = await chrome.storage.local.get('serverUrl');
  document.getElementById('server-url').value = result.serverUrl || 'http://127.0.0.1:17320';

  // 检查连接状态
  await refreshStatus();

  // 获取当前股票
  chrome.runtime.sendMessage({ type: 'GET_CURRENT_CODE' }, (response) => {
    if (response && response.code) {
      document.getElementById('stock-info').textContent = `股票代码: ${response.code}`;
    }
  });

  // 获取推送日志
  chrome.runtime.sendMessage({ type: 'GET_PUSH_LOG' }, (log) => {
    renderLog(log || []);
  });

  // 保存按钮
  document.getElementById('save-btn').addEventListener('click', async () => {
    const url = document.getElementById('server-url').value.trim();
    await chrome.storage.local.set({ serverUrl: url });
    await refreshStatus();
  });
});

async function refreshStatus() {
  const dot = document.getElementById('status-dot');
  const text = document.getElementById('status-text');

  chrome.runtime.sendMessage({ type: 'GET_STATUS' }, (status) => {
    if (status && status.connected) {
      dot.className = 'status-dot connected';
      text.textContent = `已连接 (v${status.version}, 运行${status.uptime_secs}s)`;
    } else {
      dot.className = 'status-dot disconnected';
      text.textContent = '未连接 - 请启动 astock receiver';
    }
  });
}

function renderLog(log) {
  const container = document.getElementById('push-log');
  if (!log || log.length === 0) {
    container.textContent = '暂无记录';
    return;
  }

  container.innerHTML = log.map(entry => {
    const time = entry.time ? entry.time.substring(11, 19) : '';
    const cls = entry.success ? 'success' : 'fail';
    const info = entry.count ? ` (${entry.count}条)` : '';
    return `<div class="log-entry"><span class="${cls}">${entry.type}${info}</span> ${entry.code || ''} ${time}</div>`;
  }).join('');
}
