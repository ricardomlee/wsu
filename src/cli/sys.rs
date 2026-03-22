use crate::error::Result;
use crate::interop::wsl;
use serde::Serialize;
use std::fs;
use unicode_width::UnicodeWidthStr;

/// 按显示宽度填充空格（中文字符占2个宽度）
fn pad_width(s: &str, width: usize) -> String {
    let display_width = UnicodeWidthStr::width(s);
    let padding = width.saturating_sub(display_width);
    format!("{}{}", s, " ".repeat(padding))
}

#[derive(Serialize)]
struct SystemInfo {
    wsl_version: String,
    kernel_version: String,
    host_ip: String,
    distro: String,
    mem_total_mb: u64,
    mem_used_mb: u64,
    mem_available_mb: u64,
    mem_usage_percent: f64,
}

fn read_meminfo() -> Result<(u64, u64, u64)> {
    let content = fs::read_to_string("/proc/meminfo")?;
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            mem_total = line
                .split(':')
                .nth(1)
                .and_then(|s| s.trim().split_whitespace().next())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
                / 1024; // KB to MB
        } else if line.starts_with("MemAvailable:") {
            mem_available = line
                .split(':')
                .nth(1)
                .and_then(|s| s.trim().split_whitespace().next())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
                / 1024;
        }
    }

    let mem_used = mem_total.saturating_sub(mem_available);
    Ok((mem_total, mem_used, mem_available))
}

fn read_os_release() -> Result<String> {
    let content = fs::read_to_string("/etc/os-release")?;
    for line in content.lines() {
        if line.starts_with("PRETTY_NAME=") {
            return Ok(line
                .split('=')
                .nth(1)
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_else(|| "Unknown".to_string()));
        }
    }
    Ok("Unknown".to_string())
}

fn read_kernel_version() -> Result<String> {
    let content = fs::read_to_string("/proc/version")?;
    // Format: Linux version 5.15.0-... (build info)
    let version = content
        .split_whitespace()
        .nth(2)
        .unwrap_or("unknown")
        .to_string();
    Ok(version)
}

pub fn run(json: bool) -> Result<()> {
    let host_ip = wsl::get_host_ip()?;
    let (mem_total, mem_used, mem_available) = read_meminfo()?;
    let mem_usage_percent = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let info = SystemInfo {
        wsl_version: "WSL2".to_string(),
        kernel_version: read_kernel_version()?,
        host_ip,
        distro: read_os_release()?,
        mem_total_mb: mem_total,
        mem_used_mb: mem_used,
        mem_available_mb: mem_available,
        mem_usage_percent,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        // 表格宽度 38 字符
        println!("\x1b[1;36m╔══════════════════════════════════════╗\x1b[0m");
        println!("\x1b[1;36m║\x1b[0m      \x1b[1;33mWSL2 系统信息\x1b[0m                  \x1b[1;36m║\x1b[0m");
        println!("\x1b[1;36m╠══════════════════════════════════════╣\x1b[0m");
        println!("\x1b[1;36m║\x1b[0m \x1b[32mWSL 版本:\x1b[0m  {} \x1b[1;36m║\x1b[0m", pad_width(&info.wsl_version, 26));
        println!("\x1b[1;36m║\x1b[0m \x1b[32m内核版本:\x1b[0m  {} \x1b[1;36m║\x1b[0m", pad_width(&info.kernel_version, 26));
        println!("\x1b[1;36m║\x1b[0m \x1b[32mWindows IP:\x1b[0m{} \x1b[1;36m║\x1b[0m", pad_width(&info.host_ip, 26));
        println!("\x1b[1;36m║\x1b[0m \x1b[32m发行版:\x1b[0m    {} \x1b[1;36m║\x1b[0m", pad_width(&info.distro, 26));
        println!("\x1b[1;36m╠══════════════════════════════════════╣\x1b[0m");
        println!("\x1b[1;36m║\x1b[0m \x1b[33m内存使用:\x1b[0m                            \x1b[1;36m║\x1b[0m");
        println!("\x1b[1;36m║\x1b[0m   总计: {:>6} MB                  \x1b[1;36m║\x1b[0m", info.mem_total_mb);
        println!("\x1b[1;36m║\x1b[0m   已用: {:>6} MB                  \x1b[1;36m║\x1b[0m", info.mem_used_mb);
        println!("\x1b[1;36m║\x1b[0m   可用: {:>6} MB                  \x1b[1;36m║\x1b[0m", info.mem_available_mb);
        println!("\x1b[1;36m║\x1b[0m   使用率: {:>5.1}%                   \x1b[1;36m║\x1b[0m", info.mem_usage_percent);
        println!("\x1b[1;36m╚══════════════════════════════════════╝\x1b[0m");
    }

    Ok(())
}