# Ace Translator 0.2.0 重构计划

## 概述
将悬浮窗升级为唯一主窗口，实现多服务商并行翻译、流式输出、1对1翻译模式等功能。

---

## 开发任务清单（按执行顺序）

### Phase 1: 窗口架构重构 ✅

#### 1.1 修改 tauri.conf.json 窗口配置 ✅
- [x] 1.1.1 移除 main 窗口配置
- [x] 1.1.2 将 floating 窗口改名为 main，调整为主窗口
- [x] 1.1.3 设置 `decorations: false`（无系统标题栏）
- [x] 1.1.4 设置 `minHeight: 200`
- [x] 1.1.5 设置 `resizable: true`（允许调整大小）
- [x] 1.1.6 更新 settings 窗口配置
- [x] 1.1.7 将 logs 窗口改名为 history（历史/日志）

#### 1.2 修改后端托盘和窗口逻辑 ✅
- [x] 1.2.1 修改 `src-tauri/src/lib.rs` 托盘左键点击逻辑（屏幕居中显示）
- [x] 1.2.2 修改单实例处理逻辑，使用新的 main 窗口
- [x] 1.2.3 修改 `on_window_event` 中的窗口标签引用
- [x] 1.2.4 新增 `show_main_window_centered` 命令（屏幕居中显示）
- [x] 1.2.5 新增 `resize_main_window` 命令（动态调整窗口高度）

#### 1.3 实现自定义标题栏 ✅
- [x] 1.3.1 在 MainTranslator.vue 顶部添加自定义标题栏组件
- [x] 1.3.2 实现拖拽区域（使用 `data-tauri-drag-region`）
- [x] 1.3.3 添加固定按钮（Pin/Unpin）
- [x] 1.3.4 添加设置按钮（打开设置窗口）
- [x] 1.3.5 添加关闭按钮（隐藏窗口）
- [x] 1.3.6 调整标题栏样式（深色/浅色主题适配）

#### 1.4 实现窗口高度自适应 ✅
- [x] 1.4.1 在前端计算内容区域所需高度
- [x] 1.4.2 获取屏幕高度，计算最大高度（70%）
- [x] 1.4.3 调用后端命令调整窗口大小
- [x] 1.4.4 添加内容变化时的高度更新监听

#### 1.5 清理旧代码 ✅
- [x] 1.5.1 删除 `src/components/layout/MainLayout.vue`
- [x] 1.5.2 删除 `src/components/features/Translator.vue`
- [x] 1.5.3 修改 `src/main.ts` 路由配置
- [x] 1.5.4 清理相关的 import 和引用

#### 1.6 测试窗口功能（待用户测试）
- [ ] 1.6.1 测试托盘左键点击（屏幕居中）
- [ ] 1.6.2 测试双击复制（鼠标位置）
- [ ] 1.6.3 测试标题栏拖拽
- [ ] 1.6.4 测试固定/取消固定
- [ ] 1.6.5 测试窗口高度自适应

---

### Phase 2: 翻译模式重构（1对1） ✅

#### 2.1 修改语言配置 ✅
- [x] 2.1.1 在 `src-tauri/src/commands/settings.rs` 添加 `primary_target` 字段
- [x] 2.1.2 添加 `secondary_target` 字段
- [x] 2.1.3 移除 `common_target_languages` 字段
- [x] 2.1.4 修改 `src-tauri/src/commands/settings.rs` 相关命令

#### 2.2 修改翻译命令 ✅
- [x] 2.2.1 修改 `translate_text` 命令参数：`targetLangs: Vec<String>` → `primaryTarget/secondaryTarget`
- [x] 2.2.2 添加源语言检测后的目标语言切换逻辑（由 AI 自动判断）
- [x] 2.2.3 修改缓存查询逻辑（按单一目标语言）

#### 2.3 修改提示词 ✅
- [x] 2.3.1 简化 `src-tauri/src/config/prompts.rs`
- [x] 2.3.2 修改输出格式为单语言翻译结果

#### 2.4 修改前端 UI ✅
- [x] 2.4.1 修改设置页面语言配置（第一语言/第二语言选择）
- [x] 2.4.2 移除常用语言多选
- [x] 2.4.3 修改 MainTranslator 翻译请求逻辑

