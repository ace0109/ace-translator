export function isTauriEnv() {
  if (typeof window === 'undefined') return false
  const w = window as any
  return Boolean(w.__TAURI__ || w.__TAURI_IPC__ || w.__TAURI_INTERNALS__)
}
