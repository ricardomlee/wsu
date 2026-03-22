use crate::error::{Result, WsuError};
use std::fs;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

// ============================================================================
// .wslconfig - 全局配置 (Windows 用户目录)
// ============================================================================

/// .wslconfig [wsl2] 部分配置
#[derive(Debug, Clone, Default)]
pub struct Wsl2Section {
    pub memory: Option<String>,
    pub processors: Option<u32>,
    pub swap: Option<String>,
    pub swap_file: Option<String>,
    pub localhost_forwarding: Option<bool>,
    pub kernel: Option<String>,
    pub kernel_cmdline: Option<String>,
    pub safe_mode: Option<bool>,
}

/// .wslconfig [experimental] 部分配置
#[derive(Debug, Clone, Default)]
pub struct ExperimentalSection {
    pub networking_mode: Option<String>,
    pub firewall: Option<bool>,
    pub auto_proxy: Option<bool>,
    pub auto_memory_reclaim: Option<String>,
    pub dns_tunneling: Option<bool>,
    pub best_effort_dns_parsing: Option<bool>,
    pub host_address_loopback: Option<bool>,
    pub use_windows_host: Option<bool>,
}

/// 完整的 .wslconfig 配置
#[derive(Debug, Clone, Default)]
pub struct WslConfig {
    pub wsl2: Wsl2Section,
    pub experimental: ExperimentalSection,
}