---

### Phase 3: 多服务商架构 ✅ (完成)

#### 3.1 创建 Provider 接口 ✅
- [x] 3.1.1 创建 `src-tauri/src/services/ai/mod.rs`
- [x] 3.1.2 创建 `src-tauri/src/services/ai/provider.rs`，定义 `AIProvider` trait
- [x] 3.1.3 定义 `AIError` 错误类型
- [x] 3.1.4 定义 `TranslationRequest` 和 `TranslationResponse` 结构

#### 3.2 实现智谱 AI Provider ✅
- [x] 3.2.1 创建 `src-tauri/src/services/ai/zhipu.rs`
- [x] 3.2.2 实现 `ZhipuProvider` 结构体
- [x] 3.2.3 实现 `translate` 方法
- [x] 3.2.4 实现流式 `translate_stream` 方法
- [x] 3.2.5 添加 glm-4-flash 等模型支持
- [x] 3.2.6 删除旧的 `src-tauri/src/services/zhipu.rs`

#### 3.3 实现 OpenAI Provider ✅
- [x] 3.3.1 创建 `src-tauri/src/services/ai/openai.rs`
- [x] 3.3.2 实现 `OpenAIProvider` 结构体
- [x] 3.3.3 实现 `translate` 方法
- [x] 3.3.4 实现流式 `translate_stream` 方法
- [x] 3.3.5 支持自定义 Base URL

#### 3.4 实现 Claude Provider ✅
- [x] 3.4.1 创建 `src-tauri/src/services/ai/claude.rs`
- [x] 3.4.2 实现 `ClaudeProvider` 结构体
- [x] 3.4.3 实现 `translate` 方法
- [x] 3.4.4 实现流式 `translate_stream` 方法

#### 3.5 实现 Ollama Provider ✅
- [x] 3.5.1 创建 `src-tauri/src/services/ai/ollama.rs`
- [x] 3.5.2 实现 `OllamaProvider` 结构体
- [x] 3.5.3 实现 `translate` 方法
- [x] 3.5.4 实现流式 `translate_stream` 方法
- [x] 3.5.5 支持自定义 API 地址

#### 3.6 实现并行翻译 ✅
- [x] 3.6.1 修改 `src-tauri/src/commands/translation.rs`
- [x] 3.6.2 实现获取已启用服务商列表
- [x] 3.6.3 实现并行调用多服务商（`futures::join_all`）
- [x] 3.6.4 实现流式事件发送（通过 Tauri 事件）

#### 3.7 实现流式输出功能 ✅
- [x] 3.7.1 定义 `StreamEvent` 枚举类型（Start/Chunk/Done/Error）
- [x] 3.7.2 在 `AIProvider` trait 中添加 `translate_stream` 方法
- [x] 3.7.3 实现智谱 AI 的流式 API 调用
- [x] 3.7.4 创建 `translate_multi_stream` 命令
- [x] 3.7.5 前端创建 `useStreamingTranslation` composable
- [x] 3.7.6 更新 `MainTranslator.vue` 支持流式渲染
- [x] 3.7.7 添加流式输出光标指示器

#### 3.8 更新 services/mod.rs ✅
- [x] 3.8.1 添加 `pub mod ai;`
- [x] 3.8.2 移除旧的 `pub mod zhipu;`

---

### Phase 4: 设置窗口重构 ✅

#### 4.1 服务商配置数据结构 ✅
- [x] 4.1.1 定义 `ProviderConfig` 结构体（含 provider_name 字段）
- [x] 4.1.2 定义各服务商的可选模型列表
- [x] 4.1.3 创建 provider_configs 数据库表存储服务商配置
- [x] 4.1.4 添加 `get_provider_configs` 命令
- [x] 4.1.5 添加 `save_provider_config` 命令

#### 4.2 实现 API 测试功能 ✅
- [x] 4.2.1 定义 `ApiTestResponse` 结构体（返回原始响应元数据）
- [x] 4.2.2 在各 Provider 中实现 `test_api` 方法
- [x] 4.2.3 添加 `test_provider` 命令（独立于翻译逻辑，不使用缓存）

