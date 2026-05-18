# CLAUDE.md
Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.
This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.


## 项目结构

本项目是 **astock - A股交易助手**，包含两个主要部分：

```
astock/            # Rust CLI 工具 (数据接收端、策略回测、技术分析)
astock-crx/        # Chrome 扩展 (数据采集，桥接同花顺 → astock 接收端)
```

## 常用命令

### Rust CLI

```bash
cargo build          # 开发构建
cargo build --release  # 生产构建
cargo run -- receiver  # 启动数据接收端 (127.0.0.1:17320)
astock search 茅台   # 搜索股票
astock quote 600519 # 查看实时行情
astock kline 600519 # 查看K线
```

### Chrome 扩展 (astock-crx)

```bash
cd astock-crx
pnpm dev       # 开发模式 (热重载)
pnpm build    # 生产构建 → dist/
```

## 架构概览

### 数据流

```
同花顺网页 → interceptor.ts (MAIN world) → injector.ts → background → receiver (Rust) → SQLite
```

1. **interceptor.ts** — 运行在页面 MAIN world，拦截 JSONP 请求 (d.10jqka.com.cn)
2. **background/index.ts** — Service Worker，归一化数据 + 节流 + 推送 HTTP
3. **receiver** — axum HTTP 服务，接收数据写入 SQLite

### 关键文件

| 文件 | 职责 |
|------|------|
| `astock-crx/src/content/interceptor.ts` | JSONP 拦截 + fetch 捕获 |
| `astock-crx/src/shared/data-normalizer.ts` | 同花顺 API → 归一化数据结构 |
| `astock-crx/src/background/index.ts` | 数据路由、节流、去重 |
| `src/receiver/handler.rs` | HTTP 接收端 API handler |
| `src/models/` | 数据模型 (Quote, TickTrade, OrderBook 等) |
| `src/db/` | SQLite 仓库层 |

### 实时数据类型

| 类型 | 采集 URL | 说明 |
|------|---------|------|
| quote | `/realhead/` | 实时行情 |
| orderbook | `/fiverange/` | 五档盘口 |
| tick | `/exchangedetail/` | 逐笔成交（含 tick_id） |
| kline | `/line/11/` | K线数据 |

**去重策略**：
- `tick` 用 `tick_id` 在数据库层 UNIQUE 约束
- `orderbook` 用快照对比，相同则跳过

## 开发注意事项

- `quote_snapshots` 表字段变更后需 `DROP TABLE IF EXISTS quote_snapshots` 重建
- `tick_trades` 表字段变更后需 `DROP TABLE IF EXISTS tick_trades` 重建
- 非交易时段 (`stock_status = "闭市"`/`"停牌"`) 数据会被过滤，不写入数据库
- 时间统一使用北京时间