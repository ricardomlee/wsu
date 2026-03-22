use thiserror::Error;

#[derive(Error, Debug)]
pub enum WsuError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("交互错误: {0}")]
    Dialog(#[from] dialoguer::Error),

    #[error("解析错误: {0}")]
    Parse(String),

    #[error("不在 WSL 环境中运行")]
    NotWsl,

    #[error("需要 root 权限")]
    NeedRoot,

    #[error("代理连接失败: {0}")]
    ProxyFailed(String),

    #[error("配置文件错误: {0}")]
    Config(String),

    #[error("未找到 Windows 主机 IP")]
    NoHostIp,
}

pub type Result<T> = std::result::Result<T, WsuError>;