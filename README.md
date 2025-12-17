# Ace Translator

Ace AI 翻译器 - 一款基于 AI 的桌面翻译应用，支持多语言翻译、双击复制快速翻译等功能。

# 技术栈

## 前端
- **Vue 3** - 渐进式 JavaScript 框架，使用 Composition API
- **TypeScript** - 类型安全的 JavaScript 超集
- **Vite** - 下一代前端构建工具
- **Pinia** - Vue 官方状态管理库
- **Vue Router** - Vue 官方路由管理
- **Tailwind CSS** - 原子化 CSS 框架
- **Radix Vue** - 无样式 UI 组件库
- **Lucide Vue** - 图标库

## 后端 (Tauri/Rust)
- **Tauri 2** - 轻量级跨平台桌面应用框架
- **Tokio** - Rust 异步运行时
- **SQLx** - 异步 SQL 工具库（SQLite）
- **Reqwest** - HTTP 客户端
- **Ring** - 加密库（用于 API Key 加密存储）

## AI 翻译
- **智谱 GLM-4-Flash** - 默认且强制启用的模型（负责语言检测+开箱即用）

## 平台特性
- **rdev** - 跨平台键盘事件监听（Windows/Linux）
- **Core Graphics** - macOS 键盘事件监听
- **mouse_position** - 获取鼠标位置用于悬浮窗定位

# 构建指南

macos
```shell
export TAURI_SIGNING_PRIVATE_KEY="dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5YkRpUTE2SE5EMXdLZ01ZbmFKdkpuQnBIYlQ1MGhETmRHRzRhVkhFNGJPUUFBQkFBQUFBQUFBQUFBQUlBQUFBQURhaklQelhMeTVBdGVTRzFYOWVyeE5hRkJrdGMySmJWTXl0OUl2RFB1Tzhjdk5rVUtsOTVPRmlDSjdldjd3L3pFTEYyRGNJblZjbFRvWmErcFBjMUhPYVpIRElrenFEaU4xOUIremRIRFFVQTFpUU56eUdKbDA4eEFSMEw4ak5TdklmRHNQYnEwd289Cg=="
# optionally also add a password
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="Caiyuan0109..."
```

windows
```shell
$env:TAURI_SIGNING_PRIVATE_KEY="dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5YkRpUTE2SE5EMXdLZ01ZbmFKdkpuQnBIYlQ1MGhETmRHRzRhVkhFNGJPUUFBQkFBQUFBQUFBQUFBQUlBQUFBQURhaklQelhMeTVBdGVTRzFYOWVyeE5hRkJrdGMySmJWTXl0OUl2RFB1Tzhjdk5rVUtsOTVPRmlDSjdldjd3L3pFTEYyRGNJblZjbFRvWmErcFBjMUhPYVpIRElrenFEaU4xOUIremRIRFFVQTFpUU56eUdKbDA4eEFSMEw4ak5TdklmRHNQYnEwd289Cg=="
<# optionally also add a password #>
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD="Caiyuan0109..."
```

## Windows 打包
1. 安装依赖（锁定版本）：`pnpm install --frozen-lockfile`
2. 构建 x64 安装包：`pnpm tauri build --target x86_64-pc-windows-msvc`
   - 产物位置：`src-tauri/target/x86_64-pc-windows-msvc/release/`（NSIS 在 `bundle/nsis/*.exe`，单文件 exe 同级）
   - 本地 Windows 无需 `--runner cargo-xwin`。

## macOS 打包
1. 安装依赖：`pnpm install --frozen-lockfile`
2. 添加目标（如未添加）：`rustup target add x86_64-apple-darwin aarch64-apple-darwin`
3. 构建：
   - 通用双架构：`pnpm tauri build --target universal-apple-darwin`
   - 或单架构：`pnpm tauri build --target x86_64-apple-darwin` / `pnpm tauri build --target aarch64-apple-darwin`
   - 产物位置：`src-tauri/target/<target>/release/bundle/dmg/`

## macOS 安装说明

应用未签名，首次启动可能被 Gatekeeper 拦截：

