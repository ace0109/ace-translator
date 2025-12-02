# Ace Translator AI翻译软件开发计划

## 项目概述

基于Tauri 2.0 + Vue 3 + TypeScript的桌面AI翻译应用，集成智谱AI GLM-4.6模型，支持快捷键触发悬浮翻译和主窗口专业翻译功能。

## 技术架构

### 前端技术栈
- **Vue 3** + Composition API + TypeScript
- **Vite** 作为构建工具
- **Pinia** 用于状态管理
- **Naive UI** 作为UI组件库（卡片式设计风格）
- **@vueuse/core** 用于剪贴板和快捷键监听

### 后端技术栈
- **Rust** + Tauri 2.0
- **SQLite** 用于本地数据存储
- **reqwest** 用于HTTP请求
- **tauri-plugin-tray** 系统托盘支持
- **tauri-plugin-global-shortcut** 全局快捷键
- **ring** 用于API密钥加密存储

### 项目结构
```
ace-translator/
├── src/                          # Vue.js前端
│   ├── components/                 # Vue组件
│   │   ├── features/              # 功能组件
│   │   │   ├── Translator.vue     # 主翻译界面
│   │   │   ├── FloatingTranslator.vue # 悬浮翻译窗口
│   │   │   └── Settings.vue      # 设置界面
│   │   ├── common/               # 通用组件
│   │   │   ├── LanguageSelector.vue
│   │   │   ├── TrayMenu.vue
│   │   │   └── LoadingSpinner.vue
│   │   └── layout/              # 布局组件
│   │       ├── MainLayout.vue
│   │       └── FloatingLayout.vue
│   ├── services/                 # 服务层
│   │   ├── zhipu.ts            # 智谱AI接口服务
│   │   ├── storage.ts           # 本地存储服务
│   │   └── shortcuts.ts         # 快捷键服务
│   ├── stores/                   # Pinia状态管理
│   │   ├── translation.ts        # 翻译状态
│   │   ├── settings.ts           # 应用设置
│   │   └── history.ts           # 翻译历史
│   ├── types/                    # TypeScript类型定义
│   │   ├── translation.ts
│   │   └── settings.ts
│   ├── utils/                    # 工具函数
│   │   ├── encryption.ts         # 加密工具
│   │   ├── validation.ts         # 输入验证
│   │   └── constants.ts         # 常量定义
│   ├── assets/                   # 静态资源
│   ├── App.vue                   # 根组件
│   └── main.ts                   # 应用入口
├── src-tauri/                   # Rust后端
│   ├── src/
│   │   ├── commands/             # Tauri命令
│   │   │   ├── translation.rs   # 翻译相关命令
│   │   │   ├── settings.rs      # 设置相关命令
│   │   │   └── system.rs       # 系统相关命令
│   │   ├── services/             # 后端服务
│   │   │   ├── zhipu.rs       # 智谱AI服务
│   │   │   ├── encryption.rs   # 加密服务
│   │   │   ├── database.rs     # 数据库服务
│   │   │   └── clipboard.rs   # 剪贴板服务
│   │   ├── models/              # 数据模型
│   │   │   ├── translation.rs
│   │   │   └── settings.rs
│   │   ├── utils/               # 工具模块
│   │   │   ├── error.rs        # 错误处理
│   │   │   └── config.rs       # 配置管理
│   │   ├── main.rs              # 应用入口
│   │   └── lib.rs              # 库入口
│   ├── Cargo.toml               # Rust依赖
│   └── tauri.conf.json         # Tauri配置
├── package.json                 # 前端依赖
├── vite.config.ts              # Vite配置
└── tsconfig.json               # TypeScript配置
```

## 核心功能设计

### 1. 主翻译窗口 (Main Translator Window)
- **布局**: 左右分栏，源语言和目标语言
- **功能**:
  - 语言选择器（20+语言支持）
  - 文本输入框（支持拖拽、粘贴）
  - 翻译结果显示区域
  - 语言调换按钮
  - 复制按钮、历史记录
  - 收藏功能
- **尺寸**: 1000x700（可调整）

