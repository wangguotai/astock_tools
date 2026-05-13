# astock - A股交易助手

基于 Rust 构建的 A 股交易助手 CLI 工具，帮助提高选股和操作胜率。

## 功能概览

| 阶段 | 功能 | 状态 |
|------|------|------|
| Phase 1 | 行情数据获取 + 展示 | ✅ 已完成 |
| Phase 2 | 技术分析指标 (MA/MACD/RSI/KDJ/BOLL) | ✅ 已完成 |
| Phase 3 | 策略回测引擎 | ✅ 已完成 |
| Phase 4 | 交易记录 + 持仓管理 | ✅ 已完成 |
| Phase 5 | 实时监控 + 预警 | ✅ 已完成 |
| Phase 6 | Chrome插件 + 同花顺数据桥接 | ✅ 已完成 |

## 安装

```bash
# 需要 Rust 工具链
cargo build --release

# 二进制文件位于 target/release/astock
# 可选: 复制到 PATH 中
cp target/release/astock /usr/local/bin/
```

## 使用方法

### 搜索股票

支持拼音缩写、股票代码、中文名称搜索：

```bash
# 拼音搜索
astock search mt

# 中文搜索
astock search 茅台

# 代码搜索
astock search 600519

# 限制返回数量
astock search bank --limit 5
```

### 实时行情

查看股票实时报价，支持多只股票同时查询：

```bash
# 单只股票
astock quote 600519

# 多只股票（逗号分隔）
astock quote 600519,000001,300750

# 也支持带市场前缀的格式
astock quote sh600519,sz000001
```

输出示例：
```
┌───────────┬──────────┬─────────┬───────┬────────┬─────────┬─────────┬─────────┬──────────┬─────────┐
│ 代码      ┆ 名称     ┆ 现价    ┆ 涨跌  ┆ 涨跌幅 ┆ 今开    ┆ 最高    ┆ 最低    ┆ 成交量   ┆ 成交额  │
╞═══════════╪══════════╪═════════╪═══════╪════════╪═════════╪═════════╪═════════╪══════════╪═════════╡
│ 600519.SH ┆ 贵州茅台 ┆ 1372.99 ┆ +1.94 ┆ +0.14% ┆ 1371.66 ┆ 1382.77 ┆ 1370.00 ┆ 3.34万手 ┆ 45.82亿 │
└───────────┴──────────┴─────────┴───────┴────────┴─────────┴─────────┴──────────┴──────────┴─────────┘
```

颜色遵循 A 股惯例：**红涨绿跌**。

### K线数据

获取历史K线数据，支持日线、周线、月线和分钟线：

```bash
# 日K线（默认60条）
astock kline 600519

# 指定条数
astock kline 600519 --count 120

# 周K线
astock kline 600519 --ktype week

# 月K线
astock kline 600519 --ktype month

# 5分钟线
astock kline 600519 --ktype m5 --count 30

# 15/30/60分钟线
astock kline 600519 --ktype m15
astock kline 600519 --ktype m30
astock kline 600519 --ktype m60
```

K线类型参数：`day`(默认), `week`, `month`, `m5`, `m15`, `m30`, `m60`

默认使用前复权数据。

### 基本面信息

查看股票财务数据（来自东方财富）：

```bash
astock info 600519
```

输出示例：
```
600519.SH 基本面信息:

--- 2025-09-30 00:00:00 (三季报) ---
  每股收益: 51.5300
  每股净资产: 205.28
  净资产收益率: 24.64%
  毛利率: 91.29%
  净利率: 52.08%
  营收同比增长: 6.32%
  净利润同比增长: 6.25%
  资产负债率: 12.81%
```

### 技术分析指标

查看技术分析指标，支持 MA、MACD、RSI、KDJ、布林带：

```bash
# 综合指标报告 (推荐)
astock indicator all 600519

# 单独查看各指标
astock indicator ma 600519
astock indicator macd 600519
astock indicator rsi 600519
astock indicator kdj 600519
astock indicator boll 600519

# 指定K线条数 (默认120，数据越多指标越准确)
astock indicator all 600519 --count 250
```

综合报告示例：
```
600519.SH 技术指标报告 (2026-05-08)
────────────────────────────────────────────────────────────
MA:   MA5: 1380.99  MA10: 1400.01  MA20: 1421.51  MA60: 1440.50
      → 空头排列 (MA5 < MA10 < MA20)
MACD: DIF=-16.07  DEA=-9.56  柱=-6.50
      → DIF在DEA下方 (死叉/空头)
RSI:  31.40
      → 中性区域
KDJ:  K=12.64  D=14.90  J=8.10
      → K在D下方 (死叉)
BOLL: 上轨=1485.96  中轨=1421.51  下轨=1357.07
      → 价格在中轨下方
────────────────────────────────────────────────────────────
```

