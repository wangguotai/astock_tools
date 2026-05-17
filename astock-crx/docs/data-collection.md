# 同花顺数据采集文档

## 数据源

插件拦截 `d.10jqka.com.cn` 域名的 JSONP 请求，通过以下方式获取数据：

1. **JSONP 回调 hook** — 当 URL 包含 `callback` 参数时，覆写全局回调函数捕获响应
2. **script onload + fetch** — 对 `.js` 结尾的 JSONP，直接 fetch 请求 URL 获取响应体

---

## 去重机制

为保证数据准确性，避免重复推送，采用以下策略：

| 数据类型 | 去重策略 |
|---------|---------|
| tick (exchangedetail) | 按成交单号 `id` 去重，已发送的 ID 记录在 Set 中（上限 500 条，超出后淘汰最老的） |
| orderbook (fiverange) | 变化检测，五档价格+数量与上次快照对比，无变化则跳过推送 |
| quote | 节流：10s 内同一股票不重复推送 |
| kline / moneyflow | 只推送一次（首次加载时） |

---

## 采集的数据类型

### 1. 实时行情 (quote)

| 字段 | 说明 |
|------|------|
| code | 股票代码 (sz002202 格式) |
| name | 股票名称 |
| price | 当前价格 |
| prev_close | 昨收价 |
| open | 开盘价 |
| high | 最高价 |
| low | 最低价 |
| volume | 成交量 |
| turnover | 成交额 |
| bid / ask | 买一价 / 卖一价 |
| change | 涨跌额 |
| change_pct | 涨跌幅 |
| time | 更新时间 |

**来源 URL**: `https://d.10jqka.com.cn/v2/realhead/hs_{code}/last.js`
**归一化函数**: `parseQuoteData()`
**推送 API**: `POST /api/v1/quote`
**节流**: 5s 内不重复推送同一股票

---

### 2. 五档盘口 (orderbook)

| 字段 | 说明 |
|------|------|
| code | 股票代码 |
| snap_time | 快照时间 |
| bid_prices[5] | 买一~买五价格 |
| bid_volumes[5] | 买一~买五量 |
| ask_prices[5] | 卖一~卖五价格 |
| ask_volumes[5] | 卖一~卖五量 |

**来源 URL**: `https://d.10jqka.com.cn/v2/fiverange/hs_{code}/last.js`
**归一化函数**: `parseFiverangeData()`

字段映射:

```
买: 24=价/25=量, 26=价/27=量, 28=价/29=量, 150=价/151=量, 154=价/155=量
卖: 30=价/31=量, 32=价/33=量, 34=价/35=量, 152=价/153=量, 156=价/157=量
```

**推送 API**: `POST /api/v1/orderbook`
**节流**: 3s 内不重复推送同一股票

---

### 3. 分时成交明细 (tick)

| 字段 | 说明 |
|------|------|
| code | 股票代码 |
| trade_time | 成交时间 (HH:mm:ss) |
| price | 成交价格 |
| volume | 成交量 |
| direction | 方向 (buy/sell/neutral) |

**来源 URL**: `https://d.10jqka.com.cn/v2/exchangedetail/hs_{code}/last12.js`
**归一化函数**: `parseExchangeDetailData()`

字段映射:

```
10 = 价格, 49 = 成交量, 12 = 方向(5=买,1=卖), His = 时间
```

**推送 API**: `POST /api/v1/tick`
**批量**: 攒够一批后 2s 内推送一次

---

### 4. 分时历史数据 (tick)

来自分时走势图的历史数据点，每分钟一个点。

| 字段 | 说明 |
|------|------|
| code | 股票代码 |
| trade_time | 时间 (HH:mm:ss) |
| price | 价格 |
| volume | 成交量 |
| direction | 固定 neutral |

**来源 URL**: `https://d.10jqka.com.cn/v2/line/hs_{code}/01/last.js`
**归一化函数**: `parseTimeshareData()`
**数据结构**: `data.trends` 数组，每项格式 `YYYYMMDDHHmmss,price,volume`

---

### 5. K线数据 (kline)

| 字段 | 说明 |
|------|------|
| code | 股票代码 |
| date | 日期 |
| open / close / high / low | 开盘/收盘/最高/最低 |
| volume | 成交量 |
| turnover | 成交额 |

**来源 URL**:
- 日K: `https://d.10jqka.com.cn/v2/line/hs_{code}/11/last.js`
- 周K/月K: `https://d.10jqka.com.cn/v2/line/hs_{code}/21/last.js`
**归一化函数**: `parseKlineData()`
**推送 API**: `POST /api/v1/kline`
**只推送一次** (首次加载时)

---

### 6. 资金流向 (moneyflow)

| 字段 | 说明 |
|------|------|
| code | 股票代码 |
| trade_date | 交易日期 |
| main_inflow / main_outflow / main_net | 主力流入/流出/净流入 |
| retail_inflow / retail_outflow / retail_net | 散户流入/流出/净流入 |

**来源 URL**: `https://d.10jqka.com.cn/v2/moneyflow/hs_{code}/last.js`
**归一化函数**: `parseMoneyFlowData()`
**推送 API**: `POST /api/v1/moneyflow`
**只推送一次** (首次加载时)

---

### 7. 其他盘口 (orderbook)

作为 fallback 处理非 fiverange 的盘口数据。

**来源 URL**: 包含 `/trade/` 或 `/detail` 的请求
**归一化函数**: `parseOrderBookData()`
**数据结构**: `data.bids` / `data.asks` 数组

---

## URL 路由表

| URL 包含 | 数据类型 | 归一化函数 |
|---------|---------|-----------|
| `/fiverange/` | 五档盘口 | `parseFiverangeData` |
| `/exchangedetail/` | 分时成交明细 | `parseExchangeDetailData` |
| `/line/` + `/01/` | 分时历史 | `parseTimeshareData` |
| `/line/` + `/11/` 或 `/21/` | K线 | `parseKlineData` |
| `/moneyflow/` | 资金流向 | `parseMoneyFlowData` |
| `/trade/` 或 `/detail` | 盘口(fallback) | `parseOrderBookData` |
| 其他 (如 `/realhead/`) | 实时行情 | `parseQuoteData` |

---

## 数据流

```
同花顺页面
  ↓ (JSONP script 请求)
interceptor.ts (MAIN world)
  ├── MutationObserver 监听 <script> 插入
  ├── hook JSONP 回调函数 (有 callback= 时)
  ├── document.createElement hook 捕获动态创建
  └── script.onload + fetch 获取 .js 响应体
  ↓ (window.postMessage)
injector.ts (隔离世界)
  ↓ (chrome.runtime.sendMessage)
background/index.ts
  ├── normalizeData() 归一化
  ├── 节流/批量处理
  └── pushQuote / pushTicks / pushOrderBook 等
  ↓ (HTTP POST)
astock 接收端
```