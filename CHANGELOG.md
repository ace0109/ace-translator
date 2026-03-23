# Changelog

All notable changes to this project are documented in this file.

## [0.0.1] - 2026-03-23

### Added

- Unified provider architecture based on OpenAI-compatible protocol.
- New preset providers: Xiaomi MiMo, MiniMax, Moonshot.
- Custom provider create/delete support in settings.
- Standalone TTS module (decoupled from translation provider).
- Xiaomi TTS support for speech synthesis.
- Audio playback buttons for both source text and translated text.
- Unified backend error-code envelope and frontend parsing helpers.

### Changed

- Translation provider enable strategy is now single-select (only one active at a time).
- Model configuration switched to free-text input.
- Model benchmark upgraded to multi-model dialog with cache/restore.
- Local language detection now uses `whatlang` (no extra model request for detection).
- VitePress homepage content refreshed to match current product capabilities.

### Fixed

- Provider enable toggle state mismatch issue.
- Main translation page provider display/update inconsistencies.
- Multiple settings-page interaction bugs around provider switching and validation.

### Open-source compliance updates

- Removed private keys and internal-only helper files from open-source branch.
- Removed hardcoded built-in secret key fallback.
- Switched default release/download references to GitHub public endpoints.
- Rewrote README as open-source friendly documentation.

### Verification

- `cargo check` passed.
- `cargo test` passed.
- `pnpm exec vue-tsc --noEmit` passed.
