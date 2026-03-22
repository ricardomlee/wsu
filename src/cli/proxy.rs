use crate::error::Result;
use crate::interop::wsl;

/// 自动检测并配置代理
pub fn auto() -> Result<()> {
    let host_ip = wsl::get_host_ip()?;

    // 常见代理端口
    let ports = [7890, 1080, 10808, 10809, 10810];

    println!("\x1b[36m检测 Windows 主机 IP: {}\x1b[0m", host_ip);
    println!("\x1b[36m扫描常见代理端口...\x1b[0m");

    let mut found_port = None;
    for port in ports {
        print!("  端口 {} ... ", port);
        if wsl::check_proxy_port(&host_ip, port) {
            println!("\x1b[32m✓ 可用\x1b[0m");
            found_port = Some(port);
            break;
        } else {
            println!("\x1b[31m✗\x1b[0m");
        }
    }

    if let Some(port) = found_port {
        let proxy_url = format!("http://{}:{}", host_ip, port);
        println!("\n\x1b[32m找到可用代理: {}\x1b[0m", proxy_url);
        println!("\n设置环境变量:");
        println!("  export HTTP_PROXY=\"{}\"", proxy_url);
        println!("  export HTTPS_PROXY=\"{}\"", proxy_url);
        println!("  export ALL_PROXY=\"{}\"", proxy_url);
        println!("\n\x1b[33m提示: 此配置仅在当前 shell 有效，使用 'wsu proxy export' 持久化\x1b[0m");

        // 设置环境变量
        std::env::set_var("HTTP_PROXY", &proxy_url);
        std::env::set_var("HTTPS_PROXY", &proxy_url);
        std::env::set_var("ALL_PROXY", &proxy_url);
    } else {
        println!("\n\x1b[31m未找到可用代理端口\x1b[0m");
        println!("请确保 Windows 代理软件已启动，或使用 'wsu proxy set <port>' 手动指定");
    }

    Ok(())
}

/// 手动设置代理
pub fn set(port: &str) -> Result<()> {
    let host_ip = wsl::get_host_ip()?;

    // 解析端口或完整地址
    let (addr, port_num) = if port.contains('@') {
        // 格式: user:pass@port
        let parts: Vec<&str> = port.rsplitn(2, '@').collect();
        let p = parts[0].parse::<u16>().map_err(|_| {
            crate::error::WsuError::Parse("无效的端口格式".to_string())
        })?;
        (format!("{}@{}", parts[1], host_ip), p)
    } else {
        // 格式: port
        let p = port.parse::<u16>().map_err(|_| {
            crate::error::WsuError::Parse("无效的端口格式".to_string())
        })?;
        (host_ip.clone(), p)
    };

    println!("\x1b[36m测试代理连接 {}:{}...\x1b[0m", host_ip, port_num);

    if !wsl::check_proxy_port(&host_ip, port_num) {
        println!("\x1b[31m警告: 无法连接到代理端口 {}\x1b[0m", port_num);
        println!("请检查代理软件是否已启动");
    }

    let proxy_url = format!("http://{}:{}", addr, port_num);
    println!("\x1b[32m设置代理: {}\x1b[0m", proxy_url);

    std::env::set_var("HTTP_PROXY", &proxy_url);
    std::env::set_var("HTTPS_PROXY", &proxy_url);
    std::env::set_var("ALL_PROXY", &proxy_url);

    println!("\n环境变量已设置:");
    println!("  HTTP_PROXY={}", proxy_url);
    println!("  HTTPS_PROXY={}", proxy_url);
    println!("  ALL_PROXY={}", proxy_url);

    Ok(())
}

/// 显示当前代理状态
pub fn status() -> Result<()> {
    let http_proxy = std::env::var("HTTP_PROXY").ok();
    let https_proxy = std::env::var("HTTPS_PROXY").ok();
    let all_proxy = std::env::var("ALL_PROXY").ok();

    println!("\x1b[1;36m当前代理状态:\x1b[0m");

    match &http_proxy {
        Some(p) => println!("  \x1b[32mHTTP_PROXY:\x1b[0m  {}", p),
        None => println!("  \x1b[31mHTTP_PROXY:\x1b[0m  未设置"),
    }

    match &https_proxy {
        Some(p) => println!("  \x1b[32mHTTPS_PROXY:\x1b[0m {}", p),
        None => println!("  \x1b[31mHTTPS_PROXY:\x1b[0m 未设置"),
    }

    match &all_proxy {
        Some(p) => println!("  \x1b[32mALL_PROXY:\x1b[0m    {}", p),
        None => println!("  \x1b[31mALL_PROXY:\x1b[0m    未设置"),
    }

    if http_proxy.is_none() && https_proxy.is_none() && all_proxy.is_none() {
        println!("\n\x1b[33m使用 'wsu proxy auto' 自动配置代理\x1b[0m");
    }

    Ok(())
}

/// 清除代理设置
pub fn clear() -> Result<()> {
    std::env::remove_var("HTTP_PROXY");
    std::env::remove_var("HTTPS_PROXY");
    std::env::remove_var("ALL_PROXY");
    std::env::remove_var("http_proxy");
    std::env::remove_var("https_proxy");
    std::env::remove_var("all_proxy");

    println!("\x1b[32m已清除所有代理环境变量\x1b[0m");
    Ok(())
}

/// 导出 shell 配置
pub fn export() -> Result<()> {
    let _host_ip = wsl::get_host_ip()?;  // 验证 WSL 环境

    // 生成自动配置函数
    println!("# wsu proxy auto-config");
    println!("wsu_proxy_auto() {{");
    println!("    local host_ip=$(grep nameserver /etc/resolv.conf | awk '{{print $2}}')");
    println!("    local ports=(7890 1080 10808 10809 10810)");
    println!("    for port in \"${{ports[@]}}\"; do");
    println!("        if nc -z $host_ip $port 2>/dev/null; then");
    println!("            export HTTP_PROXY=\"http://$host_ip:$port\"");
    println!("            export HTTPS_PROXY=\"http://$host_ip:$port\"");
    println!("            export ALL_PROXY=\"http://$host_ip:$port\"");
    println!("            echo \"Proxy set to $host_ip:$port\"");
    println!("            return");
    println!("        fi");
    println!("    done");
    println!("    echo \"No available proxy port found\"");
    println!("}}");
    println!("");
    println!("# 自动执行代理配置");
    println!("wsu_proxy_auto");

    Ok(())
}