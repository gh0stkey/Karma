<div align="center">
<img src="src-tauri/icons/logo.png" style="width: 20%" />
<h4><a href="https://github.com/gh0stkey/Karma">本地化 PII 检测与脱敏，基于 MLX 驱动。</a></h4>
<h5>作者：<a href="https://github.com/gh0stkey">EvilChen</a></h5>
</div>

README 版本: \[[English](README.md) | [简体中文](README_CN.md)\]

## 项目介绍

**Karma** 是一款隐私优先的桌面应用，用于检测和脱敏个人身份信息（PII），基于 [OpenAI Privacy Filter](https://huggingface.co/openai-community/openai-privacy-filter) 模型。借助 Apple MLX 框架，所有 AI 推理完全在本地设备上运行——您的数据永远不会离开您的设备。

Karma 可识别文本中 **9 种类型的 PII** 并替换为标签占位符：

| PII 类型 | 占位符 |
|----------|--------|
| 人名 | `[PERSON]` |
| 邮箱地址 | `[EMAIL]` |
| 电话号码 | `[PHONE]` |
| 物理地址 | `[ADDRESS]` |
| 日期 | `[DATE]` |
| URL | `[URL]` |
| 账号 | `[ACCOUNT_NUMBER]` |
| 密钥/密码 | `[SECRET]` |

## 功能特性

- **实时 PII 检测**：基于 MLX 在 Apple Silicon 上进行 Token 级别分类
- **文本脱敏**：一键脱敏，支持自动复制结果到剪贴板
- **HTTP API 服务**：内置 REST API 服务器（`/health`、`/redact`），可与其他工具集成
- **脱敏历史**：基于 SQLite 的历史记录，支持无限滚动浏览
- **全局快捷键**：可配置的快捷键快速唤起（默认：`Cmd+Shift+K`）
- **双语界面**：支持英文和简体中文
- **零云端依赖**：所有处理均在本地完成，确保完全隐私

## 技术栈

| 组件 | 技术 |
|------|------|
| 桌面框架 | Tauri 2（Rust） |
| 前端 | React + TypeScript + Tailwind CSS |
| 状态管理 | Zustand |
| AI 推理 | MLX + MLX Embeddings（Python Sidecar） |
| HTTP 服务 | Axum |
| 数据库 | SQLite（rusqlite） |
| 构建工具 | Vite |

## 安装

从 [Releases](https://github.com/gh0stkey/Karma/releases) 页面下载最新的 `.dmg` 安装包。

> **系统要求**：macOS 11.0+ 且搭载 Apple Silicon（M1/M2/M3/M4）

## 使用方法

### 快速开始

1. 打开 Karma，进入 **模型** 页面
2. 选择本地 MLX Token Classification 模型目录
3. 等待模型加载完成（状态指示灯变为绿色）
4. 切换到 **脱敏** 页面，粘贴文本，点击 **脱敏**

### 全局快捷键

按下 `Cmd+Shift+K`（可在设置中自定义）即可立即唤起 Karma 并跳转到脱敏页面。

### HTTP API

在 **服务** 页面启用内置 HTTP 服务器，即可使用脱敏 API：

```bash
# 健康检查
curl http://127.0.0.1:8000/health

# 文本脱敏
curl -X POST http://127.0.0.1:8000/redact \
  -H "Content-Type: application/json" \
  -d '{"text": "我叫张三，邮箱是 zhangsan@example.com"}'
```

## 从源码构建

### 前置条件

- Node.js >= 20
- Rust（stable）
- [uv](https://github.com/astral-sh/uv)（Python 包管理器）

### 构建步骤

```bash
# 安装前端依赖
npm install

# 构建 Sidecar 二进制文件（PyInstaller）
make sidecar

# 构建 Tauri 应用（DMG）
make app
```

或执行完整流水线：

```bash
make all
```

## 界面说明

| 页面 | 说明 |
|------|------|
| 脱敏 | 输入文本，检测 PII，查看脱敏结果和检测到的实体 |
| 模型 | 配置模型路径，查看已加载模型的元数据 |
| 服务 | 启用/配置 HTTP API 服务器，查看 API 文档和请求日志 |
| 历史 | 浏览历史脱敏记录，支持分页 |
| 设置 | 全局快捷键、语言、自动复制、历史上限等 |
| 关于 | 版本信息和技术栈说明 |
