export interface ParsedBackendError {
  code: string | null
  message: string
  raw: string
}

const CODED_ERROR_PATTERN = /^\[([A-Z0-9_]+)\]\s*(.*)$/s

export function parseBackendError(error: unknown): ParsedBackendError {
  const raw =
    typeof error === 'string'
      ? error
      : (error as any)?.message || String(error || '')

  const match = raw.match(CODED_ERROR_PATTERN)
  if (!match) {
    return {
      code: null,
      message: raw,
      raw,
    }
  }

  return {
    code: match[1] || null,
    message: (match[2] || '').trim() || raw,
    raw,
  }
}
