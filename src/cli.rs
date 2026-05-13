/// CLI命令定义 (使用clap derive)
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "astock", about = "A股交易助手", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 搜索股票 (支持拼音/代码/名称)
    Search {
        /// 搜索关键词
        query: String,
        /// 返回结果数量
        #[arg(short, long, default_value_t = 10)]
        limit: u32,
    },

    /// 查看实时行情
    Quote {
        /// 股票代码，多个用逗号分隔 (如: 600519,000001)
        codes: String,
    },

    /// 查看K线数据
    Kline {
        /// 股票代码
        code: String,
        /// K线类型: day, week, month, m5, m15, m30, m60
        #[arg(short, long, default_value = "day")]
        ktype: String,
        /// 返回数据条数
        #[arg(short, long, default_value_t = 60)]
        count: u32,
        /// 前复权
        #[arg(short, long, default_value_t = true)]
        qfq: bool,
    },

    /// 查看基本面信息
    Info {
        /// 股票代码
        code: String,
    },

    /// 查看技术分析指标
    Indicator {
        /// 指标类型: all, ma, macd, rsi, kdj, boll
        indicator_type: String,
        /// 股票代码
        code: String,
        /// K线条数 (需要足够数据来计算指标)
        #[arg(short, long, default_value_t = 120)]
        count: u32,
    },

    /// 策略回测
    Backtest {
        #[command(subcommand)]
        command: BacktestCommands,
    },

    /// 交易记录管理
    Trade {
        #[command(subcommand)]
        command: TradeCommands,
    },

    /// 持仓管理
    Position {
        #[command(subcommand)]
        command: PositionCommands,
    },

    /// 自选股管理
    Watch {
        #[command(subcommand)]
        command: WatchCommands,
    },

    /// 预警管理
    Alert {
        #[command(subcommand)]
        command: AlertCommands,
    },

    /// 实时监控
    Monitor {
        #[command(subcommand)]
        command: MonitorCommands,
    },

    /// 启动数据接收端 (Chrome插件推送)
    Receiver {
        /// 监听端口
        #[arg(short, long, default_value_t = 17320)]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum BacktestCommands {
    /// 运行回测
    Run {
        /// 股票代码
        code: String,
        /// 策略名称: dual_ma, macd_cross, rsi_reversal, kdj_cross, boll_break
        #[arg(short, long)]
        strategy: String,
        /// 初始资金
        #[arg(long, default_value_t = 100000)]
        capital: i64,
        /// K线条数
        #[arg(short, long, default_value_t = 250)]
        count: u32,
    },

    /// 列出所有可用策略
    ListStrategies,
}

/// 交易记录命令
#[derive(Subcommand)]
pub enum TradeCommands {
    /// 添加交易记录
    Add {
        /// 股票代码
        code: String,
        /// 买卖方向: buy 或 sell
        #[arg(short, long)]
        action: String,
        /// 价格
        #[arg(short, long)]
        price: String,
        /// 股数 (必须是100的整数倍)
        #[arg(short, long)]
        shares: i64,
        /// 交易日期 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: String,
        /// 备注
        #[arg(short, long, default_value = "")]
        note: String,
    },

    /// 查看交易记录
    List {
        /// 按股票代码筛选
        #[arg(short, long)]
        code: Option<String>,
        /// 按买卖方向筛选
        #[arg(short, long)]
        action: Option<String>,
    },

    /// 删除交易记录
    Delete {
        /// 交易记录ID
        id: i64,
    },

    /// 交易汇总
    Summary,
}

/// 持仓管理命令
#[derive(Subcommand)]
pub enum PositionCommands {
    /// 查看当前持仓 (含实时盈亏)
    List,

    /// 手动添加持仓
    Add {
        /// 股票代码
        code: String,
        /// 股数
        #[arg(short, long)]
        shares: i64,
        /// 成本均价
        #[arg(short = 'c', long)]
        avg_cost: String,
    },

    /// 删除持仓
    Remove {
        /// 股票代码
        code: String,
    },
}

/// 自选股管理命令
#[derive(Subcommand)]
pub enum WatchCommands {
    /// 查看自选股列表
    List,

    /// 添加自选股
    Add {
        /// 股票代码
        code: String,
    },

    /// 删除自选股
    Remove {
        /// 股票代码
        code: String,
    },

    /// 查看自选股实时行情
    Quote,
}

/// 预警管理命令
#[derive(Subcommand)]
pub enum AlertCommands {
    /// 添加预警规则
    Add {
        /// 股票代码
        code: String,
        /// 预警类型: price_above, price_below, ma_cross, macd_cross, kdj_cross
        #[arg(short, long)]
        signal_type: String,
        /// 价格阈值 (price_above/price_below 时使用)
        #[arg(short, long)]
        price: Option<f64>,
    },

    /// 查看所有预警规则
    List,

    /// 删除预警规则
    Remove {
        /// 规则ID
        id: i64,
    },

    /// 查看预警历史
    History,
}

/// 监控命令
#[derive(Subcommand)]
pub enum MonitorCommands {
    /// 启动实时监控
    Start {
        /// 检查间隔(秒)
        #[arg(short, long, default_value_t = 30)]
        interval: u64,
    },
}
