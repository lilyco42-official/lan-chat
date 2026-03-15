# LAN Chat (局域网聊天)

高性能局域网即时通讯工具，使用 Rust + Slint 开发。

## 功能特性

- 🔍 **局域网发现** - 自动发现同局域网内的用户
- 💬 **即时消息** - 实时 TCP 消息传输
- 🎨 **图形界面** - 原生 Slint UI
- ⚡ **高性能** - 纯 Rust 实现，非阻塞 I/O

## 技术栈

- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [Slint](https://slint.dev/) - GUI 框架
- TCP/UDP - 网络通信

## 编译运行

### 前置要求

- Rust 1.70+
- Windows 10/11 或 Linux/macOS

### 编译

```bash
# Debug 版本
cargo build

# Release 版本 (推荐)
cargo build --release
```

### 运行

```bash
# Debug
cargo run

# Release
cargo run --release

# 或直接运行编译好的 exe
./target/release/lan-chat.exe
```

## 使用方法

1. 启动程序后，点击 **刷新** 刷新在线用户列表
2. 点击列表中的用户选中
3. 在消息框输入内容，点击 **发送** 发送消息

## 端口说明

| 端口 | 协议 | 用途 |
|------|------|------|
| 45678 | UDP | 局域网发现/广播 |
| 45679 | TCP | 消息传输 |

## 项目结构

```
lan-chat/
├── src/
│   ├── main.rs    # 主程序入口
│   └── network.rs # 网络模块
├── ui/
│   └── main.slint # UI 定义
├── Cargo.toml     # 依赖配置
└── build.rs      # 构建脚本
```

## 下载预编译版本

请访问 [Releases](https://github.com/lilyco42-official/lan-chat/releases) 页面下载预编译的二进制文件。

## 许可证

MIT License - see [LICENSE](LICENSE) file

## 贡献

欢迎提交 Issue 和 Pull Request！