impl WslConfig {
    /// 从文件读取
    pub fn load() -> Result<Self> {
        let path = get_wslconfig_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)?;
        Self::parse(&content)
    }

    /// 保存到文件
    pub fn save(&self) -> Result<()> {
        let path = get_wslconfig_path()?;
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
        let mut current_section: Option<&str> = None;

        for line in content.lines() {
            let line = line.trim();

            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // 检测 section
            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(&line[1..line.len()-1]);
                continue;
            }

            // 解析键值对
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match current_section {
                Some("wsl2") => Self::parse_wsl2_key(&mut config.wsl2, key, value),
                Some("experimental") => Self::parse_experimental_key(&mut config.experimental, key, value),
                _ => {}
            }
        }

        Ok(config)
    }

    fn parse_wsl2_key(section: &mut Wsl2Section, key: &str, value: &str) {
        match key {
            "memory" => section.memory = Some(value.to_string()),
            "processors" => section.processors = value.parse().ok(),
            "swap" => section.swap = Some(value.to_string()),
            "swapFile" => section.swap_file = Some(value.to_string()),
            "localhostForwarding" => section.localhost_forwarding = parse_bool(value),
            "kernel" => section.kernel = Some(value.to_string()),
            "kernelCommandLine" => section.kernel_cmdline = Some(value.to_string()),
            "safeMode" => section.safe_mode = parse_bool(value),
            _ => {}
        }
    }

    fn parse_experimental_key(section: &mut ExperimentalSection, key: &str, value: &str) {
        match key {
            "networkingMode" => section.networking_mode = Some(value.to_string()),
            "firewall" => section.firewall = parse_bool(value),
            "autoProxy" => section.auto_proxy = parse_bool(value),
            "autoMemoryReclaim" => section.auto_memory_reclaim = Some(value.to_string()),
            "dnsTunneling" => section.dns_tunneling = parse_bool(value),
            "bestEffortDnsParsing" => section.best_effort_dns_parsing = parse_bool(value),
            "hostAddressLoopback" => section.host_address_loopback = parse_bool(value),
            "useWindowsHost" => section.use_windows_host = parse_bool(value),
            _ => {}
        }
    }

    /// 转换为 INI 格式
    fn to_ini(&self) -> String {
        let mut sections = Vec::new();

        // [wsl2] section
        let mut wsl2_lines = vec!["[wsl2]".to_string()];
        if let Some(ref v) = self.wsl2.memory {
            wsl2_lines.push(format!("memory={}", v));
        }
        if let Some(v) = self.wsl2.processors {
            wsl2_lines.push(format!("processors={}", v));
        }
        if let Some(ref v) = self.wsl2.swap {
            wsl2_lines.push(format!("swap={}", v));
        }
        if let Some(ref v) = self.wsl2.swap_file {
            wsl2_lines.push(format!("swapFile={}", v));
        }
        if let Some(v) = self.wsl2.localhost_forwarding {
            wsl2_lines.push(format!("localhostForwarding={}", v));
        }
        if let Some(ref v) = self.wsl2.kernel {
            wsl2_lines.push(format!("kernel={}", v));
        }
        if let Some(ref v) = self.wsl2.kernel_cmdline {
            wsl2_lines.push(format!("kernelCommandLine={}", v));
        }
        if let Some(v) = self.wsl2.safe_mode {
            wsl2_lines.push(format!("safeMode={}", v));
        }
        if wsl2_lines.len() > 1 {
            sections.push(wsl2_lines.join("\n"));
        }

        // [experimental] section
        let mut exp_lines = vec!["[experimental]".to_string()];
        if let Some(ref v) = self.experimental.networking_mode {
            exp_lines.push(format!("networkingMode={}", v));
        }
        if let Some(v) = self.experimental.firewall {
            exp_lines.push(format!("firewall={}", v));
        }
        if let Some(v) = self.experimental.auto_proxy {
            exp_lines.push(format!("autoProxy={}", v));
        }
        if let Some(ref v) = self.experimental.auto_memory_reclaim {
            exp_lines.push(format!("autoMemoryReclaim={}", v));
        }
        if let Some(v) = self.experimental.dns_tunneling {
            exp_lines.push(format!("dnsTunneling={}", v));
        }
        if let Some(v) = self.experimental.best_effort_dns_parsing {
            exp_lines.push(format!("bestEffortDnsParsing={}", v));
        }
        if let Some(v) = self.experimental.host_address_loopback {
            exp_lines.push(format!("hostAddressLoopback={}", v));
        }
        if let Some(v) = self.experimental.use_windows_host {
            exp_lines.push(format!("useWindowsHost={}", v));
        }
        if exp_lines.len() > 1 {
            sections.push(exp_lines.join("\n"));
        }

        if sections.is_empty() {
            "[wsl2]\n".to_string()
        } else {
            sections.join("\n\n") + "\n"
        }
    }

    /// 验证配置
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // 验证 memory 格式
        if let Some(ref mem) = self.wsl2.memory {
            if !mem.ends_with("GB") && !mem.ends_with("MB") && !mem.ends_with("%") {
                warnings.push(format!("memory 格式可能无效: {} (建议: 4GB, 512MB, 50%)", mem));
            }
        }

        // 验证 swap 格式
        if let Some(ref swap) = self.wsl2.swap {
            if !swap.ends_with("GB") && !swap.ends_with("MB") && swap != "0" {
                warnings.push(format!("swap 格式可能无效: {} (建议: 2GB, 512MB, 0)", swap));
            }
        }

        // 验证 networkingMode
        if let Some(ref mode) = self.experimental.networking_mode {
            if mode != "NAT" && mode != "mirrored" && mode != "bridged" {
                warnings.push(format!("networkingMode 可能无效: {} (有效值: NAT, mirrored, bridged)", mode));
            }
        }

        // 验证 autoMemoryReclaim
        if let Some(ref reclaim) = self.experimental.auto_memory_reclaim {
            if reclaim != "gradual" && reclaim != "dropcache" && reclaim != "disabled" {
                warnings.push(format!("autoMemoryReclaim 可能无效: {} (有效值: gradual, dropcache, disabled)", reclaim));
            }
        }

        warnings
    }
}

// ============================================================================
// wsl.conf - 发行版配置 (/etc/wsl.conf)
// ============================================================================

/// wsl.conf [boot] 部分
#[derive(Debug, Clone, Default)]
pub struct BootSection {
    pub systemd: Option<bool>,
    pub command: Option<String>,
}

