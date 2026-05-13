# astock Bridge 数据流文档

## 整体架构

```
┌─────────────────────────────────────────────────────────────────────┐
│  同花顺网页 (stockpage.10jqka.com.cn/002202/)                      │
│  浏览器发起 XHR/Fetch 请求到 d.10jqka.com.cn 等API域名             │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ 网络请求 & 响应
                               ▼
┌──────────────────────────────────────────────────────────────────────┐
│  interceptor.ts (MAIN world)                                         │
│  Hook XMLHttpRequest + fetch，拦截目标域名响应                        │
│  通过 window.postMessage 发送到隔离世界                               │
└──────────────────────────────┬───────────────────────────────────────┘
                               │ window.postMessage
                               ▼
┌──────────────────────────────────────────────────────────────────────┐
│  injector.ts (Content Script, 隔离世界)                              │
│  接收 postMessage → chrome.runtime.sendMessage 转发给 Background     │
│  同时从页面URL提取股票代码，通知 Background 当前查看的股票            │
│  监听 SPA 导航，切换股票时自动通知                                    │
└──────────────────────────────┬───────────────────────────────────────┘
                               │ chrome.runtime.sendMessage
                               ▼
┌──────────────────────────────────────────────────────────────────────┐
│  background/index.ts (Service Worker)                                │
│  1. 接收 CAPTURED_DATA 消息                                         │
│  2. 调用 normalizeData() 归一化原始数据                               │
│  3. 按数据类型执行节流策略                                           │
│  4. 调用 api-client 推送到接收端                                     │
│  5. 记录推送日志 + 更新 Badge                                        │
└──────────────────────────────┬───────────────────────────────────────┘
                               │ HTTP POST (axios)
                               ▼
┌──────────────────────────────────────────────────────────────────────┐
│  astock receiver (axum HTTP服务, 127.0.0.1:17320)                   │
│  1. 解析 JSON 请求体                                                │
│  2. 验证股票代码                                                    │
│  3. String → Decimal 转换 (避免浮点精度问题)                         │
│  4. 写入 SQLite (WAL模式)                                           │
│  5. 行情数据触发预警检查 (price_above / price_below)                 │
└──────────────────────────────────────────────────────────────────────┘
```

## 拦截机制详解

### 1. 脚本注入顺序

```
manifest.json content_scripts (document_start)
  → injector.ts 执行
    → 创建 <script> 标签注入 interceptor.ts 到 MAIN world
    → interceptor.ts 在页面脚本之前 hook XHR 和 fetch
```

### 2. XHR 拦截

```typescript
class HookedXHR extends OriginalXHR {
  open(method, url, ...args)  // 记录请求URL
  send(...args)               // 监听 load 事件，响应后 postMessage
}
window.XMLHttpRequest = HookedXHR  // 替换全局 XHR
```

### 3. Fetch 拦截

```typescript
window.fetch = async function(input, init) {
  const response = await originalFetch(input, init);  // 先正常请求
  if (isTargetUrl(url)) {
    const cloned = response.clone();  // clone 避免消费原始响应
    cloned.text().then(body => window.postMessage({...}));
  }
  return response;  // 返回原始响应，不影响页面
}
```

### 4. 目标域名过滤

| 域名 | 用途 |
|------|------|
| `d.10jqka.com.cn` | 同花顺数据API (行情、K线、分时、盘口、资金) |
| `push2his.eastmoney.com` | 东方财富历史数据 |
| `push2.eastmoney.com` | 东方财富实时推送 |
| `qt.gtimg.cn` | 腾讯财经行情接口 |

### 5. 消息桥接

```
MAIN world (interceptor.ts)
  │ window.postMessage({ type: 'ASTOCK_XHR_DATA' / 'ASTOCK_FETCH_DATA', url, response, status })
  ▼
Isolated world (injector.ts)
  │ chrome.runtime.sendMessage({ type: 'CAPTURED_DATA', url, body, status })
  ▼
Background (index.ts)
  → handleCapturedData(url, body, status)
```

