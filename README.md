<div align="center">
  <img src="./src/renderer/public/logo.svg" alt="milkup Logo" width="150"> 
  <h1>milkup</h1>
  <p><strong>跨平台即时渲染 Markdown 编辑器 · 基于 Tauri + Vue 3 + Rust</strong></p>
  
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rust-lang.org)
  [![Tauri](https://img.shields.io/badge/tauri-v2-blue.svg)](https://tauri.app)
</div>

## ✨ 特性

### 编辑体验
- 🎯 **即时渲染** — 自研 ProseMirror 引擎，媲美 Typora
- 📝 **双模式编辑** — 即时渲染 / 源码编辑一键切换
- ⌨️ **斜杠命令 `/`** — 输入 `/` 弹出快捷插入面板
- 🔍 **聚焦模式** — `Ctrl+Shift+F` 打字机滚动 + 段落高亮
- 🎬 **演示模式** — `F11` 幻灯片展示

### 内容能力
- 📋 **[TOC] 目录插入** — 一键生成可跳转目录
- 👣 **脚注 `[^1]`** — 自动渲染上标引用
- ✅ **任务列表** — `- [ ]` / `- [x]` 交互式勾选
- 🖼️ **图片缩放对齐** — 拖拽调整尺寸，左中右对齐

### 导出与协作
- 📄 **导出 HTML** — 独立文件含暗色主题
- 📚 **导出 EPUB** — 电子书格式
- 🔍 **全局搜索** — 跨文件递归搜索
- 🔗 **链接检查** — 自动检测失效链接

### 效率工具
- 📊 **字数统计增强** — 字符/行/词/阅读时间
- 📂 **大纲导航** — 文档标题树
- 🎨 **自定义 CSS** — 用户注入样式
- 🌓 **深色模式自动** — 跟随系统

### 系统特性
- 🖥️ **跨平台** — Linux / Windows / macOS
- 📦 **安装包 9.5 MB** — 基于 Tauri v2
- 🪟 **多标签 + 拖拽分离** — Tab 可拖出为独立窗口
- 🔗 **文件关联** — 双击 .md 直接打开
- 🔒 **单实例** — 多个文件在同一窗口打开

## 🏗️ 架构

```
Vue 3 前端 ← electronApiCompat.ts → Rust 后端 (9模块 38命令)
ProseMirror 内核 · CodeMirror 6 · Mermaid · KaTeX
```

| 层级 | 技术 |
|------|------|
| 桌面壳 | Tauri v2 (Rust) |
| 前端 | Vue 3 + TypeScript |
| 编辑器 | 自研 ProseMirror 内核 |
| 后端 | Rust (文件I/O · 导出 · 搜索 · Git diff) |
| 构建 | Vite + Cargo |

## 🚀 快速开始

```bash
# 安装依赖
pnpm install

# 开发模式
pnpm dev

# 构建
pnpm tauri build --bundles deb,rpm

# 交叉编译 Windows
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64 nsis
pnpm tauri build --target x86_64-pc-windows-gnu --bundles nsis
```

## 📄 许可证

MIT License
