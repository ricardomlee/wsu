# wsu - WSL2 智能工具箱

用 Rust 编写的 WSL2 实用工具集，解决 WSL2 的常见痛点。

## 功能

- **`wsu proxy`** - 智能代理配置，自动检测 Windows IP 并设置代理
- **`wsu mem`** - 内存管理，查看状态、回收缓存
- **`wsu sys`** - 查看 WSL2 系统信息
- **`wsu init`** - 开发环境快速初始化

## 安装

```bash
# 从源码编译
git clone https://github.com/user/wsu.git
cd wsu
cargo build --release

# 复制到 PATH
sudo cp target/release/wsu /usr/local/bin/
```

## 使用

### 代理配置

```bash
# 自动检测并配置代理
wsu proxy auto

# 手动设置代理端口
wsu proxy set 7890

# 查看当前代理状态
wsu proxy status

# 清除代理设置
wsu proxy clear

# 导出 shell 配置 (写入 .bashrc)
wsu proxy export >> ~/.bashrc
```

### 内存管理

```bash
# 查看内存状态
wsu mem status

# 回收缓存 (需要 root)
sudo wsu mem reclaim
```

### 系统信息

```bash
# 显示系统信息
wsu sys

# JSON 格式输出
wsu sys --json
```

### 环境初始化

```bash
# 交互式选择
wsu init

# 使用预设模板
wsu init --template rust
wsu init --template node
wsu init --template full
```

## 特点

- 单二进制文件，无依赖
- 快速启动 (~1ms)
- 彩色输出
- 类型安全的错误处理

## License

MIT