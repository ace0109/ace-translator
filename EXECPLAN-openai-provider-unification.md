# ExecPlan: OpenAI 协议统一与服务商体系升级

## 背景
- 现状仅稳定支持单一服务商路径（智谱），并且前后端存在硬编码过滤。
- 目标是统一到 OpenAI 协议优先的服务商接入模式，支持预置服务商与用户自定义服务商。

## 目标
1. 预置多个服务商（名称、模型、请求地址），API Key 由用户自行填写和保存。
2. 服务商只要兼容 OpenAI 协议即可通过配置使用。
3. 支持新增/删除自定义服务商。
4. 翻译主链路不再写死智谱，支持多服务商并行。
5. 新增小米 MiMo、MiniMax、Moonshot 预置服务商。
6. 优化 OpenAI 协议路径下的 prompt 与响应解析稳定性。
7. 产品层统一为“单服务商启用”模式（只保留一个活动服务商）。
8. 设置页模型字段改为纯输入，不再提供预置模型单选按钮。
9. 模型测速支持弹窗编辑多个模型并做缓存，下次默认回填（可增删改）。
10. 翻译链路提供统一错误码和基础可观测日志（服务商/模型/耗时/状态）。
11. 清理遗留 `multi` 语义命名与死代码，并补充关键回归测试。

## 非目标
- 不改动主 UI 布局结构。
- 不移除历史兼容 provider（如已有 claude 数据行）。

## 设计与实现

### A. 后端 provider 元数据与预置
- 在 `src-tauri/src/config/providers.rs` 增加 `ProviderPreset` 与预置列表。
- 预置：`zhipu/openai/deepseek/xiaomi/minimax/moonshot/ollama`。
- 小米默认地址：`https://api.xiaomimimo.com/v1/chat/completions`。
- MiniMax 默认地址：`https://api.minimax.io/v1/chat/completions`。
- Moonshot 默认地址：`https://api.moonshot.ai/v1/chat/completions`。

### B. 数据初始化与升级
- `src-tauri/src/services/database.rs` 改为按预置列表初始化 `provider_configs`。
- 初始化时写入默认 `model/base_url`；升级时仅补齐空值，不覆盖用户配置。

### C. 设置命令层
- `get_provider_configs` 改为读取全部 provider，不再只读 `zhipu`。
- `save_provider_config` 去除“智谱强制启用”。
- `test_provider` 采用通用分发：`zhipu/ollama/claude(兼容)/openai-compatible(默认)`。
- 新增命令：
  - `create_custom_provider`
  - `delete_custom_provider`

### D. 翻译命令层
- `get_enabled_providers` 改为读取全部启用 provider。
- 语言检测 provider 自动选择（优先可用 provider），不再强制智谱。
- 未知 provider 默认走 OpenAI 协议实现。
- 流式与非流式分发逻辑保持一致。

### E. 前端设置页与主翻译页
- 设置页移除 `zhipuOnly` 过滤，展示所有 provider。
- 增加“新增自定义服务商”对话框与“删除自定义服务商”操作。
- 模型字段改为纯可编辑输入，不再提供预置模型快捷按钮。
- 主翻译页移除 `zhipuOnly`，按启用 provider 并行翻译。

### F. Prompt 与响应优化（OpenAI 协议）
- 优化系统提示词与用户提示词，强化“仅输出翻译文本”和占位符保留约束。
- 语言检测提示词改为受限候选语言代码，减少脏输出。
- 非流式响应解析支持更宽容的 `message.content` 结构。
- 流式 SSE 解析改为事件级缓冲，兼容分片边界，降低 chunk 丢失概率。

### G. 单服务商架构收敛
- 设置层与数据库层保证同一时刻仅一个 `enabled=1`。
- 翻译命令层统一按“当前唯一启用服务商”执行，移除并行分发依赖。
- 主页面只展示当前启用服务商对应结果，不再提供多服务商并行选择。

### H. 设置页模型输入与测速弹窗
- 设置页模型仅保留输入框，移除预置模型快捷按钮，用户自由填写模型 ID。
- “模型测速”改为弹窗配置模型列表：
  - 支持新增/删除/编辑多模型。
  - 默认将当前配置模型置于第一项（若存在）。
  - 以 `provider` 维度缓存上次测速模型列表，并在下次打开时回填。

### I. 错误码与可观测性
- 后端命令返回统一错误码格式，前端按错误码展示更明确提示。
- 翻译入口/出口记录关键日志：`request_id/provider/model/mode/status/duration_ms`。

### J. 兼容层与测试
- 主流程切换到单服务商命名命令；保留旧命令作为薄兼容包装。
- 删除或封存不再使用的并行辅助函数。
- 回归测试覆盖：
  - 启用配置校验（模型/base_url/api_key）。
  - 单选启用 SQL 行为（启用 A 时自动禁用 B）。
  - 错误码格式化/关键 helper 行为。

## 验收标准
- `cargo check` 通过。
- `pnpm exec vue-tsc --noEmit` 通过。
- `cargo test` 通过（新增关键回归测试）。
- 设置页可看到预置 provider（含小米、MiniMax、Moonshot）。
- 可新增/删除自定义 provider。
- 任意时刻最多仅一个启用服务商，翻译链路按该服务商执行并流式返回。
- 设置页模型测速支持弹窗多模型编辑与缓存回填。