### 2. 悬浮翻译窗口 (Floating Translator)
- **触发**: 配置一个全局快捷键 (例如：`Alt+T` 或 `Ctrl+Alt+T`)。当此快捷键被按下时，Rust 后端将执行以下步骤：
  1. 模拟 `Ctrl+C` (或 `Cmd+C`) 操作，将当前选中的文本复制到系统剪贴板。
  2. 读取剪贴板内容以获取待翻译文本。
  3. 显示悬浮翻译窗口。
  *(注意：由于 Tauri 前端无法直接获取系统其他应用的选中文本或光标位置，此机制确保了跨应用的兼容性。)*
- **显示**:
  - 小型悬浮窗口，显示在鼠标光标附近 (或屏幕指定位置，如屏幕中央或边缘)。
  - 简洁设计，只显示翻译结果。
  - 点击外部区域自动关闭。
  - 支持拖拽移动。
  - **性能**: 窗口创建应在应用启动时完成并默认隐藏，触发时仅需显示和调整位置，响应时间 < 500ms。


### 3. 系统托盘 (System Tray)
- **图标**: Windows/Linux/ macOS适配
- **右键菜单**:
  - 打开翻译器
  - 快速翻译
  - 设置
  - 退出
- **行为**: 关闭窗口时最小化到托盘

### 4. 智谱AI集成
- **API**: GLM-4.6模型
- **接口**: POST https://open.bigmodel.cn/api/paas/v4/chat/completions
- **认证**: API Key加密存储
- **缓存**: 本地翻译结果缓存（7天）
- **限流**: 智能请求频率控制

## 实施计划

### Phase 1: 基础架构搭建 (1-2天)
**目标**: 完成项目基础结构和依赖配置

#### 1.1 依赖管理
- 更新 `Cargo.toml`:
  ```toml
  [dependencies]
  tauri = { version = "2", features = ["tray", "global-shortcut"] }
  tauri-plugin-tray = "2"
  tauri-plugin-global-shortcut = "2"
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  tokio = { version = "1", features = ["full"] }
  reqwest = { version = "0.11", features = ["json"] }
  sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "bundled"] }
  ring = "0.16"
  ```

- 更新 `package.json`:
  ```json
  {
    "dependencies": {
      "vue": "^3.5.13",
      "@tauri-apps/api": "^2",
      "pinia": "^2.1.7",
      "naive-ui": "^2.38.1",
      "@vueuse/core": "^10.7.2",
      "dayjs": "^1.11.10",
      "@vicons/ionicons5": "^0.12.0"
    }
  }
  ```

#### 1.2 Tauri配置更新
- **更新 `tauri.conf.json`**:
  - 添加多窗口配置 (主窗口、悬浮窗口、设置窗口)。**悬浮窗口 (floating window)** 建议默认 `visible: false`, `skipTaskbar: true`, `alwaysOnTop: true`, `decorations: false`。
  - **示例多窗口配置**:
    ```json
    "windows": [
      {
        "title": "ace-translator",
        "width": 1000,
        "height": 700,
        "resizable": true,
        "minimizable": true,
        "fullscreen": false
      },
      {
        "label": "floating",
        "title": "Floating Translator",
        "width": 400,
        "height": 150,
        "decorations": false,
        "transparent": true,
        "resizable": false,
        "skipTaskbar": true,
        "alwaysOnTop": true,
        "visible": false,
        "minimizable": false,
        "maximizable": false,
        "url": "index.html#/floating"
      },
      {
        "label": "settings",
        "title": "Settings",
        "width": 800,
        "height": 600,
        "resizable": false,
        "minimizable": false,
        "fullscreen": false,
        "visible": false,
        "url": "index.html#/settings"
      }
    ],
    ```

