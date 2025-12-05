# Ace Translator

Ace AI翻译器

# 构建指南

## Windows 打包
1. 安装依赖（锁定）：`pnpm install --frozen-lockfile`
2. 构建 x64 安装包/可执行：`pnpm tauri build --target x86_64-pc-windows-msvc`
   - 产物位置：`src-tauri/target/x86_64-pc-windows-msvc/release/`（NSIS 在 `bundle/nsis/*.exe`，单文件 exe 同级）
   - 本地 Windows 不需要 `--runner cargo-xwin`。

## macOS 打包
1. 安装依赖：`pnpm install --frozen-lockfile`
2. 添加目标（若未添加）：`rustup target add x86_64-apple-darwin aarch64-apple-darwin`
3. 构建：
   - 通用二进制：`pnpm tauri build --target universal-apple-darwin`
   - 或单独：`pnpm tauri build --target x86_64-apple-darwin` / `pnpm tauri build --target aarch64-apple-darwin`
   - 产物位置：`src-tauri/target/<target>/release/bundle/dmg/`

## macOS 安装说明

由于应用未经过 Apple 公证，首次打开时会被 macOS Gatekeeper 拦截。请按以下步骤操作：

1. **首次打开被拦截时**：右键点击应用 → 选择「打开」→ 在弹出的对话框中点击「打开」
2. **或者**：前往「系统偏好设置」→「隐私与安全性」→ 在底部找到被阻止的应用，点击「仍然打开」
3. **授予辅助功能权限**：应用需要辅助功能权限来监听全局快捷键（双击 Cmd+C 触发悬浮翻译）
   - 首次使用时会自动弹出权限请求
   - 或手动添加：「系统偏好设置」→「隐私与安全性」→「辅助功能」→ 添加 Ace Translator

# System Promts

```json
{
  "role": "system",
  "content": "你是一个专业的翻译，你直接帮助用户翻译用户想要翻译的内容，不需要回答无关的内容，只需要回复翻译内容即可，格式参考：{zh_CN: '你好', en: 'Hello'}。根据用户要求的语言进行翻译。"
}
```
