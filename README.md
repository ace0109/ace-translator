# Ace Translator

Ace Translator 是一个基于 **Tauri 2 + Vue 3 + Rust** 的桌面 AI 翻译工具，面向高频阅读、写作与跨语言沟通场景。  
项目支持 OpenAI 兼容协议接入、多服务商切换、流式翻译与语音播放。

- GitHub: https://github.com/ace0109/ace-translator
- Changelog: [CHANGELOG.md](./CHANGELOG.md)

## 当前版本

当前版本：`v1.3.0`  
主要升级：OpenAI 协议统一、独立 TTS、原文/译文播放、本地语言检测提速。

## 核心特性

- OpenAI 协议统一接入：预置服务商 + 自定义服务商。
- 单服务商启用策略：同一时刻仅一个翻译服务商生效，避免冲突。
- 模型自由输入：`model / base_url` 不限制预设选项。
- 模型测速：支持多模型对比，列表可缓存并复用。
- 独立 TTS 模块：语音服务与翻译服务解耦，当前预置小米 TTS。
- 原文/译文播放：支持一键语音合成播放与停止。
- 本地语言检测：优先使用 `whatlang`，减少额外网络请求。
- 本地安全：API Key 本地加密保存，历史记录落地 SQLite。

## 预置服务商

翻译服务商预置：

- Zhipu
- OpenAI
- DeepSeek
- Xiaomi MiMo
- MiniMax
- Moonshot
- Ollama

语音服务商预置：

- Xiaomi MiMo TTS

> 说明：预置仅包含服务商名称、默认模型和默认地址。  
> API Key、模型、地址等最终以用户配置为准。

## 技术栈

前端：

- Vue 3
- TypeScript
- Vite
- Pinia
- Vue Router
- Tailwind CSS

后端：

- Tauri 2
- Rust + Tokio
- SQLx (SQLite)
- Reqwest

## 快速开始

### 环境要求

- Node.js 18+
- pnpm 8+
- Rust stable toolchain
- Tauri 构建依赖（按操作系统安装，参考 Tauri 官方文档）

### 本地开发

```bash
pnpm install
pnpm tauri dev
```

### 前端独立调试

```bash
pnpm dev
```

### 类型检查

```bash
pnpm exec vue-tsc --noEmit
```

### Rust 检查与测试

```bash
cd src-tauri
cargo check
cargo test
```

## 打包构建

```bash
pnpm tauri build
```

常用脚本：

- `pnpm tauri:build:mac`
- `pnpm tauri:build:win`

> 签名与发布请按你的 CI/CD 或本地证书配置处理，不建议在仓库中提交任何私钥。

## 配置说明

### 翻译服务商

在设置页启用一个翻译服务商并填写：

- `api_key`
- `model`
- `base_url`

### 语音服务商（TTS）

在设置页启用一个语音服务商并填写：

- `api_key`
- `model`
- `base_url`
- `voice`
- `audio_format`

### 自定义服务商

只要支持 OpenAI 兼容协议，即可新增自定义服务商并参与翻译。

## 项目结构

```text
.
├── src/                     # Vue 前端
├── src-tauri/               # Rust + Tauri 后端
├── docs/                    # VitePress 文档
├── CHANGELOG.md             # 发布日志
└── README.md
```

## 隐私与安全

- API Key 仅保存在本地数据库，且做本地加密处理。
- 翻译历史默认保存在本地 SQLite，可在应用内清理。
- 开源分支不包含任何私钥或内网接口依赖。

## 贡献指南

欢迎通过 Issue / PR 参与贡献：

1. Fork 仓库并创建功能分支。
2. 提交前执行类型检查和测试。
3. 提交清晰的变更说明与复现步骤。

## License

本项目采用 [MIT License](./LICENSE)。
