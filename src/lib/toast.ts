export type ToastPayload = {
  id: number
  message: string
  type?: 'info' | 'error'
}

type Listener = (payload: ToastPayload) => void

const listeners = new Set<Listener>()
let idCounter = 0

export function showToast(message: string, type: 'info' | 'error' = 'info') {
  const payload: ToastPayload = { id: ++idCounter, message, type }
  listeners.forEach((cb) => cb(payload))
}

export function onToast(listener: Listener) {
  listeners.add(listener)
  return () => listeners.delete(listener)
}
