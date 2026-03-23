# Ace Translator

Ace Translator 是一个基于 Tauri + Vue 3 的桌面 AI 翻译工具，面向高频文本翻译场景，支持多服务商接入、流式翻译与语音播放。

## 核心能力

- OpenAI 协议统一接入：支持 OpenAI 兼容接口服务商与自定义服务商。
- 预置服务商：智谱、OpenAI、DeepSeek、小米 MiMo、MiniMax、Moonshot、Ollama。
- 单服务商启用：翻译服务商采用单选启用策略，避免配置冲突。
- 独立 TTS：语音服务与翻译服务解耦，可单独配置 TTS（当前预置小米 TTS）。
- 结果播放：支持原文和译文一键语音播放。
- 本地语言检测：优先使用本地检测（whatlang）降低额外网络请求延迟。
- 数据安全：API Key 本地加密存储，翻译历史保存在本地 SQLite。

## 技术栈

### 前端

- Vue 3 + TypeScript
- Vite
- Pinia
- Vue Router
- Tailwind CSS

### 后端

- Tauri 2
- Rust + Tokio
- SQLx (SQLite)
- Reqwest

## 本地开发

```bash
pnpm install
pnpm tauri dev
```

## 构建

```bash
pnpm tauri build
```

按平台构建可参考 Tauri 官方文档配置对应 target 与签名环境变量。

## 配置说明

- 翻译服务商：设置中启用一个翻译服务商，并填写 `API Key / model / base_url`。
- 语音服务商：设置中启用一个语音服务商，并填写 `API Key / model / base_url / voice / format`。
- 自定义服务商：只要支持 OpenAI 兼容协议，即可新增并使用。

## 开源与贡献

- GitHub: https://github.com/ace0109/ace-translator
- 欢迎提交 Issue 和 PR，一起改进产品体验。
