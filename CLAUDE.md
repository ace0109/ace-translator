# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **Ace Translator** (v0.2.0), a Tauri + Vue.js desktop application for AI-powered translation. The project follows a standard Tauri architecture with a Rust backend and Vue.js frontend.

### Architecture

- **Frontend**: Vue 3 + TypeScript + vue-i18n, built with Vite
- **Backend**: Rust using Tauri 2 framework
- **Build System**: Vite for frontend, Cargo for Rust backend
- **Package Manager**: pnpm (uses pnpm-lock.yaml)
- **Database**: SQLite (via SQLx)

### Key Files Structure

```
ace-translator/
├── src/                          # Vue.js frontend source
│   ├── main.ts                   # Vue app entry point
│   ├── App.vue                   # Root Vue component
│   ├── components/
│   │   ├── features/
│   │   │   ├── MainTranslator.vue    # Main translation window
│   │   │   ├── Settings.vue          # Settings page
│   │   │   └── HistoryAndLogs.vue    # History & logs window
│   │   ├── ui/                   # UI components (shadcn-vue style)
│   │   └── common/               # Common components
│   ├── locales/                  # i18n language files
│   │   ├── index.ts              # i18n configuration
│   │   ├── zh-CN.json            # Chinese translations
│   │   └── en.json               # English translations
│   ├── stores/                   # Pinia stores
│   └── assets/                   # Static assets
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri application entry point
│   │   ├── lib.rs                # AppState and command registration
│   │   ├── commands/
│   │   │   ├── translation.rs    # Translation commands
│   │   │   ├── settings.rs       # Settings commands
│   │   │   └── system.rs         # System commands
│   │   ├── services/
│   │   │   ├── ai/               # AI provider implementations
│   │   │   │   ├── mod.rs
│   │   │   │   ├── provider.rs   # AIProvider trait
│   │   │   │   ├── zhipu.rs      # Zhipu AI provider
│   │   │   │   ├── openai.rs     # OpenAI provider
│   │   │   │   ├── claude.rs     # Claude provider
│   │   │   │   └── ollama.rs     # Ollama provider
│   │   │   ├── hotkey/           # Hotkey listener
│   │   │   ├── database.rs       # Database initialization
│   │   │   ├── clipboard.rs      # Clipboard operations
│   │   │   └── encryption.rs     # API key encryption
│   │   ├── config/
│   │   │   └── prompts.rs        # Translation prompts
│   │   └── models/               # Data models
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── package.json                  # Frontend dependencies
├── PLAN_TODO.md                  # Development roadmap
└── README.md                     # Project documentation
```

## Development Commands

### Frontend Development
```bash
# Start development server
pnpm dev

# Build for production
pnpm build

# Type checking only
vue-tsc --noEmit
```

### Tauri Development
```bash
# Run in development mode (starts both frontend dev server and Tauri)
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Rust Check
```bash
# Check Rust compilation
cd src-tauri && cargo check
```

## Key Features (v0.2.0)

### Window Architecture
- Single main window (no separate floating window)
- Custom title bar with drag region
- Window height auto-adjusts to content
- Trigger modes:
  - Double Ctrl/Cmd+C: Shows near mouse position
  - Alt/Option+Space: Shows at screen center
  - Tray left-click: Shows at screen center

### Multi-Provider Translation
- Parallel translation with multiple AI providers
- Supported providers: Zhipu AI, OpenAI, Claude, Ollama
- Request cancellation support using `futures::Abortable`
- 30-second timeout for cloud providers, 60-second for Ollama

### Hotkey System
- Double-copy translation (Ctrl/Cmd+C × 2)
- Alt/Option+Space to open window
- Configurable enable/disable in settings
- Platform-specific implementations (CGEventTap for macOS, rdev for Windows)

### Internationalization
- vue-i18n for interface language switching
- Supported languages: Chinese (zh-CN), English (en)
- Language preference saved to localStorage

## Tauri Commands

### Translation
- `translate_multi` - Multi-provider parallel translation
- `translate_text` - Single provider translation (legacy)
- `cancel_translation` - Cancel specific translation request
- `cancel_all_translations` - Cancel all ongoing translations

### Settings
- `get_settings` / `save_settings` - App settings
- `get_provider_configs` / `save_provider_config` - AI provider configs
- `test_provider` - Test provider API connection
- `get_hotkey_config` / `save_hotkey_config` - Hotkey settings

### System
- `show_main_window` / `show_main_window_centered`
- `show_settings_window` / `show_history_window`
- `resize_main_window` - Dynamic window height
- `cache_stats` / `clear_cache` - Cache management
- `get_translation_history` / `delete_history_entry`
- `get_logs` / `clear_logs` - Log management

## Configuration

### Window Configuration
- Main window: 400×500, no decorations, resizable
- Settings window: 600×700
- History window: 600×500

### Port Configuration
- Frontend dev server: port 1420
- HMR websocket: port 1421 (when using TAURI_DEV_HOST)

## Adding New Features

### Adding a New AI Provider
1. Create `src-tauri/src/services/ai/<provider>.rs`
2. Implement the `AIProvider` trait
3. Add to `src-tauri/src/services/ai/mod.rs`
4. Add to provider configs in `settings.rs`
5. Update database migration in `database.rs`

### Adding New Translations
1. Add keys to `src/locales/zh-CN.json`
2. Add corresponding keys to `src/locales/en.json`
3. Use `t('key.path')` in Vue components