- **配置 `src-tauri/capabilities/default.json`**:
  - 在 Tauri 2.0 中，权限通过能力文件 (`.json`) 精细管理。需要确保以下权限被显式声明：
    - `window:allow-all` (或更细粒度的 `window:allow-hide`, `window:allow-show`, `window:allow-set-position`, `window:allow-set-size`)
    - `global-shortcut:allow-register`
    - `tray:allow-all` (或更细粒度的 `tray:allow-set-tooltip`, `tray:allow-set-menu`, `tray:allow-on-menu-event`)
    - `clipboard:allow-all` (或更细粒度的 `clipboard:allow-read-text`, `clipboard:allow-write-text`)
    - `app:allow-show`, `app:allow-hide`
    - `shell:allow-open` (如果需要打开外部链接)
    - `dialog:allow-all` (如果需要文件选择等对话框)
    - `path:allow-all` (如果需要访问应用数据目录存储 SQLite 数据库)
  - **示例 `default.json` 权限配置片段**:
    ```json
    {
      "identifier": "main-capability",
      "windows": [ "main", "floating", "settings" ],
      "permissions": [
        "window:all",
        "app:all",
        "global-shortcut:allow-register",
        "tray:all",
        "clipboard:all",
        "shell:allow-open",
        "dialog:all",
        "path:all"
      ]
    }
    ```

#### 1.3 基础文件结构
- 创建目录结构
- 设置基础路由
- 配置Pinia stores
- 设置TypeScript类型定义

### Phase 2: 后端核心功能 (2-3天)
**目标**: 实现翻译服务和系统功能

#### 2.1 数据库设计
```sql
-- 翻译历史表
CREATE TABLE translation_history (
    id INTEGER PRIMARY KEY,
    source_text TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    source_lang TEXT NOT NULL,
    target_lang TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 设置表
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

#### 2.2 核心服务实现
- `src-tauri/src/services/zhipu.rs`: 智谱AI API调用
- `src-tauri/src/services/encryption.rs`: API密钥加密存储
- `src-tauri/src/services/database.rs`: SQLite数据操作
- `src-tauri/src/services/clipboard.rs`: 剪贴板监听

#### 2.3 Tauri命令实现
- 翻译命令: `translate_text(text, source_lang, target_lang)`
- 设置命令: `save_settings(settings)`, `get_settings()`
- 历史命令: `get_history()`, `clear_history()`
- 系统命令: `show_floating_window()`, `hide_window()`

### Phase 3: 前端UI开发 (3-4天)
**目标**: 实现完整的用户界面

#### 3.1 主翻译界面
- `src/components/features/Translator.vue`:
  - 左右分栏布局
  - 语言选择器
  - 文本输入输出区域
  - 功能按钮组

#### 3.2 悬浮翻译界面
- `src/components/features/FloatingTranslator.vue`:
  - 简洁悬浮设计
  - 位置自动计算
  - 快速显示/隐藏

#### 3.3 设置界面
- `src/components/features/Settings.vue`:
  - API密钥配置
  - 语言偏好设置
  - 快捷键自定义
  - 主题选择

#### 3.4 通用组件
- `src/components/common/LanguageSelector.vue`: 语言选择下拉
- `src/components/common/TrayMenu.vue`: 托盘菜单

### Phase 4: 系统集成 (2-3天)
**目标**: 实现系统集成功能

#### 4.1 全局快捷键
- 注册自定义全局快捷键 (例如：`Alt+T`)。
- 当快捷键触发时，后端模拟系统 `Ctrl+C` (或 `Cmd+C`) 操作，将选中文本复制到剪贴板。
- 从剪贴板读取文本，并触发悬浮窗口显示和翻译。

#### 4.2 系统托盘
- 托盘图标显示
- 右键菜单功能
- 后台运行管理

#### 4.3 窗口管理
- 多窗口协调 (主窗口、悬浮窗口、设置窗口)
- 窗口状态同步
- 最小化/恢复逻辑 (特别是当主窗口关闭时最小化到托盘)

### Phase 5: 优化和测试 (1-2天)
**目标**: 性能优化和功能测试

#### 5.1 性能优化
- API请求去重
- 翻译缓存机制
- 内存使用优化
- 启动时间优化
- **Zhipu AI 流式响应**: 考虑在 Zhipu AI API 调用中启用流式传输 (`stream: true`)，以提供更平滑的用户体验，让翻译结果逐步显示。

#### 5.2 用户体验优化
- 加载状态指示
- 错误处理完善
- 动画效果添加
- 响应式设计适配

#### 5.3 测试和调试
- 功能测试
- 性能测试
- 跨平台兼容性测试
- 错误场景处理

## 关键技术实现要点

### 1. API安全存储
```rust
// 使用ring库进行AES加密
use ring::aead::{AES_256_GCM, LessSafeKey, OpeningKey, SealingKey};
use ring::rand::{SecureRandom, SystemRandom};