/// wsl.conf [automount] 部分
#[derive(Debug, Clone, Default)]
pub struct AutomountSection {
    pub enabled: Option<bool>,
    pub mount_fs_tab: Option<bool>,
    pub root: Option<String>,
    pub options: Option<String>,
}

/// wsl.conf [network] 部分
#[derive(Debug, Clone, Default)]
pub struct NetworkSection {
    pub generate_hosts: Option<bool>,
    pub generate_resolv_conf: Option<bool>,
    pub hostname: Option<String>,
}

/// wsl.conf [interop] 部分
#[derive(Debug, Clone, Default)]
pub struct InteropSection {
    pub enabled: Option<bool>,
    pub append_windows_path: Option<bool>,
}

/// wsl.conf [user] 部分
#[derive(Debug, Clone, Default)]
pub struct UserSection {
    pub default: Option<String>,
}

/// 完整的 wsl.conf 配置
#[derive(Debug, Clone, Default)]
pub struct WslConf {
    pub boot: BootSection,
    pub automount: AutomountSection,
    pub network: NetworkSection,
    pub interop: InteropSection,
    pub user: UserSection,
}

impl WslConf {
    /// 从文件读取
    pub fn load() -> Result<Self> {
        let path = PathBuf::from("/etc/wsl.conf");
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)?;
        Self::parse(&content)
    }

    /// 保存到文件 (需要 root)
    pub fn save(&self) -> Result<()> {
        if unsafe { libc::getuid() } != 0 {
            return Err(WsuError::NeedRoot);
        }
        let content = self.to_ini();
        fs::write("/etc/wsl.conf", content)?;
        println!("\x1b[32m配置已保存到 /etc/wsl.conf\x1b[0m");
        Ok(())
    }

    fn parse(content: &str) -> Result<Self> {
        let mut config = Self::default();
        let mut current_section: Option<&str> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(&line[1..line.len()-1]);
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match current_section {
                Some("boot") => match key {
                    "systemd" => config.boot.systemd = parse_bool(value),
                    "command" => config.boot.command = Some(value.to_string()),
                    _ => {}
                },
                Some("automount") => match key {
                    "enabled" => config.automount.enabled = parse_bool(value),
                    "mountFsTab" => config.automount.mount_fs_tab = parse_bool(value),
                    "root" => config.automount.root = Some(value.to_string()),
                    "options" => config.automount.options = Some(value.to_string()),
                    _ => {}
                },
                Some("network") => match key {
                    "generateHosts" => config.network.generate_hosts = parse_bool(value),
                    "generateResolvConf" => config.network.generate_resolv_conf = parse_bool(value),
                    "hostname" => config.network.hostname = Some(value.to_string()),
                    _ => {}
                },
                Some("interop") => match key {
                    "enabled" => config.interop.enabled = parse_bool(value),
                    "appendWindowsPath" => config.interop.append_windows_path = parse_bool(value),
                    _ => {}
                },
                Some("user") => match key {
                    "default" => config.user.default = Some(value.to_string()),
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(config)
    }

    fn to_ini(&self) -> String {
        let mut sections = Vec::new();

        // [boot]
        let mut boot_lines = vec!["[boot]".to_string()];
        if let Some(v) = self.boot.systemd {
            boot_lines.push(format!("systemd={}", v));
        }
        if let Some(ref v) = self.boot.command {
            boot_lines.push(format!("command={}", v));
        }
        if boot_lines.len() > 1 {
            sections.push(boot_lines.join("\n"));
        }

        // [automount]
        let mut auto_lines = vec!["[automount]".to_string()];
        if let Some(v) = self.automount.enabled {
            auto_lines.push(format!("enabled={}", v));
        }
        if let Some(v) = self.automount.mount_fs_tab {
            auto_lines.push(format!("mountFsTab={}", v));
        }
        if let Some(ref v) = self.automount.root {
            auto_lines.push(format!("root={}", v));
        }
        if let Some(ref v) = self.automount.options {
            auto_lines.push(format!("options={}", v));
        }
        if auto_lines.len() > 1 {
            sections.push(auto_lines.join("\n"));
        }

        // [network]
        let mut net_lines = vec!["[network]".to_string()];
        if let Some(v) = self.network.generate_hosts {
            net_lines.push(format!("generateHosts={}", v));
        }
        if let Some(v) = self.network.generate_resolv_conf {
            net_lines.push(format!("generateResolvConf={}", v));
        }
        if let Some(ref v) = self.network.hostname {
            net_lines.push(format!("hostname={}", v));
        }
        if net_lines.len() > 1 {
            sections.push(net_lines.join("\n"));
        }

        // [interop]
        let mut interop_lines = vec!["[interop]".to_string()];
        if let Some(v) = self.interop.enabled {
            interop_lines.push(format!("enabled={}", v));
        }
        if let Some(v) = self.interop.append_windows_path {
            interop_lines.push(format!("appendWindowsPath={}", v));
        }
        if interop_lines.len() > 1 {
            sections.push(interop_lines.join("\n"));
        }

        // [user]
        let mut user_lines = vec!["[user]".to_string()];
        if let Some(ref v) = self.user.default {
            user_lines.push(format!("default={}", v));
        }
        if user_lines.len() > 1 {
            sections.push(user_lines.join("\n"));
        }

        if sections.is_empty() {
            "# wsl.conf - WSL 发行版配置\n".to_string()
        } else {
            "# wsl.conf - WSL 发行版配置\n".to_string() + &sections.join("\n\n") + "\n"
        }
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn get_wslconfig_path() -> Result<PathBuf> {
    let output = std::process::Command::new("cmd.exe")
        .args(["/c", "echo %USERPROFILE%"])
        .output()?;

    let win_path = String::from_utf8_lossy(&output.stdout)
        .trim()
        .replace('\\', "/")
        .trim_end_matches('/').to_string();

    let wsl_path = if win_path.starts_with("C:") || win_path.starts_with("c:") {
        format!("/mnt/c{}", &win_path[2..])
    } else if let Some(drive) = win_path.chars().next() {
        format!("/mnt/{}{}", drive.to_lowercase(), &win_path[1..])
    } else {
        return Err(WsuError::Config("无法确定 Windows 用户目录".to_string()));
    };

    Ok(PathBuf::from(wsl_path).join(".wslconfig"))
}

// ============================================================================
// CLI 命令实现
// ============================================================================

/// 显示所有配置
pub fn show() -> Result<()> {
    let wslconfig = WslConfig::load()?;
    let wslconf = WslConf::load()?;
    let wslconfig_path = get_wslconfig_path()?;

    // 验证配置
    let warnings = wslconfig.validate();

    // 显示 .wslconfig
    println!("\x1b[1;36m╔══════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m              \x1b[1;33m.wslconfig (全局配置)\x1b[0m                    \x1b[1;36m║\x1b[0m");
    println!("\x1b[1;36m╠══════════════════════════════════════════════════════╣\x1b[0m");

    if !wslconfig_path.exists() {
        println!("\x1b[1;36m║\x1b[0m \x1b[33m文件不存在，使用默认配置\x1b[0m                          \x1b[1;36m║\x1b[0m");
    }

    println!("\x1b[1;36m║\x1b[0m \x1b[32m[wsl2] 虚拟机设置\x1b[0m                                 \x1b[1;36m║\x1b[0m");
    print_item("  memory", wslconfig.wsl2.memory.as_deref(), "系统自动");
    print_item("  processors", wslconfig.wsl2.processors.map(|v| v.to_string()).as_deref(), "系统自动");
    print_item("  swap", wslconfig.wsl2.swap.as_deref(), "系统自动");
    print_item("  swapFile", wslconfig.wsl2.swap_file.as_deref(), "无");
    print_item("  localhostForwarding", wslconfig.wsl2.localhost_forwarding.map(|v| v.to_string()).as_deref(), "true");
    print_item("  kernel", wslconfig.wsl2.kernel.as_deref(), "默认");
    print_item("  safeMode", wslconfig.wsl2.safe_mode.map(|v| v.to_string()).as_deref(), "false");

    println!("\x1b[1;36m║\x1b[0m                                                      \x1b[1;36m║\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m \x1b[32m[experimental] 实验性功能\x1b[0m                           \x1b[1;36m║\x1b[0m");
    print_item("  networkingMode", wslconfig.experimental.networking_mode.as_deref(), "NAT");
    print_item("  firewall", wslconfig.experimental.firewall.map(|v| v.to_string()).as_deref(), "true");
    print_item("  autoProxy", wslconfig.experimental.auto_proxy.map(|v| v.to_string()).as_deref(), "false");
    print_item("  autoMemoryReclaim", wslconfig.experimental.auto_memory_reclaim.as_deref(), "disabled");
    print_item("  dnsTunneling", wslconfig.experimental.dns_tunneling.map(|v| v.to_string()).as_deref(), "false");

    println!("\x1b[1;36m╠══════════════════════════════════════════════════════╣\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m              \x1b[1;33mwsl.conf (发行版配置)\x1b[0m                      \x1b[1;36m║\x1b[0m");
    println!("\x1b[1;36m╠══════════════════════════════════════════════════════╣\x1b[0m");

    println!("\x1b[1;36m║\x1b[0m \x1b[32m[boot] 启动设置\x1b[0m                                    \x1b[1;36m║\x1b[0m");
    print_item("  systemd", wslconf.boot.systemd.map(|v| v.to_string()).as_deref(), "false");
    print_item("  command", wslconf.boot.command.as_deref(), "无");

    println!("\x1b[1;36m║\x1b[0m \x1b[32m[automount] 自动挂载\x1b[0m                                \x1b[1;36m║\x1b[0m");
    print_item("  enabled", wslconf.automount.enabled.map(|v| v.to_string()).as_deref(), "true");
    print_item("  root", wslconf.automount.root.as_deref(), "/mnt/");

    println!("\x1b[1;36m║\x1b[0m \x1b[32m[interop] 互操作性\x1b[0m                                   \x1b[1;36m║\x1b[0m");
    print_item("  enabled", wslconf.interop.enabled.map(|v| v.to_string()).as_deref(), "true");
    print_item("  appendWindowsPath", wslconf.interop.append_windows_path.map(|v| v.to_string()).as_deref(), "true");

    println!("\x1b[1;36m║\x1b[0m \x1b[32m[user] 用户设置\x1b[0m                                      \x1b[1;36m║\x1b[0m");
    print_item("  default", wslconf.user.default.as_deref(), "安装时设置");

    // 显示验证警告
    if !warnings.is_empty() {
        println!("\x1b[1;36m╠══════════════════════════════════════════════════════╣\x1b[0m");
        println!("\x1b[1;36m║\x1b[0m \x1b[31m⚠ 配置警告\x1b[0m                                          \x1b[1;36m║\x1b[0m");
        for warning in &warnings {
            println!("\x1b[1;36m║\x1b[0m   \x1b[33m{}\x1b[0m", pad_right(warning, 50));
            println!("\x1b[1;36m║\x1b[0m                                                      \x1b[1;36m║\x1b[0m");
        }
    }

    println!("\x1b[1;36m╚══════════════════════════════════════════════════════╝\x1b[0m");

    // 显示用法提示
    println!("\n\x1b[36m用法:\x1b[0m");
    println!("  \x1b[32m# .wslconfig (全局)\x1b[0m");
    println!("  wsu config set memory 4GB");
    println!("  wsu config set networkingMode mirrored");
    println!("  wsu config set autoProxy true");
    println!("  wsu config set autoMemoryReclaim gradual");
    println!("");
    println!("  \x1b[32m# wsl.conf (发行版，部分需要 root)\x1b[0m");
    println!("  sudo wsu config set-systemd true");
    println!("  sudo wsu config set-hostname mywsl");
    println!("");
    println!("  \x1b[32m# 编辑配置文件\x1b[0m");
    println!("  wsu config edit        # 编辑 .wslconfig");
    println!("  sudo wsu config edit-conf  # 编辑 wsl.conf");

    Ok(())
}

fn print_item(label: &str, value: Option<&str>, default: &str) {
    let display = value.unwrap_or(default);
    println!("\x1b[1;36m║\x1b[0m {}: {}", pad_right(label, 22), display);
}

fn pad_right(s: &str, width: usize) -> String {
    let display_width = UnicodeWidthStr::width(s);
    let padding = width.saturating_sub(display_width);
    format!("{}{}", s, " ".repeat(padding))
}

/// 设置 .wslconfig 配置项
pub fn set(key: &str, value: &str) -> Result<()> {
    let mut config = WslConfig::load()?;

    // [wsl2] 配置项
    match key {
        "memory" => {
            validate_memory_format(value)?;
            config.wsl2.memory = Some(value.to_string());
        }
        "processors" => {
            let n: u32 = value.parse().map_err(|_| {
                WsuError::Parse("处理器数必须是正整数".to_string())
            })?;
            if n == 0 {
                return Err(WsuError::Parse("处理器数必须大于 0".to_string()));
            }
            config.wsl2.processors = Some(n);
        }
        "swap" => {
            validate_memory_format(value)?;
            config.wsl2.swap = Some(value.to_string());
        }
        "swapFile" => {
            config.wsl2.swap_file = Some(value.to_string());
        }
        "localhostForwarding" => {
            config.wsl2.localhost_forwarding = Some(parse_bool(value).unwrap_or(true));
        }
        "kernel" => {
            config.wsl2.kernel = Some(value.to_string());
        }
        "kernelCommandLine" => {
            config.wsl2.kernel_cmdline = Some(value.to_string());
        }
        "safeMode" => {
            config.wsl2.safe_mode = Some(parse_bool(value).unwrap_or(false));
        }
        // [experimental] 配置项
        "networkingMode" | "networking" => {
            validate_networking_mode(value)?;
            config.experimental.networking_mode = Some(value.to_string());
        }
        "firewall" => {
            config.experimental.firewall = Some(parse_bool(value).unwrap_or(true));
        }
        "autoProxy" => {
            config.experimental.auto_proxy = Some(parse_bool(value).unwrap_or(false));
        }
        "autoMemoryReclaim" | "memoryReclaim" => {
            validate_auto_memory_reclaim(value)?;
            config.experimental.auto_memory_reclaim = Some(value.to_string());
        }
        "dnsTunneling" => {
            config.experimental.dns_tunneling = Some(parse_bool(value).unwrap_or(false));
        }
        "bestEffortDnsParsing" => {
            config.experimental.best_effort_dns_parsing = Some(parse_bool(value).unwrap_or(false));
        }
        "hostAddressLoopback" => {
            config.experimental.host_address_loopback = Some(parse_bool(value).unwrap_or(false));
        }
        "useWindowsHost" => {
            config.experimental.use_windows_host = Some(parse_bool(value).unwrap_or(false));
        }
        _ => {
            return Err(WsuError::Config(format!(
                "未知配置项: {}\n\n\
                 [wsl2] 配置项:\n  memory, processors, swap, swapFile, localhostForwarding, kernel, safeMode\n\n\
                 [experimental] 配置项:\n  networkingMode, firewall, autoProxy, autoMemoryReclaim, dnsTunneling",
                key
            )));
        }
    }

    // 验证并保存
    let warnings = config.validate();
    for warning in &warnings {
        println!("\x1b[33m警告: {}\x1b[0m", warning);
    }

    config.save()?;
    show()?;

    Ok(())
}

/// 设置 wsl.conf 配置项 (需要 root)
pub fn set_conf(key: &str, value: &str) -> Result<()> {
    if unsafe { libc::getuid() } != 0 {
        return Err(WsuError::NeedRoot);
    }

    let mut conf = WslConf::load()?;

    match key {
        "systemd" => {
            conf.boot.systemd = Some(parse_bool(value).unwrap_or(false));
        }
        "bootCommand" => {
            conf.boot.command = Some(value.to_string());
        }
        "automountEnabled" => {
            conf.automount.enabled = Some(parse_bool(value).unwrap_or(true));
        }
        "automountRoot" => {
            conf.automount.root = Some(value.to_string());
        }
        "automountOptions" => {
            conf.automount.options = Some(value.to_string());
        }
        "generateHosts" => {
            conf.network.generate_hosts = Some(parse_bool(value).unwrap_or(true));
        }
        "generateResolvConf" => {
            conf.network.generate_resolv_conf = Some(parse_bool(value).unwrap_or(true));
        }
        "hostname" => {
            conf.network.hostname = Some(value.to_string());
        }
        "interopEnabled" => {
            conf.interop.enabled = Some(parse_bool(value).unwrap_or(true));
        }
        "appendWindowsPath" => {
            conf.interop.append_windows_path = Some(parse_bool(value).unwrap_or(true));
        }
        "defaultUser" => {
            conf.user.default = Some(value.to_string());
        }
        _ => {
            return Err(WsuError::Config(format!(
                "未知 wsl.conf 配置项: {}\n\n\
                 [boot] 配置项:\n  systemd, bootCommand\n\n\
                 [automount] 配置项:\n  automountEnabled, automountRoot, automountOptions\n\n\
                 [network] 配置项:\n  generateHosts, generateResolvConf, hostname\n\n\
                 [interop] 配置项:\n  interopEnabled, appendWindowsPath\n\n\
                 [user] 配置项:\n  defaultUser",
                key
            )));
        }
    }

    conf.save()?;
    show()?;

    Ok(())
}

// 验证函数
fn validate_memory_format(value: &str) -> Result<()> {
    let valid = value.ends_with("GB")
        || value.ends_with("MB")
        || value.ends_with("%")
        || value == "0";

    if !valid {
        println!("\x1b[33m提示: 内存格式建议为 4GB, 512MB, 50% 或 0\x1b[0m");
    }
    Ok(())
}

fn validate_networking_mode(value: &str) -> Result<()> {
    match value {
        "NAT" | "mirrored" | "bridged" => Ok(()),
        _ => {
            println!("\x1b[33m警告: networkingMode 有效值为 NAT, mirrored, bridged\x1b[0m");
            Ok(())
        }
    }
}

fn validate_auto_memory_reclaim(value: &str) -> Result<()> {
    match value {
        "gradual" | "dropcache" | "disabled" => Ok(()),
        _ => {
            println!("\x1b[33m警告: autoMemoryReclaim 有效值为 gradual, dropcache, disabled\x1b[0m");
            Ok(())
        }
    }
}

/// 用编辑器打开 .wslconfig
pub fn edit() -> Result<()> {
    let path = get_wslconfig_path()?;

    if !path.exists() {
        let config = WslConfig::default();
        let content = config.to_ini();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
    }

    let path_str = path.to_string_lossy().to_string();
    std::process::Command::new("cmd.exe")
        .args(["/c", "start", "", &path_str])
        .spawn()?;

    println!("\x1b[32m已打开 .wslconfig\x1b[0m");
    Ok(())
}

/// 用编辑器打开 wsl.conf (需要 root)
pub fn edit_conf() -> Result<()> {
    let path = PathBuf::from("/etc/wsl.conf");

    if !path.exists() {
        let conf = WslConf::default();
        let content = conf.to_ini();
        fs::write(&path, content)?;
    }

    // 使用 nano 或 vim 打开
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    std::process::Command::new(&editor)
        .arg("/etc/wsl.conf")
        .status()?;

    Ok(())
}