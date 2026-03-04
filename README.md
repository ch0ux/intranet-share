# Easy Share

<p align="center">
    <img src="./app-icon.png" alt="Database Proxy Log logo" width="200" height="200">
</p>

<div align="center">

一个基于 Tauri 2.0 的轻量级局域网文件共享工具

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/tauri-2.0-24c8db.svg)](https://tauri.app)
[![Vue](https://img.shields.io/badge/vue-3.5-4fc08d.svg)](https://vuejs.org)

</div>

## 📖 简介

Easy Share 是一个现代化、高性能的局域网文件共享应用程序。它使用 Rust 和 Tauri 构建，提供了简洁直观的用户界面，让您能够轻松地在局域网内发现和传输文件。

## 截图

<img src="doc\app-screenshot1.png" style="zoom:30%;" />


## ✨ 特性

- 🚀 **快速发现** - 基于 mDNS 协议自动发现局域网内的设备
- 🎯 **简单易用** - 简洁直观的界面，无需复杂配置
- 🔒 **安全可靠** - 使用 Blake3 哈希算法确保文件完整性
- 💡 **轻量级** - 基于 Tauri，安装包体积小，内存占用低
- 🎨 **现代化 UI** - 使用 Vue 3 + Naive UI 构建的响应式界面
- 📦 **跨平台** - 支持 Windows、macOS 和 Linux


## 🛠️ 技术栈

### 前端
- **框架**: [Vue 3.5](https://vuejs.org) + TypeScript
- **构建工具**: [Vite 6.0](https://vite.dev)
- **UI 组件库**: [Naive UI](https://naive-ui.com)
- **状态管理**: [Pinia](https://pinia.vuejs.org)

### 后端
- **语言**: [Rust](https://www.rust-lang.org) (Edition 2024)
- **框架**: [Tauri 2.0](https://tauri.app)
- **异步运行时**: [Tokio](https://tokio.rs)
- **服务发现**: [mdns-sd](https://crates.io/crates/mdns-sd)
- **哈希算法**: [Blake3](https://github.com/BLAKE3-team/BLAKE3)

## 📋 环境要求

在开始之前，请确保您的开发环境满足以下要求：

- [Node.js](https://nodejs.org) >= 18.x
- [Rust](https://www.rust-lang.org/tools/install) >= 1.75
- Git

### 推荐 IDE 配置

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 🚀 快速开始

### 克隆项目

```bash
git clone https://github.com/ch0ux/intranet-share
cd intranet-share
```

### 安装依赖
```bash
# 安装 Node.js 依赖
npm install
#启动开发服务器
npm run tauri dev
```
### 构建发布版本
```bash
# 构建应用程序
npm run tauri build
```
构建完成后，可执行文件和安装包将生成在 `src-tauri/target/release` 目录下。

## 📜 可用命令

| 命令 | 描述 |
|------|------|
| `npm run dev` | 启动 Vite 开发服务器 |
| `npm run build` | 构建前端资源 |
| `npm run preview` | 预览构建结果 |
| `npm run tauri dev` | 启动 Tauri 开发环境 |
| `npm run tauri build` | 构建 Tauri 应用 |

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

感谢以下优秀的开源项目：

- [Tauri](https://tauri.app) - 构建安全的跨平台桌面应用
- [Vue.js](https://vuejs.org) - 渐进式 JavaScript 框架
- [Naive UI](https://naive-ui.com) - Vue 3 组件库
- [Rust](https://www.rust-lang.org) - 赋予每个人构建高效软件的能力

## 📬 联系方式

- 作者：choux
- Issues: [GitHub Issues](https://github.com/ch0ux/intranet-share/issues)

<p>
如果这个项目对你有帮助，请给一个 ⭐️ Star 支持一下！
</p>