## 数据归一化

### URL → 数据类型映射

| URL特征 | 数据类型 | 解析函数 |
|---------|---------|---------|
| `/line/` + `/01/` | 分时成交 (tick) | `parseTimeshareData` |
| `/line/` + `/11/` 或 `/21/` | K线 (kline) | `parseKlineData` |
| `/line/` (其他) | K线 (kline) | `parseKlineData` |
| `/moneyflow/` | 资金流向 (moneyflow) | `parseMoneyFlowData` |
| `/trade/` 或 `/detail` | 盘口 (orderbook) | `parseOrderBookData` |
| 其他匹配 | 实时行情 (quote) | `parseQuoteData` |

### JSONP 解析

同花顺API响应多为 JSONP 格式: `callback({...})`，`extractJsonFromJsonp()` 依次尝试:
1. 直接 JSON.parse
2. 正则去掉回调函数名: `callback(json)` → 提取 `json`
3. 找第一个 `{` 到最后一个 `}` 提取

### 股票代码映射

同花顺API中使用 `hs_002202` 格式，astock 内部使用 `sz002202` 格式:

| 6位代码首位 | 交易所 | 前缀 | 示例 |
|------------|--------|------|------|
| 6 | 上海 | sh | 600519 → sh600519 |
| 0, 3 | 深圳 | sz | 002202 → sz002202 |
| 8, 4 | 北交所 | bj | 830799 → bj830799 |

## 采集数据类型

### 1. 实时行情 (Quote)

**推送端点**: `POST /api/v1/quote`
**节流策略**: 同一股票 10 秒间隔

| 字段 | 类型 | 说明 |
|------|------|------|
| code | string | 股票代码 (sz002202) |
| name | string? | 股票名称 |
| price | string | 当前价格 |
| prev_close | string? | 昨收价 |
| open | string? | 开盘价 |
| high | string? | 最高价 |
| low | string? | 最低价 |
| volume | number? | 成交量 (股) |
| turnover | string? | 成交额 |
| bid | string? | 买一价 |
| ask | string? | 卖一价 |
| change | string? | 涨跌额 |
| change_pct | string? | 涨跌幅 |
| time | string? | 行情时间 |

**SQLite 表**: `quote_snapshots`

**推送请求示例**:
```json
{
  "source": "10jqka",
  "timestamp": "2026-05-13T10:30:00.000Z",
  "data": {
    "code": "sz002202",
    "name": "金风科技",
    "price": "12.35",
    "prev_close": "12.10",
    "open": "12.15",
    "high": "12.50",
    "low": "12.08",
    "volume": 5238400,
    "turnover": "64567890.00",
    "bid": "12.34",
    "ask": "12.35",
    "change": "0.25",
    "change_pct": "2.07",
    "time": "2026-05-13 10:30:00"
  }
}
```

### 2. K线 (Kline)

**推送端点**: `POST /api/v1/kline`
**节流策略**: 每只股票页面加载时仅推送一次

| 字段 | 类型 | 说明 |
|------|------|------|
| code | string | 股票代码 |
| date | string | 日期 (YYYY-MM-DD) |
| open | string | 开盘价 |
| close | string | 收盘价 |
| high | string | 最高价 |
| low | string | 最低价 |
| volume | number? | 成交量 |
| turnover | string? | 成交额 |

**SQLite 表**: `daily_bars` (复用已有表，补充 turnover 字段)

**推送请求示例**:
```json
{
  "source": "10jqka",
  "timestamp": "2026-05-13T10:30:00.000Z",
  "data": {
    "bars": [
      { "code": "sz002202", "date": "2026-05-12", "open": "12.10", "close": "12.35", "high": "12.50", "low": "12.08", "volume": 5238400, "turnover": "64567890.00" },
      { "code": "sz002202", "date": "2026-05-11", "open": "11.95", "close": "12.10", "high": "12.20", "low": "11.90", "volume": 4120000, "turnover": "49800000.00" }
    ]
  }
}
```