#### 指标说明

| 指标 | 含义 | 关键信号 |
|------|------|----------|
| MA | 移动平均线 (5/10/20/60日) | 多头/空头排列、均线交叉 |
| MACD | 指数平滑异同移动平均线 | 金叉(买入)/死叉(卖出) |
| RSI | 相对强弱指标 | >70超买, <30超卖 |
| KDJ | 随机指标 | K/D金叉死叉, J>100超买, J<0超卖 |
| BOLL | 布林带 | 突破上轨超买, 跌破下轨超卖 |

### 策略回测

基于历史K线数据回测交易策略，评估策略表现：

```bash
# 运行双均线策略回测
astock backtest run 600519 --strategy dual_ma

# 指定初始资金和K线条数
astock backtest run 000001 --strategy macd_cross --capital 50000 --count 250

# 列出所有可用策略
astock backtest list-strategies
```

可用策略：

| 策略名 | 说明 | 参数 |
|--------|------|------|
| `dual_ma` | 双均线交叉 (MA5/MA20) | 短期均线上穿长期均线买入 |
| `macd_cross` | MACD金叉死叉 | 柱状图由负转正买入 |
| `rsi_reversal` | RSI超买超卖 | RSI<30买入，RSI>70卖出 |
| `kdj_cross` | KDJ金叉死叉 | K线上穿D线买入 |
| `boll_break` | 布林带突破 | 价格突破上轨买入 |

回测结果示例：
```
=== 回测结果 (dual_ma) 600519.SH ===
交易次数: 12
胜率: 16.67%
总收益: -10.30%
最大回撤: 22.45%
盈亏比: 0.57
```

回测模型说明：
- **T+1规则**：当日买入次日才可卖出
- **交易成本**：佣金万三(最低5元)，印花税0.05%(仅卖出)，过户费0.001%
- **滑点**：默认0.1%
- **执行价格**：信号触发后以下一根K线开盘价执行

### 交易记录

管理买卖交易记录，自动计算费用：

```bash
# 添加买入记录
astock trade add 600519 --action buy --price 1350.00 --shares 100 --date 2025-12-01

# 添加卖出记录
astock trade add 600519 --action sell --price 1400.00 --shares 100 --date 2025-12-10

# 查看交易记录
astock trade list
astock trade list --code 600519       # 按股票筛选
astock trade list --action buy        # 按买卖方向筛选

# 删除交易记录
astock trade delete 1

# 交易汇总
astock trade summary
```

费用自动计算：
- 佣金：0.03%（最低5元）
- 印花税：0.05%（仅卖出）
- 过户费：0.001%

### 持仓管理

查看和管理当前持仓，实时计算盈亏：

```bash
# 查看当前持仓 (含实时盈亏)
astock position list

# 手动添加持仓
astock position add 600519 --shares 100 --avg-cost 1350.00

# 删除持仓
astock position remove 600519
```

持仓列表示例：
```
┌───────────┬──────────┬──────┬──────────┬─────────┬──────────┬─────────┐
│ 代码      ┆ 名称     ┆ 股数 ┆ 成本价   ┆ 现价    ┆ 盈亏     ┆ 盈亏%   │
╞═══════════╪══════════╪══════╪══════════╪═════════╪══════════╪═════════╡
│ 600519.SH ┆ 贵州茅台 ┆ 100  ┆ 1350.00  ┆ 1372.99 ┆ +2299.00 ┆ +1.70%  │
└───────────┴──────────┴──────┴──────────┴─────────┴──────────┴─────────┘
```

### 自选股管理

管理自选股列表，快速查看关注股票的行情：

```bash
# 添加自选股
astock watch add 600519
astock watch add 000001

# 查看自选股列表
astock watch list

# 查看自选股实时行情
astock watch quote

# 删除自选股
astock watch remove 600519
```

### 预警管理

设置价格和技术指标预警，触发时发送桌面通知：

```bash
# 添加价格预警 (突破指定价格)
astock alert add 600519 --signal-type price_above --price 1400

# 添加价格预警 (跌破指定价格)
astock alert add 000001 --signal-type price_below --price 10.5

# 添加技术指标预警
astock alert add 600519 --signal-type ma_cross
astock alert add 600519 --signal-type macd_cross
astock alert add 600519 --signal-type kdj_cross

# 查看所有预警规则
astock alert list

# 删除预警规则
astock alert remove 1

# 查看预警触发历史
astock alert history
```

