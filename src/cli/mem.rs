use crate::error::{Result, WsuError};
use std::fs;

/// 查看内存状态
pub fn status() -> Result<()> {
    let content = fs::read_to_string("/proc/meminfo")?;
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;
    let mut cached = 0u64;
    let mut swap_total = 0u64;
    let mut swap_free = 0u64;

    for line in content.lines() {
        let parse_value = |l: &str| -> u64 {
            l.split(':')
                .nth(1)
                .and_then(|s| s.trim().split_whitespace().next())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        };

        if line.starts_with("MemTotal:") {
            mem_total = parse_value(line) / 1024;
        } else if line.starts_with("MemAvailable:") {
            mem_available = parse_value(line) / 1024;
        } else if line.starts_with("Cached:") {
            cached = parse_value(line) / 1024;
        } else if line.starts_with("SwapTotal:") {
            swap_total = parse_value(line) / 1024;
        } else if line.starts_with("SwapFree:") {
            swap_free = parse_value(line) / 1024;
        }
    }

    let mem_used = mem_total.saturating_sub(mem_available);
    let swap_used = swap_total.saturating_sub(swap_free);
    let mem_usage = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };
    let swap_usage = if swap_total > 0 {
        (swap_used as f64 / swap_total as f64) * 100.0
    } else {
        0.0
    };

    // 绘制进度条
    fn progress_bar(usage: f64, width: usize) -> String {
        let filled = (usage / 100.0 * width as f64).round() as usize;
        let empty = width.saturating_sub(filled);
        let color = if usage > 80.0 {
            "\x1b[31m" // 红
        } else if usage > 60.0 {
            "\x1b[33m" // 黄
        } else {
            "\x1b[32m" // 绿
        };
        format!(
            "{}[{}{}]\x1b[0m",
            color,
            "█".repeat(filled),
            "░".repeat(empty)
        )
    }

    println!("\x1b[1;36m╔════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m           \x1b[1;33mWSL2 内存状态\x1b[0m                       \x1b[1;36m║\x1b[0m");
    println!("\x1b[1;36m╠════════════════════════════════════════════════╣\x1b[0m");

    println!("\x1b[1;36m║\x1b[0m \x1b[32m内存 (RAM):\x1b[0m");
    println!(
        "\x1b[1;36m║\x1b[0m   {} {:>5.1}%",
        progress_bar(mem_usage, 20),
        mem_usage
    );
    println!("\x1b[1;36m║\x1b[0m   总计: {:>6} MB    已用: {:>6} MB", mem_total, mem_used);
    println!("\x1b[1;36m║\x1b[0m   可用: {:>6} MB    缓存: {:>6} MB", mem_available, cached);
    println!("\x1b[1;36m║\x1b[0m");

    if swap_total > 0 {
        println!("\x1b[1;36m║\x1b[0m \x1b[32m交换分区:\x1b[0m");
        println!(
            "\x1b[1;36m║\x1b[0m   {} {:>5.1}%",
            progress_bar(swap_usage, 20),
            swap_usage
        );
        println!("\x1b[1;36m║\x1b[0m   总计: {:>6} MB    已用: {:>6} MB", swap_total, swap_used);
    } else {
        println!("\x1b[1;36m║\x1b[0m \x1b[33m交换分区: 未启用\x1b[0m");
    }

    println!("\x1b[1;36m║\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m \x1b[33m提示: 使用 'wsu mem reclaim' 释放缓存 (需要 root)\x1b[0m");
    println!("\x1b[1;36m╚════════════════════════════════════════════════╝\x1b[0m");

    Ok(())
}

/// 回收缓存
pub fn reclaim() -> Result<()> {
    // 检查是否是 root
    if unsafe { libc::getuid() } != 0 {
        return Err(WsuError::NeedRoot);
    }

    println!("\x1b[36m正在回收内存缓存...\x1b[0m");

    // sync 文件系统缓存
    #[cfg(target_os = "linux")]
    unsafe {
        libc::sync();
    }

    // 写入 drop_caches
    fs::write("/proc/sys/vm/drop_caches", "1")?;

    println!("\x1b[32m内存缓存已回收\x1b[0m");

    // 显示回收后的状态
    status()?;

    Ok(())
}