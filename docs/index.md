---
layout: home

hero:
  name: "Ace Translator"
  text: "面向日常与开发场景的桌面 AI 翻译工作台"
  tagline: 基于 Tauri 2.0 + Rust 构建，统一 OpenAI 协议接入多服务商，支持独立 TTS 语音播放、本地语言检测与流式翻译体验，轻量、安全、高效。
  actions:
    - theme: brand
      text: 下载 Windows 版
      link: https://nest.wangcaiyuan.com/translator/download?platform=windows
    - theme: alt
      text: 下载 macOS 版
      link: https://nest.wangcaiyuan.com/translator/download?platform=macos
    - theme: alt
      text: GitHub 开源仓库
      link: https://github.com/ace0109/ace-translator

features:
  - title: 🔌 OpenAI 协议统一接入
    details: 内置智谱、OpenAI、DeepSeek、小米 MiMo、MiniMax、Moonshot、Ollama，并支持自定义 OpenAI 兼容服务商。
  - title: ⌨️ 全局快捷触发，减少切换成本
    details: 支持快捷方式快速唤起翻译窗口（双击复制 / 快捷键打开），输入即翻，适合高频阅读与写作场景。
  - title: 🎯 单服务商启用，更易管理
    details: 设置页采用单选启用策略，同一时间仅一个翻译服务商生效，避免配置冲突；模型与接口地址均可自由输入。
  - title: 🧪 模型测速与配置优化
    details: 支持按服务商进行多模型测速对比，测速模型列表可缓存复用，便于快速找到速度与效果更合适的模型。
  - title: 🔊 独立 TTS 语音播放
    details: 翻译服务与语音服务解耦，可单独配置 TTS 服务商（当前已支持小米），支持原文与译文一键播放。
  - title: ⚡ 本地语言检测 + 流式翻译
    details: 语言识别优先走本地 whatlang，减少额外网络请求；翻译过程支持流式返回，首屏反馈更快。
  - title: 🛡️ 本地安全与轻量性能
    details: API Key 本地加密存储，历史记录本地数据库保存；基于 Rust 与 Tauri，低占用、启动快、运行稳定。
---

Ace Translator 已在 GitHub 开源，仓库地址：
[https://github.com/ace0109/ace-translator](https://github.com/ace0109/ace-translator)

## 当前版本亮点

- 翻译服务与语音服务解耦：翻译模型和 TTS 模型可独立配置，能力组合更灵活。
- 配置体验更直接：模型、接口地址均支持自由输入，适配更多 OpenAI 兼容平台。
- 单服务商启用策略：减少误切换与并发冲突，线上使用更稳定。
- 本地语言检测提速：优先本地识别语言，减少额外网络请求等待。
- 可观测与可排错：统一错误码与日志能力，定位问题更高效。

## 典型使用场景

- 开发者阅读英文技术文档、API 文档、开源仓库说明。
- 产品/运营处理多语言素材，快速完成初稿翻译与语义核对。
- 跨境沟通场景下的即时文本转换与语音播放辅助。
- 需要在云模型与本地模型（如 Ollama）之间灵活切换的团队。

## 快速上手

1. 下载并安装应用，打开设置页。
2. 在“AI 服务商配置”中选择并启用一个翻译服务商，填写 API Key、模型与接口地址。
3. 如需语音播放，在“语音合成（TTS）”中启用并配置语音服务商。
4. 回到主窗口输入或粘贴文本，应用会自动检测语言并给出翻译结果。
5. 使用原文/译文播放按钮进行语音合成；需要调优时可通过模型测速对比不同模型响应速度。

你可以在仓库中查看源码、提交 Issue、参与贡献，或直接前往 Releases 获取最新版本。