预警类型说明：

| 预警类型 | 说明 | 需要参数 |
|----------|------|----------|
| `price_above` | 价格突破阈值 | `--price` |
| `price_below` | 价格跌破阈值 | `--price` |
| `ma_cross` | MA5上穿MA20 (金叉) | 无 |
| `macd_cross` | DIF上穿DEA (金叉) | 无 |
| `kdj_cross` | K上穿D (金叉) | 无 |

### 实时监控

在交易时段自动轮询行情和预警规则，涨跌幅超过3%时发送桌面通知：

```bash
# 启动监控 (默认每30秒检查)
astock monitor start

# 指定检查间隔 (秒)
astock monitor start --interval 60
```

监控行为：
- 仅在交易时段运行：工作日 09:30-11:30、13:00-15:00
- 自动检查所有启用的预警规则
- 自动检查自选股行情
- 涨跌幅超过3%时发送 macOS 桌面通知
- 按 `Ctrl+C` 优雅退出

### 数据接收端 (Chrome插件桥接)

启动本地HTTP服务接收Chrome插件从同花顺网页端截取的实时数据：

```bash
# 启动接收端 (默认端口17320)
astock receiver

# 指定端口
astock receiver --port 8080
```

接收端API：

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/v1/quote` | POST | 接收实时行情 |
| `/api/v1/kline` | POST | 接收K线数据 |
| `/api/v1/tick` | POST | 接收分时成交 |
| `/api/v1/orderbook` | POST | 接收盘口5档 |
| `/api/v1/moneyflow` | POST | 接收资金流向 |
| `/api/v1/status` | GET | 健康检查 |
| `/api/v1/watchlist` | GET | 返回自选股列表 |

### Chrome插件 (astock-crx)

从同花顺网页端截取实时数据并推送到astock接收端：

**安装**：
1. Chrome打开 `chrome://extensions/`
2. 开启"开发者模式"
3. 点击"加载已解压的扩展程序"，选择 `astock-crx/` 目录

**使用**：
1. 启动astock接收端：`astock receiver`
2. 打开同花顺股票页面：`https://stockpage.10jqka.com.cn/002202/`
3. 插件自动拦截页面数据API，归一化后推送到本地接收端
4. 点击插件图标查看连接状态和推送记录

**数据节流**：
- 行情：价格变化或10秒间隔推送
- 分时成交：5秒批量推送
- 盘口：5秒一次
- K线/资金流向：页面加载时推送一次

## 股票代码格式

程序自动识别市场，支持多种输入格式：

| 输入格式 | 示例 | 解析结果 |
|----------|------|----------|
| 纯6位数字 | `600519` | 自动判断市场 |
| 小写前缀 | `sh600519` | 上海 |
| 大写前缀 | `SZ000001` | 深圳 |
| Wind格式 | `600519.SH` | 上海 |

市场规则：
- `6xxxxx` → 上海证券交易所 (sh)
- `0xxxxx` → 深圳主板 (sz)
- `3xxxxx` → 深圳创业板 (sz)
- `8xxxxx`, `4xxxxx` → 北京交易所 (bj)

## 数据源

| 数据类型 | 数据源 | 说明 |
|----------|--------|------|
| 实时行情 | 腾讯财经 `qt.gtimg.cn` | GBK编码，免费无需认证，约15分钟延迟 |
| K线数据 | 腾讯财经 `ifzq.gtimg.cn` | UTF-8 JSON，支持前复权 |
| 财务数据 | 东方财富 `datacenter.eastmoney.com` | UTF-8 JSON，需Referer头 |
| 股票搜索 | 东方财富 `searchapi.eastmoney.com` | 支持拼音/代码/名称 |
| 实时数据 | 同花顺 (Chrome插件) | 无延迟，通过XHR拦截获取 |

## 项目结构

