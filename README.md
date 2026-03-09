# Ace Translator

Ace Translator is a Tauri + Vue desktop translator focused on fast keyboard-driven workflows and multi-provider AI translation.

## Highlights

- Desktop app built with `Tauri 2`, `Vue 3`, and `TypeScript`
- Multiple providers: `Zhipu`, `OpenAI`, `Claude`, and `Ollama`
- Global shortcuts for quick translation
- Local history, cache management, and app logs
- No embedded API keys or private update infrastructure in this open-source branch

## Security Notes

- This branch does not include any shared API credentials.
- Local development secrets should stay in untracked `.env.*` files.
- Tauri signing keys must be provided through environment variables if you add your own signed release workflow later.

## Development

```bash
pnpm install
pnpm tauri dev
```

Frontend only:

```bash
pnpm dev
pnpm build
```

Rust checks:

```bash
cd src-tauri
cargo check
```

## Configuration

1. Copy `.env.example` to a local `.env.development` or other untracked `.env.*` file if needed.
2. Configure at least one AI provider in the app settings.
3. For cloud providers, use your own API key.
4. For `Ollama`, point the base URL at your local or self-hosted instance.

## Build

Standard build:

```bash
pnpm tauri build
```

Helper scripts:

```bash
pnpm tauri:build:mac
pnpm tauri:build:win
```

If you later enable signed updater artifacts for your own distribution, inject these values externally instead of committing them:

```bash
export TAURI_SIGNING_PRIVATE_KEY="..."
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="..."
```

## macOS Permissions

On macOS, the app may request Accessibility permission so global shortcuts can work. If the system blocks it:

1. Open `System Settings > Privacy & Security > Accessibility`
2. Add Ace Translator
3. Enable permission for the app

## Open-Source Release Flow

- Source code: [GitHub Repository](https://github.com/ace0109/ace-translator)
- Releases: [GitHub Releases](https://github.com/ace0109/ace-translator/releases)

This branch intentionally removes private deployment files, signing material, and vendor credentials so it can be published safely.
