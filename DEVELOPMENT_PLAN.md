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
- **触发**: Ctrl+C+C (Win/Linux), Cmd+C+C (macOS)
- **显示**:
  - 小型悬浮窗口，显示在选中文本附近
  - 简洁设计，只显示翻译结果
  - 点击外部区域自动关闭
  - 支持拖拽移动
- **性能**: 响应时间 < 500ms

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
  sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }
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
      "dayjs": "^1.11.10"
    }
  }
  ```

#### 1.2 Tauri配置更新
- 更新 `tauri.conf.json`:
  - 添加多窗口配置（主窗口、悬浮窗口、设置窗口）
  - 配置系统托盘权限
  - 配置全局快捷键权限
  - 配置剪贴板访问权限

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
- 注册Ctrl+C+C / Cmd+C+C快捷键
- 实现选中文本检测
- 触发悬浮窗口显示

#### 4.2 系统托盘
- 托盘图标显示
- 右键菜单功能
- 后台运行管理

#### 4.3 窗口管理
- 多窗口协调
- 窗口状态同步
- 最小化/恢复逻辑

### Phase 5: 优化和测试 (1-2天)
**目标**: 性能优化和功能测试

#### 5.1 性能优化
- API请求去重
- 翻译缓存机制
- 内存使用优化
- 启动时间优化

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
```typescript
// 使用@vueuse/core进行快捷键监听
import { useMagicKeys } from '@vueuse/core'

const { ctrl_c, cmd_c } = useMagicKeys()

watch([ctrl_c, cmd_c], ([ctrl, cmd]) => {
  if (ctrl && isSecondPress) {
    // 触发悬浮翻译
  }
})
```

### 3. 悬浮窗口定位
```typescript
// 获取选中文本位置并计算悬浮窗口位置
async function getFloatingWindowPosition() {
  const selection = window.getSelection()
  const range = selection.getRangeAt(0)
  const rect = range.getBoundingClientRect()

  return {
    x: rect.left + rect.width / 2 - 150,
    y: rect.bottom + 10
  }
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