```
src/
  main.rs              -- 入口，tokio runtime，CLI分发
  cli.rs               -- clap命令定义
  error.rs             -- 统一错误类型
  models/
    stock.rs           -- StockCode (代码+市场前缀映射)
    bar.rs             -- Bar (OHLCV K线), BarSeries
    quote.rs           -- Quote (实时行情快照)
    tick_trade.rs      -- TickTrade (分时成交)
    order_book.rs      -- OrderBookSnapshot (盘口5档)
    money_flow.rs      -- MoneyFlow (资金流向)
  data/
    mod.rs             -- DataClient (统一API客户端)
    encoding.rs        -- GBK→UTF-8解码
    tencent.rs         -- 腾讯实时行情API
    tencent_kline.rs   -- 腾讯K线API
    eastmoney.rs       -- 东方财富搜索+财务API
  analysis/
    mod.rs             -- 模块入口
    indicators.rs      -- MA, EMA, 布林带 (ta crate)
    macd.rs            -- MACD (ta crate)
    rsi.rs             -- RSI (ta crate)
    kdj.rs             -- KDJ (自定义实现)
    report.rs          -- 综合指标报告
  backtest/
    mod.rs             -- 模块入口
    engine.rs          -- 回测引擎 (walk-forward循环)
    costs.rs           -- A股交易成本模型
    config.rs          -- 回测配置 (T+1, 滑点)
    strategies/
      mod.rs           -- 策略注册
      dual_ma.rs       -- 双均线交叉
      macd_cross.rs    -- MACD金叉死叉
      rsi_reversal.rs  -- RSI超买超卖
      kdj_cross.rs     -- KDJ金叉死叉
      boll_break.rs    -- 布林带突破
  db/
    mod.rs             -- DB初始化+迁移
    schema.rs          -- SQL建表语句
    bar_repo.rs        -- 历史K线存储/查询
    trade_repo.rs      -- 交易记录CRUD
    position_repo.rs   -- 持仓管理
    watch_repo.rs      -- 自选股CRUD
    alert_repo.rs      -- 预警规则CRUD+历史
    tick_repo.rs       -- 分时成交CRUD
    orderbook_repo.rs  -- 盘口快照CRUD
    moneyflow_repo.rs  -- 资金流向CRUD
    quote_snapshot_repo.rs -- 行情快照CRUD
  receiver/
    mod.rs             -- axum路由+CORS+服务启动
    handler.rs         -- 5个POST+2个GET端点处理
    models.rs          -- 请求/响应JSON结构体
  monitor/
    mod.rs             -- 模块入口
    watcher.rs         -- 监控循环 (tokio interval)
    notifier.rs        -- macOS桌面通知
  display/
    table.rs           -- 表格展示 (红涨绿跌)

astock-crx/                -- Chrome插件
  manifest.json            -- Manifest V3配置
  background.js            -- Service Worker数据流编排
  content_scripts/
    inject.js              -- 注入桥接+页面代码识别
    interceptor.js         -- XHR/fetch拦截 (MAIN world)
  popup/                   -- 插件弹窗UI
  options/                 -- 设置页
  lib/
    stock-code.js          -- 股票代码映射
    data-normalizer.js     -- API数据归一化
    api-client.js          -- HTTP推送客户端
```

## 技术栈

| 领域 | 依赖 | 用途 |
|------|------|------|
| CLI | clap (derive) | 命令行参数解析 |
| HTTP | reqwest + tokio | 异步HTTP请求 |
| 编码 | encoding_rs | GBK→UTF-8解码 |
| 精度 | rust_decimal | 金融计算避免浮点误差 |
| 时间 | chrono | 日期时间处理 |
| 序列化 | serde + serde_json | JSON解析 |
| 表格 | comfy-table | 终端表格展示 |
| 错误 | anyhow + thiserror | 错误处理 |
| 技术分析 | ta | MA/MACD/RSI/BOLL指标计算 |
| 数据库 | rusqlite (bundled) | SQLite本地存储 |
| HTTP服务 | axum + tower-http | 数据接收端 (Chrome插件桥接) |
| 通知 | notify-rust | macOS桌面通知 |
| 进程 | ctrlc | Ctrl+C优雅退出 |

## 注意事项

1. **GBK编码**：腾讯实时行情API返回GBK编码，程序自动处理，无需手动转换
2. **K线字段顺序**：腾讯K线返回 `[date, open, close, high, low, volume]`，close在high前面
3. **精度**：所有价格使用 `Decimal` 类型，避免浮点精度问题
4. **A股惯例**：显示颜色为红涨绿跌（与国际市场相反）
5. **数据延迟**：免费API数据可能有15分钟延迟，不适用于实时交易决策
6. **数据库位置**：`~/.astock/astock.db`，首次运行自动创建
7. **A股成本模型**：佣金万三(最低5元)、印花税0.05%(仅卖出)、过户费0.001%
8. **同花顺数据**：Chrome插件通过XHR拦截获取，无延迟；接收端自动触发预警检查
9. **数据接收端**：仅监听127.0.0.1，使用WAL模式减少SQLite写锁冲突
