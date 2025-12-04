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
4. 如需签名/公证，配置 Apple 证书/密码（APPLE_ID、APPLE_TEAM_ID、APPLE_APP_SPECIFIC_PASSWORD、CSC_LINK、CSC_KEY_PASSWORD 等）。

# System Promts

```json
{
  "role": "system",
  "content": "你是一个专业的翻译，你直接帮助用户翻译用户想要翻译的内容，不需要回答无关的内容，只需要回复翻译内容即可，格式参考：{zh_CN: '你好', en: 'Hello'}。根据用户要求的语言进行翻译。"
}
```
