/**
 * chrome.storage.local 封装
 */
import type { StorageSchema, PushLogEntry, AlertRule, AlertHistory } from './types';
import { DEFAULT_SERVER_URL } from './constants';

/** 获取服务器地址 */
export async function getServerUrl(): Promise<string> {
  const result = await chrome.storage.local.get('astock_server_url');
  return result.astock_server_url || DEFAULT_SERVER_URL;
}

/** 设置服务器地址 */
export async function setServerUrl(url: string): Promise<void> {
  await chrome.storage.local.set({ astock_server_url: url });
}

/** 获取推送日志 */
export async function getPushLog(): Promise<PushLogEntry[]> {
  const result = await chrome.storage.local.get('astock_push_log');
  return result.astock_push_log || [];
}

/** 追加推送日志 (保留最近50条) */
export async function appendPushLog(entry: PushLogEntry): Promise<void> {
  const log = await getPushLog();
  log.unshift(entry);
  if (log.length > 50) log.length = 50;
  await chrome.storage.local.set({ astock_push_log: log });
}

/** 获取当前股票代码 */
export async function getCurrentCode(): Promise<string | null> {
  const result = await chrome.storage.local.get('astock_current_code');
  return result.astock_current_code || null;
}

/** 设置当前股票代码 */
export async function setCurrentCode(code: string | null): Promise<void> {
  await chrome.storage.local.set({ astock_current_code: code });
}

/** 获取告警规则缓存 */
export async function getAlertRules(): Promise<AlertRule[]> {
  const result = await chrome.storage.local.get('astock_alert_rules');
  return result.astock_alert_rules || [];
}

/** 设置告警规则缓存 */
export async function setAlertRules(rules: AlertRule[]): Promise<void> {
  await chrome.storage.local.set({ astock_alert_rules: rules });
}

/** 获取告警历史缓存 */
export async function getAlertHistory(): Promise<AlertHistory[]> {
  const result = await chrome.storage.local.get('astock_alert_history');
  return result.astock_alert_history || [];
}

/** 设置告警历史缓存 */
export async function setAlertHistory(history: AlertHistory[]): Promise<void> {
  await chrome.storage.local.set({ astock_alert_history: history });
}