### 3. 分时成交 (Tick)

**推送端点**: `POST /api/v1/tick`
**节流策略**: 5 秒窗口批量推送

| 字段 | 类型 | 说明 |
|------|------|------|
| code | string | 股票代码 |
| trade_time | string | 成交时间 (HH:MM:SS) |
| price | string | 成交价格 |
| volume | number | 成交量 (股) |
| direction | string? | 买卖方向 (buy/sell/neutral) |

**SQLite 表**: `tick_trades`

**推送请求示例**:
```json
{
  "source": "10jqka",
  "timestamp": "2026-05-13T10:30:05.000Z",
  "data": {
    "ticks": [
      { "code": "sz002202", "trade_time": "10:30:01", "price": "12.35", "volume": 500, "direction": "buy" },
      { "code": "sz002202", "trade_time": "10:30:02", "price": "12.34", "volume": 200, "direction": "sell" }
    ]
  }
}
```

### 4. 盘口5档 (OrderBook)

**推送端点**: `POST /api/v1/orderbook`
**节流策略**: 同一股票 5 秒间隔

| 字段 | 类型 | 说明 |
|------|------|------|
| code | string | 股票代码 |
| snap_time | string | 快照时间 |
| bid_prices | string[5] | 买1-5价格 |
| bid_volumes | number[5] | 买1-5挂单量 |
| ask_prices | string[5] | 卖1-5价格 |
| ask_volumes | number[5] | 卖1-5挂单量 |

**SQLite 表**: `order_book_snapshots` (bid/ask 数组 JSON 序列化存储)

**推送请求示例**:
```json
{
  "source": "10jqka",
  "timestamp": "2026-05-13T10:30:00.000Z",
  "data": {
    "code": "sz002202",
    "snap_time": "2026-05-13 10:30:00",
    "bid_prices": ["12.35", "12.34", "12.33", "12.32", "12.31"],
    "bid_volumes": [5200, 3100, 4800, 2200, 1500],
    "ask_prices": ["12.36", "12.37", "12.38", "12.39", "12.40"],
    "ask_volumes": [3800, 2600, 5100, 1900, 3200]
  }
}
```

### 5. 资金流向 (MoneyFlow)

**推送端点**: `POST /api/v1/moneyflow`
**节流策略**: 每只股票页面加载时仅推送一次

| 字段 | 类型 | 说明 |
|------|------|------|
| code | string | 股票代码 |
| trade_date | string | 交易日期 (YYYY-MM-DD) |
| main_inflow | string | 主力流入 |
| main_outflow | string | 主力流出 |
| main_net | string | 主力净流入 |
| retail_inflow | string | 散户流入 |
| retail_outflow | string | 散户流出 |
| retail_net | string | 散户净流入 |

**SQLite 表**: `money_flow` (UNIQUE 约束: code + trade_date，INSERT OR REPLACE)

**推送请求示例**:
```json
{
  "source": "10jqka",
  "timestamp": "2026-05-13T10:30:00.000Z",
  "data": {
    "code": "sz002202",
    "trade_date": "2026-05-13",
    "main_inflow": "5000000",
    "main_outflow": "3800000",
    "main_net": "1200000",
    "retail_inflow": "3800000",
    "retail_outflow": "5000000",
    "retail_net": "-1200000"
  }
}
```

## 接收端 API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/quote` | 接收实时行情 |
| POST | `/api/v1/kline` | 接收K线批量数据 |
| POST | `/api/v1/tick` | 接收分时成交批量数据 |
| POST | `/api/v1/orderbook` | 接收盘口快照 |
| POST | `/api/v1/moneyflow` | 接收资金流向 |
| GET | `/api/v1/status` | 健康检查 |
| GET | `/api/v1/watchlist` | 获取自选股列表 |

**通用响应格式**:
```json
{ "status": "ok" }
{ "status": "error", "message": "无效股票代码" }
```

