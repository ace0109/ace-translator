import { ref, onUnmounted, Ref } from 'vue'
import { listen } from '@tauri-apps/api/event'

export interface StreamEventData {
  provider: string
  model: string
  content?: string // For Chunk
  detected_source_lang?: string // For Done
  target_lang?: string // For Done
  full_translation?: string // For Done
  error?: string // For Error
  request_id: number // Added request_id to all event data
}

export interface StreamEvent {
  type: 'Start' | 'Chunk' | 'Done' | 'Error'
  data: StreamEventData
}

export interface StreamingResult {
  provider: string
  model: string
  content: string
  isComplete: boolean
  loading: boolean
  error?: string
  // request_id: number; // Not needed in StreamingResult itself, handled by the listener
}

export function useStreamingTranslation(
  currentRequestId: Ref<number>,
  onLanguageUpdate?: (detected: string, target: string) => void,
) {
  const STREAM_TIMEOUT_MS = 30000
  const streamingResults = ref<Map<string, StreamingResult>>(new Map())
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  let unlisten: (() => void) | null = null
  let timeoutHandle: number | null = null

  // 初始化监听器
  const initListener = async () => {
    unlisten = await listen('translation-stream', (event: any) => {
      const streamEvent = event.payload as StreamEvent
      
      // Only process events that match the current active request ID
      if (streamEvent.data.request_id !== currentRequestId.value) {
        return
      }

      switch (streamEvent.type) {
        case 'Start':
          handleStart(streamEvent.data)
          break
        case 'Chunk':
          handleChunk(streamEvent.data)
          break
        case 'Done':
          handleDone(streamEvent.data)
          break
        case 'Error':
          handleError(streamEvent.data)
          break
      }
    })
  }

  const setResult = (key: string, updater: (current?: StreamingResult) => StreamingResult) => {
    const next = new Map(streamingResults.value)
    const current = next.get(key)
    next.set(key, updater(current))
    streamingResults.value = next
  }

  const initProviders = (providers: Array<{ provider: string; model: string }>) => {
    if (timeoutHandle) {
      window.clearTimeout(timeoutHandle)
      timeoutHandle = null
    }
    const requestIdSnapshot = currentRequestId.value

    const next = new Map<string, StreamingResult>()
    providers.forEach(p => {
      const key = `${p.provider}-${p.model}`
      next.set(key, {
        provider: p.provider,
        model: p.model,
        content: '',
        isComplete: false,
        loading: true,
      })
    })
    streamingResults.value = next
    isLoading.value = true
    error.value = null

    // 超时兜底：停止 loading 并给出提示，忽略后续迟到事件
    timeoutHandle = window.setTimeout(() => {
      if (currentRequestId.value !== requestIdSnapshot) return
      const timedOut = new Map<string, StreamingResult>()
      streamingResults.value.forEach((r, key) => {
        timedOut.set(key, {
          ...r,
          isComplete: true,
          loading: false,
          error: '请求超时，请重试',
        })
      })
      streamingResults.value = timedOut
      isLoading.value = false
      error.value = '请求超时，请重试'
      currentRequestId.value = 0 // 忽略后续迟到事件
      timeoutHandle = null
    }, STREAM_TIMEOUT_MS)
  }

  const handleStart = (data: StreamEventData) => {
    const key = `${data.provider}-${data.model}`
    setResult(key, (existing) => ({
      provider: data.provider,
      model: data.model,
      content: existing?.content || '',
      isComplete: existing?.isComplete || false,
      loading: true,
      error: undefined,
    }))
    error.value = null
  }

  const handleChunk = (data: StreamEventData) => {
    const provider = data.provider
    for (const [key, result] of streamingResults.value.entries()) {
      if (result.provider === provider && !result.isComplete) {
        setResult(key, (current) => ({
          provider: result.provider,
          model: result.model,
          content: (current?.content || '') + (data.content || ''),
          isComplete: current?.isComplete || false,
          loading: false, // 一旦有内容就不再显示 loading
          error: current?.error,
        }))
        break
      }
    }
  }

  const handleDone = (data: StreamEventData) => {
    const key = `${data.provider}-${data.model}`
    setResult(key, (result) => ({
      provider: data.provider,
      model: data.model,
      content: data.full_translation || result?.content || '',
      isComplete: true,
      loading: false,
      error: result?.error,
    }))

    // Update detected/target languages so the UI can display them
    if (onLanguageUpdate && (data.detected_source_lang || data.target_lang)) {
      onLanguageUpdate(data.detected_source_lang || '', data.target_lang || '')
    }

    // 检查是否全部完成
    const allComplete = Array.from(streamingResults.value.values()).every(r => r.isComplete)
    if (allComplete) {
      isLoading.value = false
      if (timeoutHandle) {
        window.clearTimeout(timeoutHandle)
        timeoutHandle = null
      }
    }
  }

  const handleError = (data: StreamEventData) => {
    // 标记对应的 result 为错误状态
    let found = false
    for (const [key, result] of streamingResults.value.entries()) {
      if (result.provider === data.provider) {
        setResult(key, (current) => ({
          provider: result.provider,
          model: result.model,
          content: current?.content || '',
          isComplete: true,
          loading: false,
          error: data.error,
        }))
        found = true
        break
      }
    }

    if (!found) {
      error.value = `${data.provider}: ${data.error}`
    }

    const allComplete = Array.from(streamingResults.value.values()).every(r => r.isComplete)
    if (allComplete) {
      isLoading.value = false
      if (timeoutHandle) {
        window.clearTimeout(timeoutHandle)
        timeoutHandle = null
      }
    }
  }

  const reset = () => {
    streamingResults.value = new Map()
    isLoading.value = false
    error.value = null
    if (timeoutHandle) {
      window.clearTimeout(timeoutHandle)
      timeoutHandle = null
    }
  }

  // 初始化
  initListener()

  // 清理
  onUnmounted(() => {
    if (unlisten) {
      unlisten()
    }
  })

  return {
    streamingResults,
    isLoading,
    error,
    reset,
    initProviders,
  }
}
