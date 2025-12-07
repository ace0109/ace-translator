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
  const streamingResults = ref<Map<string, StreamingResult>>(new Map())
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  let unlisten: (() => void) | null = null

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

  const initProviders = (providers: Array<{ provider: string; model: string }>) => {
    streamingResults.value.clear()
    providers.forEach(p => {
      const key = `${p.provider}-${p.model}`
      streamingResults.value.set(key, {
        provider: p.provider,
        model: p.model,
        content: '',
        isComplete: false,
        loading: true,
      })
    })
    isLoading.value = true
    error.value = null
  }

  const handleStart = (data: StreamEventData) => {
    const key = `${data.provider}-${data.model}`
    const existing = streamingResults.value.get(key)
    if (existing) {
      existing.loading = true // 仍在加载（流式传输中）
      existing.error = undefined
    } else {
      streamingResults.value.set(key, {
        provider: data.provider,
        model: data.model,
        content: '',
        isComplete: false,
        loading: true,
      })
    }
    // isLoading.value = true; // Overall isLoading is handled by allComplete check
    error.value = null
  }

  const handleChunk = (data: StreamEventData) => {
    const provider = data.provider
    // 查找匹配的 result
    for (const [key, result] of streamingResults.value.entries()) {
      if (result.provider === provider && !result.isComplete) {
        result.content += data.content || ''
        break
      }
    }
  }

  const handleDone = (data: StreamEventData) => {
    const key = `${data.provider}-${data.model}`
    const result = streamingResults.value.get(key)
    if (result) {
      result.content = data.full_translation || ''
      result.isComplete = true
      result.loading = false
    }

    // Update detected/target languages so the UI can display them
    if (onLanguageUpdate && (data.detected_source_lang || data.target_lang)) {
      onLanguageUpdate(data.detected_source_lang || '', data.target_lang || '')
    }

    // 检查是否全部完成
    const allComplete = Array.from(streamingResults.value.values()).every(r => r.isComplete)
    if (allComplete) {
      isLoading.value = false
    }
  }

  const handleError = (data: StreamEventData) => {
    // 标记对应的 result 为错误状态
    let found = false
    for (const result of streamingResults.value.values()) {
      if (result.provider === data.provider) {
        result.isComplete = true
        result.loading = false
        result.error = data.error
        found = true
        break
      }
    }
    
    // 如果是全局错误（没找到特定 provider），或者都出错了
    if (!found) {
       error.value = `${data.provider}: ${data.error}`
    }
    
    const allComplete = Array.from(streamingResults.value.values()).every(r => r.isComplete)
    if (allComplete) {
      isLoading.value = false
    }
  }

  const reset = () => {
    streamingResults.value.clear()
    isLoading.value = false
    error.value = null
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
