use crate::error::Result;
use dialoguer::{MultiSelect, theme::ColorfulTheme};

const TEMPLATES: &[(&str, &[&str])] = &[
    ("基础工具", &["git", "curl", "wget", "build-essential"]),
    ("Rust 开发", &["rustup", "cargo", "rust-analyzer"]),
    ("Node.js 开发", &["nodejs", "npm", "nvm"]),
    ("Python 开发", &["python3", "pip", "pyenv"]),
    ("常用 CLI", &["bat", "ripgrep", "fd-find", "exa", "starship"]),
    ("Shell 增强", &["zsh", "oh-my-zsh"]),
];

fn get_package_manager() -> &'static str {
    if std::path::Path::new("/usr/bin/apt").exists() {
        "apt"
    } else if std::path::Path::new("/usr/bin/dnf").exists() {
        "dnf"
    } else if std::path::Path::new("/usr/bin/yum").exists() {
        "yum"
    } else if std::path::Path::new("/usr/bin/pacman").exists() {
        "pacman"
    } else {
        "apt" // 默认
    }
}

fn map_package(pkg: &str, pm: &str) -> String {
    // 不同包管理器的包名映射
    match (pkg, pm) {
        ("ripgrep", "apt") => "ripgrep".to_string(),
        ("ripgrep", "pacman") => "ripgrep".to_string(),
        ("ripgrep", _) => "ripgrep".to_string(),
        ("fd-find", "pacman") => "fd".to_string(),
        ("fd-find", _) => "fd-find".to_string(),
        _ => pkg.to_string(),
    }
}

pub fn run(template: Option<&str>) -> Result<()> {
    println!("\x1b[1;36m╔══════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m      \x1b[1;33mWSL2 开发环境初始化\x1b[0m              \x1b[1;36m║\x1b[0m");
    println!("\x1b[1;36m╚══════════════════════════════════════╝\x1b[0m");

    let pm = get_package_manager();
    println!("\n\x1b[36m检测到包管理器: {}\x1b[0m\n", pm);

    let mut selected_packages = Vec::new();

    if let Some(t) = template {
        // 使用预设模板
        println!("\x1b[36m使用模板: {}\x1b[0m\n", t);

        let template_map: std::collections::HashMap<&str, &[&str]> = {
            let mut m = std::collections::HashMap::new();
            m.insert("full", &[
                "git", "curl", "wget", "build-essential",
                "rustup", "nodejs", "npm", "python3", "pip",
                "bat", "ripgrep", "fd-find", "exa", "starship", "zsh"
            ][..]);
            m.insert("rust", &["git", "curl", "build-essential", "rustup", "bat", "ripgrep"][..]);
            m.insert("node", &["git", "curl", "build-essential", "nodejs", "npm"][..]);
            m.insert("python", &["git", "curl", "build-essential", "python3", "pip"][..]);
            m
        };

        if let Some(packages) = template_map.get(t) {
            selected_packages = packages.iter().map(|s| s.to_string()).collect();
        } else {
            println!("\x1b[31m未知模板: {}\x1b[0m", t);
            println!("可用模板: full, rust, node, python");
            return Ok(());
        }
    } else {
        // 交互式选择
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要安装的工具类别")
            .items(&TEMPLATES.iter().map(|(name, _)| *name).collect::<Vec<_>>())
            .defaults(&[true, false, false, false, true, false])
            .interact()?;

        for idx in selections {
            selected_packages.extend(TEMPLATES[idx].1.iter().map(|s| s.to_string()));
        }
    }

    if selected_packages.is_empty() {
        println!("\x1b[33m未选择任何工具\x1b[0m");
        return Ok(());
    }

    // 去重
    selected_packages.sort();
    selected_packages.dedup();

    println!("\n\x1b[36m将安装以下工具:\x1b[0m");
    for pkg in &selected_packages {
        println!("  • {}", pkg);
    }

    // 生成安装命令
    let mapped_packages: Vec<String> = selected_packages
        .iter()
        .map(|p| map_package(p, pm))
        .collect();

    let install_cmd = match pm {
        "apt" => format!("sudo apt update && sudo apt install -y {}", mapped_packages.join(" ")),
        "dnf" => format!("sudo dnf install -y {}", mapped_packages.join(" ")),
        "yum" => format!("sudo yum install -y {}", mapped_packages.join(" ")),
        "pacman" => format!("sudo pacman -S --noconfirm {}", mapped_packages.join(" ")),
        _ => format!("sudo apt install -y {}", mapped_packages.join(" ")),
    };

    println!("\n\x1b[33m运行以下命令安装:\x1b[0m");
    println!("\x1b[36m{}\x1b[0m\n", install_cmd);

    println!("\x1b[33m提示: 部分工具 (如 rustup, nvm, pyenv) 需要额外安装脚本\x1b[0m");

    Ok(())
}