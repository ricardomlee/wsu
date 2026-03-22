use crate::error::{Result, WsuError};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// .wslconfig 配置项
#[derive(Debug, Clone)]
pub struct WslConfig {
    pub memory: Option<String>,
    pub processors: Option<u32>,
    pub swap: Option<String>,
    pub localhost_forwarding: Option<bool>,
    pub networking_mode: Option<String>,
    pub firewall: Option<bool>,
    pub auto_proxy: Option<bool>,
}

impl Default for WslConfig {
    fn default() -> Self {
        Self {
            memory: None,
            processors: None,
            swap: None,
            localhost_forwarding: Some(true),
            networking_mode: None,
            firewall: None,
            auto_proxy: None,
        }
    }
}

impl WslConfig {
    /// 从文件读取配置
    pub fn load() -> Result<Self> {
        let path = get_wslconfig_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        Self::parse(&content)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        let path = get_wslconfig_path()?;

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = self.to_ini();
        fs::write(&path, content)?;

        println!("\x1b[32m配置已保存到: {}\x1b[0m", path.display());
        println!("\x1b[33m提示: 需要重启 WSL 生效 (wsl --shutdown)\x1b[0m");

        Ok(())
    }

    /// 解析 INI 格式
    fn parse(content: &str) -> Result<Self> {
        let mut config = Self::default();
        let mut in_wsl2_section = false;

        for line in content.lines() {
            let line = line.trim();

            if line == "[wsl2]" {
                in_wsl2_section = true;
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_wsl2_section = false;
                continue;
            }

            if !in_wsl2_section || line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "memory" => config.memory = Some(value.to_string()),
                "processors" => config.processors = value.parse().ok(),
                "swap" => config.swap = Some(value.to_string()),
                "localhostForwarding" => {
                    config.localhost_forwarding = parse_bool(value)
                }
                "networkingMode" => config.networking_mode = Some(value.to_string()),
                "firewall" => config.firewall = parse_bool(value),
                "autoProxy" => config.auto_proxy = parse_bool(value),
                _ => {}
            }
        }