1. **首次打开被拦截**：右键点击应用选择“打开”，在弹窗中再次点击“打开”。
2. **或**：系统设置 → 隐私与安全性 → 找到被拦截的应用 → 点击“仍要打开”。
3. **辅助功能权限**：用于监听全局快捷键（双击 Cmd+C 触发浮窗）。首次使用会弹权限提示；或手动前往 系统设置 → 隐私与安全性 → 辅助功能，添加并勾选 Ace Translator。

# 版本

# 默认策略（开箱即用保障）
- **智谱强制启用**：内置/远程 API Key 自动注入，用户无需配置即可使用。
- **语言检测只用智谱**：所有检测统一走智谱，确保速度和一致性。
- **默认模型固定为 `glm-4-flash`**：即便未配置模型也会回落到该模型。
- **API Key 拉取优先级**：先请求 `https://nest.wangcaiyuan.com/ai/translator/api-key`（code=00000 且 data.apiKey 有值），失败则使用内置 Key 兜底。
- **流式翻译取消不可真正中断**：前端不提供取消按钮，使用超时关闭 loading 并提示超时；后端会忽略已标记取消请求的事件，避免报错。

## 0.1.0 - 初始版本
- 主窗口：多语言翻译，自动检测源语言，最多 5 个目标语言（常用列表+当前选择），命中缓存直接返回，源/目标相同时自动切换备用目标。
- 浮窗：双击复制（macOS: Cmd+C 连按；Windows: Ctrl+C 连按）自动弹出并跟随鼠标，支持取消翻译、结果复制、固定/失焦自动隐藏。
- 翻译后端：接入智谱 GLM-4 系列（默认 glm-4-flash），多目标批量翻译，统一的请求/结果处理链路。
- 设置：API Key 加密存储；默认/常用目标语言；主题切换（默认深色）；设置变更事件实时同步多窗口。
- 缓存：SQLite 翻译历史缓存，提供数量/占用统计与一键清空。
- 日志：内置日志收集与独立日志窗口，可从托盘打开。
- 托盘与窗口：托盘菜单（显示主窗/设置/日志/退出）；关闭窗口仅隐藏；单实例启动。
- 快捷键与权限：监听双击复制；macOS 可检测并提示辅助功能权限。

## 0.2.0 - 窗口架构重构与多服务商支持

### 核心重构：单窗口架构
- 移除原主窗口，将悬浮窗升级为唯一主窗口
- 自定义标题栏：无系统装饰，包含拖拽区域、固定按钮、设置按钮、关闭按钮
- 窗口高度自适应：最小高度固定，内容区域自动伸缩，最大高度不超过屏幕 70%
- 双触发模式：
  - 双击 Cmd+C（macOS）/Ctrl+C（Windows）：鼠标位置附近弹出
  - Alt/Option+空格：屏幕居中弹出
  - 托盘左键点击：屏幕居中弹出

### 翻译模式重构
- 1对1 翻译模式：取消多语言同时翻译，改为主语言/备用语言自动切换
- 源语言与目标语言相同时自动切换到备用语言

### 多服务商并行翻译
- 支持多个 AI 服务商同时翻译：智谱 AI、OpenAI、Claude、Ollama
- 流式输出：实时显示翻译结果
- 各服务商结果独立渲染，支持单独复制
- 翻译结果按服务商分别缓存

### 设置页面重构
- 服务商 Tab 布局：每个服务商独立配置面板
- 智谱 AI：默认启用 glm-4-flash 免费模型（无需 API Key）
- OpenAI：支持自定义 Base URL
- Claude：API Key 配置
- Ollama：支持自定义 API 地址和模型
- 测试连接功能

### 快捷键系统
- 新增打开窗口快捷键：Alt/Option+空格
- 快捷键可自定义配置
- 双击复制功能可启用/禁用
- 一键恢复默认快捷键

### 历史与日志窗口
- 三合一窗口：历史记录 + 缓存管理 + 应用日志
- 页面级 Tab 栏切换
- 历史记录：查看翻译历史（原文预览、服务商、目标语言、时间）
- 缓存管理：按服务商统计和清空

### 性能优化
- 网络超时处理：API 请求 30 秒超时
- 请求取消处理：前端可超时停止 loading，后端请求自然结束（流式任务不会被强制中断）
- 数据库索引优化

### 国际化
- 界面多语言支持：中文/英文切换（vue-i18n）
