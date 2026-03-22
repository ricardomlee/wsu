use crate::error::{Result, WsuError};
use std::fs;

/// 检查是否在 WSL 环境中运行
pub fn is_wsl() -> bool {
    fs::read_to_string("/proc/version")
        .map(|content| content.contains("Microsoft") || content.contains("WSL"))
        .unwrap_or(false)
}

/// 获取 Windows 主机 IP (通过 /etc/resolv.conf 中的 nameserver)
pub fn get_host_ip() -> Result<String> {
    if !is_wsl() {
        return Err(WsuError::NotWsl);
    }

    let content = fs::read_to_string("/etc/resolv.conf")?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("nameserver") {
            return line
                .split_whitespace()
                .nth(1)
                .map(|s| s.to_string())
                .ok_or(WsuError::NoHostIp);
        }
    }

    Err(WsuError::NoHostIp)
}

/// 检查代理端口是否可连通
pub fn check_proxy_port(host: &str, port: u16) -> bool {
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = format!("{}:{}", host, port);
    TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(2)).is_ok()
}