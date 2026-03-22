mod cli;
mod error;
mod interop;

use clap::{Parser, Subcommand};
use cli::{config, mem, proxy, sys};

/// WSL2 智能工具箱
#[derive(Parser)]
#[command(name = "wsu", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 智能代理配置 - 自动检测 Windows IP 并设置代理
    Proxy {
        #[command(subcommand)]
        action: ProxyAction,
    },

    /// 内存管理 - 查看状态、回收缓存
    Mem {
        #[command(subcommand)]
        action: MemAction,
    },

    /// WSL 配置管理 - 读写 .wslconfig
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// 开发环境快速初始化
    Init {
        /// 使用预设模板
        #[arg(short, long)]
        template: Option<String>,
    },

    /// 系统信息
    Sys {
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ProxyAction {
    /// 自动检测并配置代理
    Auto,

    /// 手动设置代理端口
    Set {
        /// 代理端口或完整地址 (如 7890 或 user:pass@7890)
        port: String,
    },

    /// 显示当前代理状态
    Status,

    /// 清除代理设置
    Clear,

    /// 导出 shell 配置
    Export,
}

#[derive(Subcommand)]
enum MemAction {
    /// 查看内存状态
    Status,

    /// 回收缓存 (需要 root)
    Reclaim,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// 显示当前配置
    Show,

    /// 设置配置项 (如: memory 4GB, processors 2)
    Set {
        /// 配置项名称
        key: String,
        /// 配置值
        value: String,
    },

    /// 用编辑器打开配置文件
    Edit,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Sys { json } => sys::run(json),
        Commands::Proxy { action } => match action {
            ProxyAction::Auto => proxy::auto(),
            ProxyAction::Set { port } => proxy::set(&port),
            ProxyAction::Status => proxy::status(),
            ProxyAction::Clear => proxy::clear(),
            ProxyAction::Export => proxy::export(),
        },
        Commands::Mem { action } => match action {
            MemAction::Status => mem::status(),
            MemAction::Reclaim => mem::reclaim(),
        },
        Commands::Config { action } => match action {
            ConfigAction::Show => config::show(),
            ConfigAction::Set { key, value } => config::set(&key, &value),
            ConfigAction::Edit => config::edit(),
        },
        Commands::Init { template } => cli::init::run(template.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("\x1b[31m错误:\x1b[0m {}", e);
        std::process::exit(1);
    }
}