        Ok(config)
    }

    /// 转换为 INI 格式
    fn to_ini(&self) -> String {
        let mut lines = vec!["[wsl2]".to_string()];

        if let Some(ref v) = self.memory {
            lines.push(format!("memory={}", v));
        }
        if let Some(v) = self.processors {
            lines.push(format!("processors={}", v));
        }
        if let Some(ref v) = self.swap {
            lines.push(format!("swap={}", v));
        }
        if let Some(v) = self.localhost_forwarding {
            lines.push(format!("localhostForwarding={}", v));
        }
        if let Some(ref v) = self.networking_mode {
            lines.push(format!("networkingMode={}", v));
        }
        if let Some(v) = self.firewall {
            lines.push(format!("firewall={}", v));
        }
        if let Some(v) = self.auto_proxy {
            lines.push(format!("autoProxy={}", v));
        }

        lines.join("\n") + "\n"
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// 获取 .wslconfig 文件路径
fn get_wslconfig_path() -> Result<PathBuf> {
    // 从 Windows 环境变量获取用户目录
    let output = std::process::Command::new("cmd.exe")
        .args(["/c", "echo %USERPROFILE%"])
        .output()?;

    let win_path = String::from_utf8_lossy(&output.stdout)
        .trim()
        .replace('\\', "/")
        .trim_end_matches('/').to_string();

    // 转换为 WSL 路径
    let wsl_path = if win_path.starts_with("C:") || win_path.starts_with("c:") {
        format!("/mnt/c{}", &win_path[2..])
    } else if let Some(drive) = win_path.chars().next() {
        format!("/mnt/{}{}", drive.to_lowercase(), &win_path[1..])
    } else {
        return Err(WsuError::Config("无法确定 Windows 用户目录".to_string()));
    };

    Ok(PathBuf::from(wsl_path).join(".wslconfig"))
}

/// 显示当前配置
pub fn show() -> Result<()> {
    let config = WslConfig::load()?;
    let path = get_wslconfig_path()?;

    println!("\x1b[1;36m╔════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m          \x1b[1;33mWSL 配置 (.wslconfig)\x1b[0m           \x1b[1;36m║\x1b[0m");
    println!("\x1b[1;36m╠════════════════════════════════════════╣\x1b[0m");

    if !path.exists() {
        println!("\x1b[1;36m║\x1b[0m \x1b[33m配置文件不存在，使用默认配置\x1b[0m        \x1b[1;36m║\x1b[0m");
    } else {
        println!("\x1b[1;36m║\x1b[0m 文件: {:<30} \x1b[1;36m║\x1b[0m", path.display());
    }

    println!("\x1b[1;36m╠════════════════════════════════════════╣\x1b[0m");

    print_config_item("内存限制", config.memory.as_deref(), "系统自动");
    print_config_item("处理器数", config.processors.map(|v| v.to_string()).as_deref(), "系统自动");
    print_config_item("交换空间", config.swap.as_deref(), "系统自动");
    print_config_item("本地转发", config.localhost_forwarding.map(|v| v.to_string()).as_deref(), "true");
    print_config_item("网络模式", config.networking_mode.as_deref(), "NAT");
    print_config_item("防火墙", config.firewall.map(|v| v.to_string()).as_deref(), "true");
    print_config_item("自动代理", config.auto_proxy.map(|v| v.to_string()).as_deref(), "false");

    println!("\x1b[1;36m╚════════════════════════════════════════╝\x1b[0m");

    println!("\n\x1b[36m用法:\x1b[0m");
    println!("  wsu config set memory 4GB");
    println!("  wsu config set processors 2");
    println!("  wsu config edit  # 用编辑器打开");

    Ok(())
}

fn print_config_item(label: &str, value: Option<&str>, default: &str) {
    let display_value = value.unwrap_or(default);
    let label_formatted = format!("{}:", label);
    println!("\x1b[1;36m║\x1b[0m {:12} {:<28} \x1b[1;36m║\x1b[0m", label_formatted, display_value);
}

/// 设置配置项
pub fn set(key: &str, value: &str) -> Result<()> {
    let mut config = WslConfig::load()?;

    match key {
        "memory" => {
            // 验证格式
            if !value.ends_with("GB") && !value.ends_with("MB") && !value.ends_with("%") {
                println!("\x1b[33m提示: 内存格式应为 4GB, 512MB 或 50%\x1b[0m");
            }
            config.memory = Some(value.to_string());
        }
        "processors" => {
            let n: u32 = value.parse().map_err(|_| {
                WsuError::Parse("处理器数必须是数字".to_string())
            })?;
            config.processors = Some(n);
        }
        "swap" => {
            config.swap = Some(value.to_string());
        }
        "localhostForwarding" | "localhost" => {
            config.localhost_forwarding = Some(parse_bool(value).unwrap_or(true));
        }
        "networkingMode" | "networking" => {
            config.networking_mode = Some(value.to_string());
        }
        "firewall" => {
            config.firewall = Some(parse_bool(value).unwrap_or(true));
        }
        "autoProxy" | "proxy" => {
            config.auto_proxy = Some(parse_bool(value).unwrap_or(false));
        }
        _ => {
            return Err(WsuError::Config(format!(
                "未知配置项: {}\n可用: memory, processors, swap, localhostForwarding, networkingMode, firewall, autoProxy",
                key
            )));
        }
    }

    config.save()?;
    show()?;

    Ok(())
}

/// 用编辑器打开配置文件
pub fn edit() -> Result<()> {
    let path = get_wslconfig_path()?;

    // 如果文件不存在，创建默认配置
    if !path.exists() {
        let config = WslConfig::default();
        let content = config.to_ini();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
    }

    // 使用 Windows 默认编辑器打开
    let path_str = path.to_string_lossy();
    std::process::Command::new("cmd.exe")
        .args(["/c", "start", "", &path_str])
        .spawn()?;

    println!("\x1b[32m已打开配置文件\x1b[0m");

    Ok(())
}