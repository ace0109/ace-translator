# ExecPlan: 独立 TTS 模块与小米语音合成接入

## 背景
- 当前产品已完成翻译服务商 OpenAI 协议统一，但翻译结果不支持语音播放。
- 需求是优先接入小米语音合成，并在主页面翻译结果区域支持一键播放。
- 为避免翻译服务商与语音服务商耦合，采用独立 TTS 配置模块。

## 目标
1. 新增独立 TTS 配置模块，支持单选启用（任意时刻仅一个启用）。
2. 预置小米 TTS（名称/默认模型/默认地址），用户自行填写 API Key/模型/地址/voice。
3. 新增后端语音合成命令，按 OpenAI 协议调用并返回可播放音频。
4. 主页面翻译结果卡片新增播放按钮，点击后合成并播放语音。
5. 提供基础失败兜底与状态反馈（请求中、播放中、失败提示）。

## 非目标
- 本期不实现流式音频边下边播。
- 本期不实现多服务商同时 TTS 混合路由。
- 不改动翻译主链路与现有翻译服务商配置结构。

## 设计方案

### A. 数据层
- 新增表 `speech_provider_configs`：
  - `provider_name TEXT PRIMARY KEY`
  - `enabled INTEGER NOT NULL DEFAULT 0`
  - `api_key TEXT NOT NULL DEFAULT ''`
  - `model TEXT NOT NULL DEFAULT ''`
  - `base_url TEXT`
  - `voice TEXT NOT NULL DEFAULT ''`
  - `audio_format TEXT NOT NULL DEFAULT 'wav'`
  - `updated_at DATETIME DEFAULT CURRENT_TIMESTAMP`
- 初始化预置行：`xiaomi`，默认：
  - `model = mimo-v2-tts`
  - `base_url = https://api.xiaomimimo.com/v1/chat/completions`
  - `voice = mimo_default`
  - `audio_format = wav`
- 建唯一索引保证单启用：
  - `idx_speech_provider_single_enabled` (`enabled`) `WHERE enabled = 1`

### B. 后端命令层
- 在 `commands/settings.rs` 新增：
  - `get_speech_provider_configs`
  - `save_speech_provider_config`
  - `test_speech_provider`
- 在 `commands/translation.rs` 新增：
  - `synthesize_speech`（输入文本，返回 base64 + format + provider/model）
- 小米 TTS 请求按 OpenAI 协议：
  - endpoint: `/v1/chat/completions`
  - body 带 `audio: { voice, format }`
  - `messages` 使用文档约定的角色与内容
  - 从 `choices[0].message.audio.data` 读取 base64

### C. 设置页
- 新增“语音合成（TTS）”配置区块：
  - 服务商切换（先仅小米，保留可扩展结构）
  - 启用开关（单选策略）
  - API Key / 模型 / Base URL / Voice / 音频格式（输入或枚举）
  - 保存配置、测试连接按钮
- 错误处理沿用现有错误码解析逻辑。

### D. 主页面播放交互
- 在每个翻译结果卡片头部加入播放按钮（仅在有翻译文本时可点）。
- 点击后：
  - 若已有播放任务则先停止当前播放；
  - 调用 `synthesize_speech(text)`；
  - 组装 `data:audio/{format};base64,...` 并用 `Audio` 播放。
- UI 状态：
  - 合成中：按钮显示 loading；
  - 播放中：按钮高亮，可再次点击停止。

## 验收标准
- 设置页可配置并保存小米 TTS 参数。
- 主页面点击播放可正常发起合成并播放音频。
- 未配置或配置错误时给出明确提示，不影响翻译主流程。
- `cargo check`、`cargo test`、`pnpm exec vue-tsc --noEmit` 均通过。