#### 4.3 重构设置页面 UI ✅
- [x] 4.3.1 添加服务商 Tab 栏（智谱/OpenAI/Claude/Ollama）
- [x] 4.3.2 实现智谱 AI 配置面板（启用开关、模型选择、API Key）
- [x] 4.3.3 实现 OpenAI 配置面板（启用开关、模型选择、API Key、Base URL）
- [x] 4.3.4 实现 Claude 配置面板（启用开关、模型选择、API Key）
- [x] 4.3.5 实现 Ollama 配置面板（启用开关、模型选择、API 地址）
- [x] 4.3.6 添加测试连接按钮，显示原始响应元数据
- [x] 4.3.7 创建 Switch UI 组件

#### 4.4 清理旧配置 ✅
- [x] 4.4.1 移除原有的单一 API Key 输入框
- [x] 4.4.2 移除缓存管理部分（迁移到历史窗口）

---

### Phase 5: 前端翻译流程重构 ✅

#### 5.1 修改 MainTranslator 组件 ✅
- [x] 5.1.1 移除多语言结果渲染
- [x] 5.1.2 添加多服务商结果渲染区域
- [x] 5.1.3 实现流式渲染（逐字符显示）
- [x] 5.1.4 添加各服务商的加载状态指示

#### 5.2 监听翻译事件 ✅
- [x] 5.2.1 监听 `translation-start` 事件
- [x] 5.2.2 监听 `translation-chunk` 事件（流式内容）
- [x] 5.2.3 监听 `translation-done` 事件
- [x] 5.2.4 监听 `translation-error` 事件

#### 5.3 翻译结果交互 ✅
- [x] 5.3.1 每个服务商结果添加复制按钮
- [x] 5.3.2 显示服务商名称和模型信息
- [x] 5.3.3 错误状态显示

---

### Phase 6: 历史与日志窗口 ✅

#### 6.1 修改数据库表结构 ✅
- [x] 6.1.1 添加 `provider` 字段到 translation_history 表
- [x] 6.1.2 添加 `model` 字段
- [x] 6.1.3 编写数据迁移逻辑

#### 6.2 创建 HistoryAndLogs 组件 ✅
- [x] 6.2.1 重命名 LogViewer.vue 为 HistoryAndLogs.vue
- [x] 6.2.2 实现页面内 tab 栏（历史记录/缓存管理/应用日志）
- [x] 6.2.3 实现 tab 切换逻辑

#### 6.3 实现历史记录 Tab ✅
- [x] 6.3.1 添加 `get_translation_history` 命令
- [x] 6.3.2 实现历史记录列表（原文预览、服务商、目标语言、时间）
- [x] 6.3.3 实现点击展开查看完整翻译
- [x] 6.3.4 添加删除单条历史功能

#### 6.4 实现缓存管理 Tab ✅
- [x] 6.4.1 显示缓存条数和占用空间
- [x] 6.4.2 添加清空缓存按钮

#### 6.5 保留应用日志 Tab ✅
- [x] 6.5.1 迁移原 LogViewer 功能到日志 Tab

#### 6.6 更新路由 ✅
- [x] 6.6.1 修改 `src/main.ts` 路由配置
- [x] 6.6.2 修改 tauri.conf.json 中的窗口 URL

---

### Phase 7: 快捷键系统 ✅

#### 7.1 新增 Alt+Space 快捷键 ✅
- [x] 7.1.1 在 `src-tauri/src/services/hotkey/mod.rs` 添加 Alt+Space 监听
- [x] 7.1.2 实现屏幕居中显示逻辑
- [x] 7.1.3 macOS 实现（监听 Option+Space）
- [x] 7.1.4 Windows 实现（监听 Alt+Space）

#### 7.2 快捷键配置存储 ✅
- [x] 7.2.1 在 settings 表添加快捷键配置字段
- [x] 7.2.2 添加 `get_hotkey_config` 命令
- [x] 7.2.3 添加 `save_hotkey_config` 命令
- [x] 7.2.4 在 AppState 添加快捷键启用状态