**状态响应** (`GET /api/v1/status`):
```json
{ "status": "ok", "version": "0.1.0", "uptime_secs": 3600 }
```

## 节流策略汇总

| 数据类型 | 节流间隔 | 策略说明 |
|---------|---------|---------|
| quote (实时行情) | 10秒 | 同一股票10秒内只推送一次 |
| tick (分时成交) | 5秒窗口 | 5秒内累积成交，批量推送 |
| orderbook (盘口) | 5秒 | 同一股票5秒内只推送一次 |
| kline (K线) | 页面级一次 | 切换股票后重置，新股票可推 |
| moneyflow (资金流向) | 页面级一次 | 切换股票后重置，新股票可推 |

**重置机制**: 插件 popup 中点击"强制推送"按钮，或切换到新股票页面时，重置所有节流状态。

## 预警触发

接收端收到实时行情后自动检查预警规则:

1. 从 `alert_rules` 表查询该股票所有 `enabled=1` 的规则
2. 匹配 `signal_type`:
   - `price_above`: 当前价 > 目标价 → 触发通知
   - `price_below`: 当前价 < 目标价 → 触发通知
3. 触发时调用系统通知 + 写入 `alerts` 记录表

## 数据库表结构 (接收端新增)

### quote_snapshots — 行情快照

```sql
CREATE TABLE quote_snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL,
    name        TEXT DEFAULT '',
    price       TEXT NOT NULL,
    prev_close  TEXT NOT NULL,
    open        TEXT NOT NULL,
    high        TEXT NOT NULL,
    low         TEXT NOT NULL,
    volume      INTEGER NOT NULL DEFAULT 0,
    turnover    TEXT NOT NULL DEFAULT '0',
    bid         TEXT NOT NULL DEFAULT '0',
    ask         TEXT NOT NULL DEFAULT '0',
    change_val  TEXT NOT NULL DEFAULT '0',
    change_pct  TEXT NOT NULL DEFAULT '0',
    quote_time  TEXT DEFAULT '',
    source      TEXT NOT NULL DEFAULT '10jqka',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### tick_trades — 分时成交

```sql
CREATE TABLE tick_trades (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    code       TEXT NOT NULL,
    trade_time TEXT NOT NULL,
    price      TEXT NOT NULL,
    volume     INTEGER NOT NULL,
    direction  TEXT NOT NULL DEFAULT 'neutral',
    source     TEXT NOT NULL DEFAULT '10jqka',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### order_book_snapshots — 盘口快照

```sql
CREATE TABLE order_book_snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL,
    snap_time   TEXT NOT NULL,
    bid_prices  TEXT NOT NULL,  -- JSON: ["12.35","12.34",...]
    bid_volumes TEXT NOT NULL,  -- JSON: [5200,3100,...]
    ask_prices  TEXT NOT NULL,  -- JSON
    ask_volumes TEXT NOT NULL,  -- JSON
    source      TEXT NOT NULL DEFAULT '10jqka',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### money_flow — 资金流向

```sql
CREATE TABLE money_flow (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    code           TEXT NOT NULL,
    trade_date     TEXT NOT NULL,
    main_inflow    TEXT NOT NULL,
    main_outflow   TEXT NOT NULL,
    main_net       TEXT NOT NULL,
    retail_inflow  TEXT NOT NULL,
    retail_outflow TEXT NOT NULL,
    retail_net     TEXT NOT NULL,
    source         TEXT NOT NULL DEFAULT '10jqka',
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(code, trade_date)
);
```

## 使用流程

1. 启动 astock 接收端: `astock receiver --port 17320`
2. Chrome 加载 astock-crx 插件 (开发模式)
3. 打开同花顺股票页面: `https://stockpage.10jqka.com.cn/002202/`
4. 插件自动拦截 XHR/Fetch 请求，解析并推送到接收端
5. 点击插件 popup 查看连接状态和推送日志
6. 数据入库后可触发预警检查
