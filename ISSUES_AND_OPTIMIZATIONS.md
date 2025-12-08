# Ace Translator - 问题与优化清单

> 生成日期: 2025-12-08
> 分析范围: 前端 (Vue.js) + 后端 (Rust/Tauri)

---

## 目录

- [严重问题](#严重问题)
- [中等问题](#中等问题)
- [性能优化](#性能优化)
- [代码质量](#代码质量)
- [平台一致性问题](#平台一致性问题)
- [修复优先级](#修复优先级)

---

## 严重问题

### 1. 🔴 硬编码加密密钥（安全漏洞）

**文件:** `src-tauri/src/services/encryption.rs:6-11`

```rust
const MASTER_KEY_BYTES: [u8; 32] = [
    0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF,
    // ... 硬编码值
];
```

**问题描述:**
API 密钥使用硬编码的加密密钥，所有用户的安装都使用相同的密钥。攻击者从一个二进制文件提取密钥后可以解密所有用户的 API 密钥。

**影响:** 严重 - API 密钥存储不安全

**建议修复:**
使用操作系统的 Keychain/Credential Manager：
- macOS: Keychain Services
- Windows: Credential Manager
- Tauri 插件: `tauri-plugin-keyring`

---

### 2. 🔴 Mutex 锁 unwrap 可能导致 panic

**文件:** `src-tauri/src/services/hotkey/mod.rs:260, 422`

```rust
let mut guard = state.lock().unwrap();
```

**问题描述:**
热键监听器在新线程中调用 `.unwrap()` 获取 Mutex 锁。如果 Mutex 被毒化（另一个线程持有锁时发生 panic），这里会导致整个热键监听线程崩溃，用户将失去热键功能直到重启应用。

**影响:** 高 - 用户可能意外失去热键功能

**建议修复:**
```rust
// 方案 1: 使用 map_err 记录错误
let mut guard = state.lock().map_err(|e| {
    log::error!("Failed to acquire hotkey state lock: {}", e);
    e
})?;

// 方案 2: 使用 unwrap_or_else 提供默认行为
if let Ok(mut guard) = state.lock() {
    // 正常逻辑
} else {
    log::warn!("Hotkey state lock poisoned, skipping event");
}
```

---

## 中等问题

### 3. 🟠 LogsWindow 定时器可能内存泄漏

**文件:** `src/components/features/LogsWindow.vue:140-147`

```typescript
onMounted(() => {
  refreshLogs()
  refreshInterval = window.setInterval(() => {
    if (autoRefresh.value) {
      refreshLogs()
    }
  }, 2000)
})
```

**问题描述:**
如果组件快速挂载/卸载多次，定时器可能没有被正确清理就创建了新的，导致多个定时器同时运行。

**建议修复:**
```typescript
onMounted(() => {
  // 确保先清理旧的定时器
  if (refreshInterval) {
    window.clearInterval(refreshInterval)
  }
  refreshLogs()
  refreshInterval = window.setInterval(() => {
    if (autoRefresh.value) {
      refreshLogs()
    }
  }, 2000)
})

onUnmounted(() => {
  if (refreshInterval) {
    window.clearInterval(refreshInterval)
    refreshInterval = null
  }
})
```

---

### 4. 🟠 数据库写入错误被静默忽略

**文件:** `src-tauri/src/commands/translation.rs:663-675`

```rust
tokio::spawn(async move {
    let _ = sqlx::query(...)
        .execute(&db)
        .await;
});
```

**问题描述:**
翻译历史保存失败时没有日志或错误报告，用户不知道历史记录没有保存成功。

**建议修复:**
```rust
tokio::spawn(async move {
    if let Err(e) = sqlx::query(...)
        .execute(&db)
        .await
    {
        log::error!("Failed to save translation history: {}", e);
    }
});
```

---

### 5. 🟠 流式翻译请求取消存在竞态条件

**文件:** `src-tauri/src/commands/translation.rs:46-65`

**问题描述:**
`cancelled_requests` 集合在流式事件中被检查，但存在竞态条件：事件可能在请求被标记为取消之前已经入队，然后在之后被处理。

**建议修复:**
在所有事件处理器中更防御性地嵌入 request_id 检查，或使用 channel 来通知取消。

---

### 6. 🟠 剪贴板读取可能阻塞热键监听

**文件:** `src-tauri/src/services/clipboard.rs:30-36`

```rust
pub fn read_clipboard(app: &AppHandle) -> Result<String, String> {
    app.clipboard()
        .read_text()
        .map_err(|e| e.to_string())
}
```

**问题描述:**
在热键处理器中同步调用剪贴板读取，如果剪贴板被其他应用锁定，可能导致热键监听器卡住。

**建议修复:**
使用带超时的异步调用：
```rust
pub async fn read_clipboard_with_timeout(app: &AppHandle) -> Result<String, String> {
    tokio::time::timeout(
        Duration::from_millis(500),
        tokio::task::spawn_blocking(move || {
            app.clipboard().read_text()
        })
    )
    .await
    .map_err(|_| "Clipboard read timeout".to_string())?
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
```

---

### 7. 🟠 MainTranslator 组件卸载时未清理 resize 定时器

**文件:** `src/components/features/MainTranslator.vue:546-572`

```typescript
let resizeTimer: number | null = null

watch([translationResults, streamingResults, ...], () => {
  if (resizeTimer) {
    window.clearTimeout(resizeTimer)
  }
  resizeTimer = window.setTimeout(() => {
    updateWindowHeight()
    resizeTimer = null
  }, 20)
}, { deep: true })
```

**问题描述:**
如果组件在 resize 等待期间卸载，`updateWindowHeight` 会在已卸载的组件上执行。

**建议修复:**
```typescript
onUnmounted(() => {
  if (resizeTimer) {
    window.clearTimeout(resizeTimer)
    resizeTimer = null
  }
})
```

---

## 性能优化

### 8. 🟡 深度监听大对象效率低

**文件:** `src/components/features/MainTranslator.vue:565`

```typescript
watch([translationResults, streamingResults, sourcePreview, streamingLoading, enabledProviders], () => {
  // ...
}, { deep: true })
```

**问题描述:**
深度监听 `streamingResults`（一个 Map）开销很大，每个流式块都会触发整个 Map 的深度比较。

**建议修复:**
```typescript
// 使用浅监听 + 手动触发
const resizeTrigger = ref(0)

// 在需要时手动触发
const triggerResize = () => {
  resizeTrigger.value++
}

watch(resizeTrigger, () => {
  updateWindowHeight()
})
```

---

### 9. 🟡 流式事件匹配效率低

**文件:** `src/composables/useStreamingTranslation.ts:132-144`

```typescript
const handleChunk = (data: StreamEventData) => {
    const provider = data.provider
    for (const [key, result] of streamingResults.value.entries()) {
        if (result.provider === provider && !result.isComplete) {
            // Update...
            break
        }
    }
}
```

**问题描述:**
每个块都需要线性搜索 Map，复杂度为 O(n)。

**建议修复:**
使用 provider 名称作为 Map 的键，实现 O(1) 查找：
```typescript
const streamingResults = ref<Map<string, StreamingResult>>(new Map())

// 使用 provider 作为 key
streamingResults.value.set(provider, result)
```

---

### 10. 🟡 Provider 配置循环内有异步调用

**文件:** `src-tauri/src/commands/settings.rs:100-160`

```rust
for row in rows {
    let api_key = if !row.api_key.is_empty() {
        decrypt_api_key(&row.api_key).unwrap_or_default()
    } else if row.provider_name == "zhipu" {
        // 循环内的异步调用！
        resolve_default_zhipu_api_key().await
    }
    // ...
}
```

**问题描述:**
`resolve_default_zhipu_api_key()` 在循环内被调用，应该批量处理或缓存。

**建议修复:**
```rust
// 先获取默认 key
let default_zhipu_key = resolve_default_zhipu_api_key().await;

for row in rows {
    let api_key = if !row.api_key.is_empty() {
        decrypt_api_key(&row.api_key).unwrap_or_default()
    } else if row.provider_name == "zhipu" {
        default_zhipu_key.clone()
    }
    // ...
}
```

---

## 代码质量

### 11. 🔵 API 密钥解密失败静默处理

**文件:** `src-tauri/src/commands/translation.rs:125-126`

```rust
decrypt_api_key(&row.api_key).unwrap_or_default()
```

**问题描述:**
解密失败时静默回退到空字符串，没有日志记录。

**建议修复:**
```rust
decrypt_api_key(&row.api_key).unwrap_or_else(|e| {
    log::warn!("Failed to decrypt API key for {}: {}", row.provider_name, e);
    String::new()
})
```

---

### 12. 🔵 窗口操作 unwrap

**文件:** `src-tauri/src/lib.rs:216`

```rust
window.hide().unwrap();
```

**建议修复:**
```rust
if let Err(e) = window.hide() {
    log::warn!("Failed to hide window: {}", e);
}
```

---

### 13. 🔵 魔数未命名常量

**文件:** `src-tauri/src/services/hotkey/mod.rs:264`

```rust
let key_code = event.get_integer_value_field(9) as CGKeyCode;
```

**建议修复:**
```rust
const CGEVENT_KEYCODE_FIELD: i64 = 9;
let key_code = event.get_integer_value_field(CGEVENT_KEYCODE_FIELD) as CGKeyCode;
```

---

### 14. 🔵 缺少输入验证

**文件:** `src-tauri/src/commands/settings.rs`

**问题描述:**
保存 provider 配置时没有验证 `model` 是否在该 provider 的可用模型列表中。

**建议修复:**
添加模型验证逻辑：
```rust
pub async fn save_provider_config(...) -> Result<(), String> {
    // 验证模型是否有效
    if !config.available_models.contains(&config.model) {
        return Err(format!("Invalid model '{}' for provider '{}'", config.model, config.provider_name));
    }
    // ...
}
```

---

## 平台一致性问题

### 15. 🔴 热键配置 UI 缺失

**相关文件:**
- `src/locales/zh-CN.json:95-99` - 国际化字符串已存在
- `src/locales/en.json:95-99` - 国际化字符串已存在
- `src-tauri/src/commands/settings.rs:272-319` - 后端命令已实现
- `src/components/features/Settings.vue` - **缺失热键配置 UI**

**问题描述:**
后端已经实现了完整的热键配置系统，国际化字符串也准备好了，但 Settings.vue 中完全没有热键配置界面。

**已有国际化字符串:**
```json
{
  "hotkey": {
    "title": "快捷键设置",
    "doubleCopy": "双击复制翻译",
    "doubleCopyDescMac": "双击 ⌘+C 唤起翻译",
    "doubleCopyDescWin": "双击 Ctrl+C 唤起翻译",
    "altSpace": "快捷键打开窗口",
    "altSpaceDescMac": "Option+Space 打开窗口",
    "altSpaceDescWin": "Alt+Space 打开窗口"
  }
}
```

**建议修复:**
在 Settings.vue 中添加热键配置部分。

---

### 16. 🟠 输入框占位符未区分平台

**文件:** `src/components/features/MainTranslator.vue:44-48`

**当前国际化文本:**
- 中文: `"输入要翻译的文本 (Ctrl+Enter 翻译)"`
- 英文: `"Enter text to translate (Ctrl+Enter)"`

**问题描述:**
macOS 用户应该看到 `⌘+Enter` 而不是 `Ctrl+Enter`。

**建议修复:**
添加平台感知的占位符：

```json
// zh-CN.json
{
  "translator": {
    "inputPlaceholderMac": "输入要翻译的文本 (⌘+Enter 翻译)",
    "inputPlaceholderWin": "输入要翻译的文本 (Ctrl+Enter 翻译)"
  }
}
```

```typescript
// MainTranslator.vue
const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
const placeholder = computed(() =>
  isMac ? t('translator.inputPlaceholderMac') : t('translator.inputPlaceholderWin')
)
```

---

### 17. 🔵 未使用的平台检测变量

**文件:** `src/components/features/Settings.vue:274`

```typescript
const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
```

**问题描述:**
这个变量声明了但从未使用，应该是为热键帮助文本准备的残留代码。

**建议修复:**
实现热键配置 UI 时使用此变量，或删除未使用代码。

---

## 已正确实现的平台差异 ✅

| 功能 | macOS | Windows | 文件 | 状态 |
|-----|-------|---------|------|------|
| 热键监听 | CGEventTap | rdev | `hotkey/mod.rs` | ✅ |
| 复制快捷键 | ⌘+C | Ctrl+C | `hotkey/mod.rs` | ✅ |
| 呼出窗口 | Option+Space | Alt+Space | `hotkey/mod.rs` | ✅ |
| 剪贴板模拟 | Meta键 | Control键 | `clipboard.rs:11-14` | ✅ |
| 托盘菜单文字 | Option+Space | Alt+Space | `tray.rs:7-11` | ✅ |
| 辅助功能权限 | 需要 | 不需要 | `hotkey/mod.rs:35-117` | ✅ |
| 窗口空间处理 | 支持 | 跳过 | `window.rs` | ✅ |
| 平台依赖 | cocoa, core-graphics | 无 | `Cargo.toml:41-46` | ✅ |

---

## 修复优先级

### P0 - 立即修复（安全/稳定性）
| # | 问题 | 文件 | 预估工作量 |
|---|------|------|-----------|
| 1 | 硬编码加密密钥 | `encryption.rs` | 中 |
| 2 | Mutex unwrap panic | `hotkey/mod.rs` | 小 |

### P1 - 尽快修复（功能完整性）
| # | 问题 | 文件 | 预估工作量 |
|---|------|------|-----------|
| 15 | 热键配置 UI 缺失 | `Settings.vue` | 中 |
| 16 | 占位符未区分平台 | `MainTranslator.vue` + locales | 小 |

### P2 - 计划修复（可靠性）
| # | 问题 | 文件 | 预估工作量 |
|---|------|------|-----------|
| 3 | LogsWindow 定时器泄漏 | `LogsWindow.vue` | 小 |
| 4 | 数据库写入错误静默 | `translation.rs` | 小 |
| 5 | 请求取消竞态条件 | `translation.rs` | 中 |
| 6 | 剪贴板阻塞风险 | `clipboard.rs` | 中 |
| 7 | resize 定时器未清理 | `MainTranslator.vue` | 小 |

### P3 - 后续优化（性能）
| # | 问题 | 文件 | 预估工作量 |
|---|------|------|-----------|
| 8 | 深度监听效率 | `MainTranslator.vue` | 中 |
| 9 | 流式事件匹配效率 | `useStreamingTranslation.ts` | 小 |
| 10 | Provider 配置循环优化 | `settings.rs` | 小 |

### P4 - 代码质量（维护性）
| # | 问题 | 文件 | 预估工作量 |
|---|------|------|-----------|
| 11 | 解密失败静默处理 | `translation.rs` | 小 |
| 12 | 窗口操作 unwrap | `lib.rs` | 小 |
| 13 | 魔数未命名 | `hotkey/mod.rs` | 小 |
| 14 | 缺少输入验证 | `settings.rs` | 小 |
| 17 | 未使用变量 | `Settings.vue` | 小 |

---

## 问题统计

| 严重程度 | 数量 | 类别 |
|---------|-----|------|
| 🔴 严重 | 3 | 安全、稳定性、功能缺失 |
| 🟠 中等 | 6 | 内存泄漏、竞态条件、平台一致性 |
| 🟡 性能 | 3 | 优化建议 |
| 🔵 代码质量 | 5 | 维护性改进 |

**总计: 17 个问题**
