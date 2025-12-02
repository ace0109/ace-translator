# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **Ace Translator**, a Tauri + Vue.js desktop application for AI-powered translation. The project follows a standard Tauri architecture with a Rust backend and Vue.js frontend.

### Architecture

- **Frontend**: Vue 3 + TypeScript, built with Vite
- **Backend**: Rust using Tauri framework
- **Build System**: Vite for frontend, Cargo for Rust backend
- **Package Manager**: pnpm (uses pnpm-lock.yaml)

### Key Files Structure

```
ace-translator/
├── src/                    # Vue.js frontend source
│   ├── main.ts            # Vue app entry point
│   ├── App.vue            # Root Vue component
│   └── assets/            # Static assets
├── src-tauri/             # Rust backend
│   ├── src/main.rs        # Tauri application entry point
│   ├── src/lib.rs         # Tauri commands and core logic
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── package.json           # Frontend dependencies and scripts
├── vite.config.ts         # Vite configuration
└── tsconfig.json          # TypeScript configuration
```

## Development Commands

### Frontend Development
```bash
# Start development server
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

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

### Full Build Process
```bash
# This runs both frontend and backend build
pnpm build          # Build frontend first
pnpm tauri build    # Then build Tauri app
```

## Development Notes

### Tauri Commands
The project uses Tauri's command system for frontend-backend communication. Commands are defined in `src-tauri/src/lib.rs` and can be called from the frontend using `invoke()` from `@tauri-apps/api/core`.

Current commands:
- `greet(name: &str) -> String` - Example command that greets the user

### Configuration
- **Window Configuration**: Set in `src-tauri/tauri.conf.json`
  - Default size: 800x600
  - Development URL: http://localhost:1420
- **Frontend Build Output**: `dist/` (configured in tauri.conf.json)

### Adding New Features
- **Frontend Components**: Add to `src/` directory
- **Tauri Commands**: Add to `src-tauri/src/lib.rs` using the `#[tauri::command]` macro
- **Rust Dependencies**: Add to `src-tauri/Cargo.toml`
- **Frontend Dependencies**: Add to `package.json`

### TypeScript Configuration
- Strict mode enabled
- Target: ES2020
- Module resolution: bundler mode
- Vue SFC support enabled via vue-tsc

### Port Configuration
- Frontend dev server: port 1420
- HMR websocket: port 1421 (when using TAURI_DEV_HOST)