pub fn encrypt_api_key(api_key: &str) -> Result<String, Error> {
    // 加密实现
}

pub fn decrypt_api_key(encrypted: &str) -> Result<String, Error> {
    // 解密实现
}
```

### 2. 快捷键检测
```rust
// 使用 tauri-plugin-global-shortcut 注册全局快捷键
#[tauri::command]
async fn register_global_shortcut(handle: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    // 注册全局快捷键，例如 "Alt+T"
    // 当快捷键被按下时，触发 Rust 后端逻辑：
    // 1. 模拟 Ctrl+C 将选中文本复制到剪贴板。
    // 2. 读取剪贴板内容。
    // 3. 调用 Tauri API 显示悬浮窗口并传递文本。
    // 注意：模拟按键操作可能需要特定的权限和平台适配。
    // 也可以考虑只监听快捷键，然后前端调用 `invoke("read_clipboard")` 获取内容。
    Ok(())
}
```

### 3. 悬浮窗口定位
```rust
// 通过 Tauri 后端获取鼠标当前位置并计算悬浮窗口位置
use tauri::{LogicalPosition, Manager, Monitor};

#[tauri::command]
async fn get_floating_window_position(app: tauri::AppHandle) -> Result<(f64, f64), String> {
    let cursor_position = app.get_window("floating")
        .and_then(|w| w.current_monitor().ok())
        .flatten()
        .and_then(|monitor: Monitor| monitor.position().to_logical(monitor.scale_factor()).to_physical(monitor.scale_factor()).into())
        .ok_or_else(|| "无法获取鼠标位置".to_string())?;

    // 计算悬浮窗口的显示位置，例如在鼠标下方偏移一定距离
    let window_width = 300.0; // 假设悬浮窗口宽度
    let window_height = 100.0; // 假设悬浮窗口高度

    Ok((cursor_position.x - window_width / 2.0, cursor_position.y + 20.0))
}
```

### 4. 智谱AI API调用
```rust
// 使用reqwest进行HTTP请求
async fn translate_text(
    text: &str,
    source_lang: &str,
    target_lang: &str
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://open.bigmodel.cn/api/paas/v4/chat/completions")
        .json(&json!({
            "model": "glm-4.6",
            "stream": false,
            "thinking": {"type": "disabled"},
            "response_format": {"type": "text"},
            "messages": [{
                "role": "user",
                "content": format!(r#"{{"content": "{}", "langs": ["{}", "{}"]}}"#, text, source_lang, target_lang)
            }]
        }))
        .header("Authorization", format!("Bearer {}", get_api_key()?))
        .send()
        .await?;

    // 处理响应
}
```

## 支持的语言列表 (20+)

### 主要语言
- 中文（简体、繁体）
- 英语
- 日语
- 韩语
- 法语
- 德语
- 西班牙语
- 俄语
- 阿拉伯语
- 葡萄牙语
- 意大利语
- 荷兰语
- 瑞典语
- 挪威语
- 丹麦语
- 芬兰语
- 波兰语
- 捷克语
- 匈牙利语
- 罗马尼亚语

## 预期交付物

1. **完整的桌面应用程序**：
   - Windows安装包 (.exe)
   - macOS应用包 (.dmg)
   - Linux AppImage

2. **源代码**：
   - 完整的前后端代码
   - 详细的代码注释
   - 部署文档

3. **用户文档**：
   - 安装和使用指南
   - 功能说明
   - 常见问题解答

## 风险评估与应对

### 技术风险
- **智谱AI API变更**: 建立适配层，便于API版本升级
- **跨平台兼容性**: 使用Tauri的跨平台特性，充分测试
- **性能问题**: 实现缓存机制，优化请求频率

### 用户体验风险
- **快捷键冲突**: 提供快捷键自定义功能
- **悬浮窗口位置**: 智能位置计算，避免遮挡重要内容
- **网络延迟**: 提供离线缓存和降级方案

这个计划提供了完整的技术路径和实施细节，确保项目的成功交付。