#### 7.3 快捷键配置 UI ✅
- [x] 7.3.1 在设置页面添加快捷键配置区域
- [x] 7.3.2 添加双击复制启用/禁用开关
- [x] 7.3.3 添加 Alt+Space 启用/禁用开关

#### 7.4 动态应用快捷键 ✅
- [x] 7.4.1 热键处理函数检查配置状态
- [x] 7.4.2 保存配置时更新 AppState

---

### Phase 8: 性能优化 ✅

#### 8.1 网络超时处理 ✅
- [x] 8.1.1 为所有 Provider 的 HTTP Client 设置 30 秒超时（已实现）
- [x] 8.1.2 添加连接超时配置（Ollama 使用 60 秒以适应本地模型）

#### 8.2 请求取消优化 ✅
- [x] 8.2.1 在 AppState 添加请求取消 token 管理
- [x] 8.2.2 修改翻译命令支持取消
- [x] 8.2.3 前端取消时发送取消命令
- [x] 8.2.4 实现 `futures::Abortable` 取消逻辑

#### 8.3 测试性能优化
- [ ] 8.3.1 测试请求超时
- [ ] 8.3.2 测试请求取消
- [ ] 8.3.3 测试并行请求性能

---

### Phase 9: 界面国际化 ✅

#### 9.1 安装和配置 vue-i18n ✅
- [x] 9.1.1 安装 vue-i18n 依赖
- [x] 9.1.2 创建 `src/locales/index.ts` 配置
- [x] 9.1.3 在 main.ts 中注册 i18n

#### 9.2 创建语言包 ✅
- [x] 9.2.1 创建 `src/locales/zh-CN.json` 中文语言包
- [x] 9.2.2 创建 `src/locales/en.json` 英文语言包

#### 9.3 替换硬编码文本 ✅
- [x] 9.3.1 替换 MainTranslator.vue 中的文本
- [x] 9.3.2 替换 Settings.vue 中的文本（完整国际化）
- [x] 9.3.3 替换 HistoryAndLogs.vue 中的文本（完整国际化）

#### 9.4 语言切换功能 ✅
- [x] 9.4.1 在设置页面添加语言切换下拉框
- [x] 9.4.2 保存语言设置到 localStorage
- [x] 9.4.3 应用启动时加载语言设置

---

### Phase 10: 最终测试与文档 ✅

#### 10.1 功能测试（待用户测试）
- [ ] 10.1.1 测试所有服务商翻译
- [ ] 10.1.2 测试流式输出（待后续实现）
- [ ] 10.1.3 测试缓存功能
- [ ] 10.1.4 测试历史记录
- [ ] 10.1.5 测试快捷键

#### 10.2 更新文档 ✅
- [x] 10.2.1 更新 README.md 版本说明
- [x] 10.2.2 更新 CLAUDE.md
- [x] 10.2.3 更新版本号到 0.2.0

---

## 关键文件清单

**需要修改：**
- `src-tauri/tauri.conf.json`
- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/hotkey/mod.rs`
- `src-tauri/src/services/database.rs`
- `src-tauri/src/commands/translation.rs`
- `src-tauri/src/commands/settings.rs`
- `src-tauri/src/commands/system.rs`
- `src-tauri/src/config/prompts.rs`
- `src-tauri/src/models/settings.rs`
- `src/main.ts`
- `src/components/features/FloatingTranslator.vue`
- `src/components/features/Settings.vue`
- `src/components/features/LogViewer.vue`
- `src/stores/settings.ts`

**需要新建：**
- `src-tauri/src/services/ai/mod.rs`
- `src-tauri/src/services/ai/provider.rs`
- `src-tauri/src/services/ai/zhipu.rs`
- `src-tauri/src/services/ai/openai.rs`
- `src-tauri/src/services/ai/claude.rs`
- `src-tauri/src/services/ai/ollama.rs`
- `src/locales/zh-CN.json`
- `src/locales/en.json`
- `src/locales/index.ts`

**需要删除：**
- `src/components/layout/MainLayout.vue`
- `src/components/features/Translator.vue`
- `src-tauri/src/services/zhipu.rs`（移动到 ai/ 目录）
