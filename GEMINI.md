# Ace Translator Project Context

## Project Overview
Ace Translator is a desktop AI translation application built with Tauri 2.0. It integrates Zhipu AI (GLM-4) to provide high-quality translations. The application features a main translation window for detailed work and a floating window triggered by a global shortcut for quick "select-to-translate" functionality across the operating system.

## Tech Stack

### Frontend
*   **Framework**: Vue 3 (Composition API)
*   **Language**: TypeScript
*   **Build Tool**: Vite
*   **UI Library**: Naive UI
*   **State Management**: Pinia
*   **Icons**: @vicons/ionicons5
*   **Routing**: Vue Router

### Backend (Rust)
*   **Framework**: Tauri 2.0
*   **Database**: SQLite (via `sqlx`)
*   **HTTP Client**: `reqwest`
*   **Input Simulation**: `enigo` (for simulating copy commands)
*   **Input Monitoring**: `rdev` (for passive hotkey detection)
*   **Security**: `ring` (for encrypting API keys)
*   **Plugins**: `tauri-plugin-clipboard-manager`, `tauri-plugin-opener`, `tauri-plugin-shell`

## Key Features
1.  **Main Window**: A comprehensive interface for text translation, featuring language selection (source/target), input/output areas, and history management.
2.  **Floating Window**: A lightweight, minimalist popup that appears near the cursor for immediate translation results.
3.  **Smart "Select to Translate"**:
    *   Listens for a specific global hotkey sequence (e.g., Double Ctrl/Cmd+C).
    *   Automatically simulates a copy command (`Ctrl+C`/`Cmd+C`) to capture selected text.
    *   Reads the clipboard and instantly triggers the floating window with the translation.
4.  **System Tray**: The application runs in the background with a system tray icon for quick access and management.
5.  **Secure & Local**: API keys are encrypted locally. Translation history is stored in a local SQLite database.

## Project Structure

### `src/` (Frontend)
*   `components/`: Vue components organized by type.
    *   `features/`: Core functional components (`Translator.vue`, `FloatingTranslator.vue`, `Settings.vue`).
    *   `common/`: Reusable UI components (`LanguageSelector.vue`, `LoadingSpinner.vue`).
    *   `layout/`: App layout structures.
*   `stores/`: Pinia state management modules (`translation.ts`, `settings.ts`, `history.ts`).
*   `services/`: Frontend-side service abstractions.
*   `App.vue`: Root component handling global layout.

### `src-tauri/` (Backend)
*   `src/lib.rs`: The application entry point. Initializes plugins, database, system tray, and starts the background hotkey listener.
*   `src/commands/`: Tauri commands callable from the frontend.
    *   `translation.rs`: Handles API calls to Zhipu AI and caching.
    *   `settings.rs`: Manages encrypted settings storage.
    *   `system.rs`: Window visibility and management.
*   `src/services/`: Internal Rust logic.
    *   `zhipu.rs`: API client implementation.
    *   `clipboard.rs`: Clipboard manipulation and input simulation.
    *   `database.rs`: SQLite connection and schema migration.
    *   `encryption.rs`: AES encryption/decryption logic.
*   `capabilities/default.json`: Tauri 2.0 permission configuration.
*   `tauri.conf.json`: Main Tauri configuration (windows, bundle settings).

## Development Workflow

### Prerequisites
*   Node.js (v18+)
*   pnpm
*   Rust (Stable)
*   Tauri CLI

### Installation
Install frontend dependencies:
```bash
pnpm install
```

### Running in Development
Start the frontend and backend in development mode with hot-reloading:
```bash
pnpm tauri dev
# Or simply
pnpm dev
```

### Building for Production
Build the optimized application package for your OS:
```bash
pnpm tauri build
```

## Architecture Notes

### Communication
*   **Frontend to Backend**: Uses `invoke('command_name', payload)`.
*   **Backend to Frontend**: Uses `window.emit('event_name', payload)` (e.g., triggering the floating window).

### Database Schema
*   `settings`: Key-value store for app configuration (API key, theme, shortcuts).
*   `translation_history`: Stores past translations with source/target languages and timestamps.

### Permissions
This project uses Tauri 2.0's capability system. Permissions are explicitly defined in `src-tauri/capabilities/default.json`. Key permissions include:
*   `clipboard-manager:default` & `allow-clear`
*   `core:tray:default`
*   `core:window:default`

## Configuration
*   **API Provider**: Zhipu AI (GLM-4 model family).
*   **Environment**: API keys and sensitive data are stored locally in the user's app data directory, encrypted using `ring`.
