<template>
  <div ref="mainContainer" class="relative flex w-full flex-col bg-background text-foreground rounded-xl border shadow-lg overflow-hidden">
    <!-- Custom Title Bar -->
    <header
      data-tauri-drag-region
      class="flex items-center justify-between h-8 px-2 bg-card border-b select-none shrink-0"
    >
      <span data-tauri-drag-region class="text-xs font-medium text-muted-foreground">{{ t('translator.title') }}</span>
      <div class="flex items-center gap-1">
        <!-- Pin Button -->
        <button
          class="p-1 rounded hover:bg-muted/60 transition"
          :class="{ 'text-primary': pinned, 'text-muted-foreground': !pinned }"
          @click="togglePin"
          :title="pinned ? t('translator.unpinWindow') : t('translator.pinWindow')"
        >
          <component :is="pinned ? PinOff : Pin" class="h-3.5 w-3.5" />
        </button>
        <!-- Settings Button -->
        <button
          class="p-1 rounded hover:bg-muted/60 transition text-muted-foreground"
          @click="openSettings"
          :title="t('translator.openSettings')"
        >
          <Settings class="h-3.5 w-3.5" />
        </button>
        <!-- Close Button -->
        <button
          class="p-1 rounded hover:bg-destructive/80 hover:text-destructive-foreground transition text-muted-foreground"
          @click="hideWindow"
          :title="t('translator.closeWindow')"
        >
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    </header>

    <!-- Content Area -->
    <div class="flex flex-col">
      <!-- Language Info Header -->
      <div class="flex items-center justify-between border-b bg-card px-3 py-2 text-sm shadow-sm shrink-0">
        <div class="flex items-center gap-2">
          <span class="text-xs text-muted-foreground">{{ t('translator.detected') }}:</span>
          <span class="rounded-md border bg-muted/40 px-2 py-0.5 text-xs">{{ detectedLabel }}</span>
        </div>
        <div class="flex items-center gap-1">
          <ArrowRight class="h-3 w-3 text-muted-foreground" />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-muted-foreground">{{ t('translator.target') }}:</span>
          <span class="rounded-md border bg-accent/60 px-2 py-0.5 text-xs text-accent-foreground">{{ targetLabel }}</span>
        </div>
      </div>

      <!-- Source Text Input -->
      <section class="shrink-0 border-b bg-muted/30 px-3 py-2">
        <div class="flex items-center justify-between mb-1">
          <p class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{{ t('translator.original') }}</p>
          <button
            v-if="sourcePreview"
            class="text-xs text-primary hover:underline"
            @click="startTranslation(sourcePreview)"
          >
            {{ t('translator.translateBtn') }}
          </button>
        </div>
        <div class="relative">
          <textarea
            v-model="sourcePreview"
            ref="sourceTextarea"
            class="w-full min-h-[60px] max-h-[20rem] overflow-auto rounded-md border bg-background/80 p-2 text-sm leading-relaxed shadow-inner focus:outline-none focus:ring-1 focus:ring-primary resize-none"
            :placeholder="t('translator.inputPlaceholder')"
            @input="autoResizeTextarea"
            @keydown.enter.exact.prevent="startTranslation(sourcePreview)"
            @keydown.enter.ctrl.exact.prevent="startTranslation(sourcePreview)"
            @keydown.enter.meta.exact.prevent="startTranslation(sourcePreview)"
          ></textarea>
        </div>
      </section>

      <!-- Translation Results -->
      <section class="px-3 py-2 space-y-2">
        <div v-if="enabledProviders.length === 0 && !streamingLoading" class="py-6 text-center text-sm text-muted-foreground">
          {{ t('translator.waitingTranslation') }}
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="provider in enabledProviders"
            :key="provider.name"
            class="rounded-lg border bg-card shadow-sm overflow-hidden"
          >
            <!-- Provider Header -->
            <div class="flex items-center justify-between border-b bg-muted/30 px-3 py-1.5">
              <div class="flex items-center gap-2">
                <span
                  class="h-2 w-2 rounded-full"
                  :class="{
                    'bg-green-500': getProviderCardState(provider).success && !getProviderCardState(provider).loading,
                    'bg-red-500': getProviderCardState(provider).error && !getProviderCardState(provider).loading,
                    'bg-gray-400 animate-pulse': getProviderCardState(provider).loading,
                    'bg-muted-foreground': !getProviderCardState(provider).loading && !getProviderCardState(provider).success && !getProviderCardState(provider).error,
                  }"
                />
                <span class="text-xs font-medium">{{ provider.display_name }}</span>
                <span class="text-[10px] text-muted-foreground">{{ getProviderCardState(provider).model || provider.config.model }}</span>
              </div>
              <button
                v-if="getProviderCardState(provider).success && !getProviderCardState(provider).loading"
                class="inline-flex items-center gap-1 rounded border bg-muted/60 px-2 py-0.5 text-[10px] text-muted-foreground transition hover:bg-accent hover:text-accent-foreground"
                @click="copyText(getProviderCardState(provider).translation)"
                :title="t('common.copy')"
              >
                <Copy class="h-3 w-3" /> {{ t('common.copy') }}
              </button>
            </div>

            <!-- Result Content -->
            <div class="px-3 py-2">
              <div v-if="getProviderCardState(provider).error" class="rounded-md border border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950/30 p-2">
                <p class="text-xs text-red-600 dark:text-red-400">
                  {{ getProviderCardState(provider).error || t('translator.translationFailed') }}
                </p>
              </div>
              <div v-else class="relative rounded-md border border-dashed bg-background/70 p-2">
                <p class="text-sm leading-relaxed text-foreground/90 whitespace-pre-wrap break-words min-h-[32px]">
                  {{
                    getProviderCardState(provider).translation
                      || (getProviderCardState(provider).loading ? t('translator.translating') : t('translator.waitingTranslation'))
                  }}
                </p>
                <div
                  v-if="getProviderCardState(provider).loading"
                  class="absolute right-2 top-2 flex items-center justify-center"
                >
                  <LoadingSpinner />
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- API Key Prompt Dialog -->
    <transition name="fade">
      <div
        v-if="showApiKeyPrompt"
        class="fixed inset-0 z-50 flex items-center justify-center bg-background/70 backdrop-blur"
      >
        <div class="w-[320px] rounded-lg border bg-card p-4 shadow-lg">
          <h3 class="text-base font-semibold text-foreground">{{ t('translator.noProvider') }}</h3>
          <p class="mt-2 text-sm text-muted-foreground">{{ t('translator.noProviderDesc') }}</p>
          <div class="mt-4 flex justify-end gap-2">
            <button
              class="rounded border px-3 py-1 text-sm text-muted-foreground transition hover:bg-muted/60"
              @click="dismissApiPrompt"
            >
              {{ t('common.later') }}
            </button>
            <button
              class="rounded bg-primary px-3 py-1 text-sm text-primary-foreground transition hover:brightness-110"
              @click="goToSettings"
            >
              {{ t('common.goToSettings') }}
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import LoadingSpinner from '../common/LoadingSpinner.vue'
import { useSettingsStore } from '@/stores/settings'
import { languageOptions } from '@/constants/languages'
import { showToast } from '@/lib/toast'
import { useStreamingTranslation } from '@/composables/useStreamingTranslation'
import { Pin, PinOff, Settings, X, Copy, ArrowRight } from 'lucide-vue-next'
import { isTauriEnv } from '@/utils/env'

const { t } = useI18n()

// 定义 ProviderInfo 接口，与 Rust 后端返回的结构对应
interface ProviderInfo {
  name: string
  display_name: string
  config: {
    provider_name: string
    enabled: boolean
    api_key: string
    model: string
    base_url: string | null
  }
  available_models: string[]
  supports_base_url: boolean
}

interface ProviderTranslationResult {
  provider: string
  model: string
  detected_source_lang: string
  target_lang: string
  translation: string
  success: boolean
  error: string | null
}

interface MultiProviderResult {
  results: ProviderTranslationResult[]
}

const mainContainer = ref<HTMLElement | null>(null)
const translationResults = ref<ProviderTranslationResult[]>([])
const sourcePreview = ref('')
const sourceTextarea = ref<HTMLTextAreaElement | null>(null)
// const isLoading = ref(false) // No longer needed, replaced by streamingLoading
const detectedLang = ref('')
const targetLang = ref('')
const pinned = ref(false)
const currentRequestId = ref(0)
const enabledProviders = ref<ProviderInfo[]>([])

const settingsStore = useSettingsStore()
const { streamingResults, isLoading: streamingLoading, error: streamingError, reset, initProviders } = useStreamingTranslation(
  currentRequestId,
  (detected, target) => {
    if (detected) detectedLang.value = detected
    if (target) targetLang.value = target
  },
)

const langLabel = (lang: string) => {
  if (!lang) return '...'
  const found = languageOptions.find((o) => o.value === lang)
  return found ? found.label : lang
}

const detectedLabel = computed(() => langLabel(detectedLang.value))
const targetLabel = computed(() => langLabel(targetLang.value))

const providerKey = (provider: ProviderInfo) => {
  const model = provider.config.model || provider.available_models[0] || 'default'
  return `${provider.name}-${model}`
}

type ProviderCardState = {
  loading: boolean
  translation: string
  success: boolean
  error: string | null
  model: string
}

const providerCardStates = computed(() => {
  const map = new Map<string, ProviderCardState>()

  enabledProviders.value.forEach((provider) => {
    const key = providerKey(provider)
    const model = provider.config.model || provider.available_models[0] || ''
    const baseState: ProviderCardState = {
      loading: false,
      translation: '',
      success: false,
      error: null,
      model,
    }

    const stream = streamingResults.value.get(key)
    if (stream) {
      map.set(key, {
        loading: stream.loading,
        translation: stream.content,
        success: stream.isComplete && !stream.error,
        error: stream.error,
        model: stream.model || model,
      })
      return
    }

    const nonStream = translationResults.value.find(
      (r) => r.provider === provider.name || r.provider === provider.display_name,
    )
    if (nonStream) {
      map.set(key, {
        loading: false,
        translation: nonStream.translation,
        success: nonStream.success,
        error: nonStream.error || null,
        model: nonStream.model || model,
      })
      return
    }

    map.set(key, baseState)
  })

  return map
})

const getProviderCardState = (provider: ProviderInfo) => {
  return providerCardStates.value.get(providerKey(provider)) || {
    loading: false,
    translation: '',
    success: false,
    error: null,
    model: provider.config.model || provider.available_models[0] || '',
  }
}

const showApiKeyPrompt = ref(false)
const useStreaming = ref(true) // Add streaming translation switch

const applyDefaultModel = (provider: ProviderInfo): ProviderInfo => {
  const fallbackModel = provider.config.model || provider.available_models[0] || ''
  return {
    ...provider,
    config: {
      ...provider.config,
      model: fallbackModel,
    },
  }
}

const loadEnabledProviders = async () => {
  if (!isTauriEnv()) return
  try {
    const allProviders = await invoke<ProviderInfo[]>('get_provider_configs')
    enabledProviders.value = allProviders
      .filter((p) => p.config.enabled)
      .map(applyDefaultModel)
    await nextTick()
    updateWindowHeight()
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`获取服务商配置失败：${errMsg}`, 'error')
  }
}

const openSettings = async () => {
  try {
    await invoke('show_settings_window')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`打开设置窗口失败：${errMsg}`, 'error')
  }
}

const goToSettings = async () => {
  showApiKeyPrompt.value = false
  try {
    await invoke('show_settings_window')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`打开设置窗口失败：${errMsg}`, 'error')
  } finally {
    try {
      await invoke('hide_window')
    } catch (_) {}
  }
}

const dismissApiPrompt = async () => {
  showApiKeyPrompt.value = false
  try {
    await invoke('hide_window')
  } catch (_) {}
}

const hideWindow = async () => {
  try {
    await invoke('hide_window')
  } catch (_) {}
}

const copyText = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    showToast(t('translator.copySuccess'), 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('common.copyFailed')}：${errMsg}`, 'error')
  }
}

let unlisten: (() => void) | undefined
let providerConfigUnlisten: (() => void) | undefined
let focusUnlisten: (() => void) | undefined

const startTranslation = async (text: string) => {
  if (!text.trim()) return

  if (streamingLoading.value) { // Use streamingLoading to check for ongoing streaming translations
    // 取消当前进行中的翻译
    try {
      await invoke('cancel_all_translations')
    } catch (_) {}
  }

  // 生成新的请求 ID
  const now = Date.now()
  currentRequestId.value = now

  // Reset states
  translationResults.value = []
  detectedLang.value = ''
  targetLang.value = ''
  
  // 重置流式翻译状态
  reset()

  // isLoading.value = true // Managed by useStreamingTranslation's streamingLoading
  try {
    await invoke('set_main_loading', { loading: true })
  } catch (_) {}
  try {
    pinned.value = await invoke('get_main_pinned')
  } catch (_) {}

  // Get enabled providers and initialize streamingResults
  let activeProviders: ProviderInfo[] = []
  try {
    const allProviders = await invoke<ProviderInfo[]>('get_provider_configs')
    activeProviders = allProviders
      .filter(p => p.config.enabled)
      .map(applyDefaultModel)
    enabledProviders.value = activeProviders

    if (activeProviders.length === 0) {
      showApiKeyPrompt.value = true
      reset()
      try {
        await invoke('set_main_loading', { loading: false })
      } catch (_) {}
      return // No enabled providers, return directly
    }

    // Initialize streamingResults with loading states for each enabled provider
    initProviders(activeProviders.map(p => ({ provider: p.name, model: p.config.model })))
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`获取服务商配置失败：${errMsg}`, 'error')
    return
  }

  try {
    // 使用流式翻译
    if (useStreaming.value) {
      // Call the new translate_multi_stream_individual command
      // This command will trigger parallel streaming translations for multiple providers,
      // and update the frontend via the event system.
      const providerNames = activeProviders.map(p => p.name)
      await invoke('translate_multi_stream_individual', {
        text: text,
        primaryTarget: settingsStore.primaryTarget,
        secondaryTarget: settingsStore.secondaryTarget,
        requestId: now,
        providers: providerNames, // Pass the names of enabled providers
      })

      // We don't wait for a result here, as results are streamed via events to `streamingResults`
      // We only handle potential errors or initial state setup.
    } else {
      // Use traditional translation
      const result = await invoke<MultiProviderResult>('translate_multi', {
        text: text,
        primaryTarget: settingsStore.primaryTarget,
        secondaryTarget: settingsStore.secondaryTarget,
        requestId: now,
      })

      // Check if it's the response for the current request
      if (currentRequestId.value !== now) {
        return
      }

      translationResults.value = result.results
    }
    // Language detection will now be handled by events received in useStreamingTranslation
    // and updated to detectedLang.value and targetLang.value accordingly.
    // The first event carrying this information (likely StreamEvent::Start or Chunk) will trigger the update.

  } catch (error: any) {
    // Check if it's the response for the current request
    if (currentRequestId.value !== now) {
      return
    }
    const errMsg = error?.message || String(error)

    // This check should now be handled when fetching `enabledProviders`, so it might not be needed here.
    if (errMsg.includes('没有已启用的服务商')) {
      showApiKeyPrompt.value = true
    } else {
      showToast(`${t('translator.translationFailed')}：${errMsg}`, 'error')
    }
    try {
      await invoke('set_main_loading', { loading: false })
    } catch (_) {}
  } finally {
    // `set_main_loading` still needs to be called to update the Tauri window's loading state.
    // It should be set to false once all streaming results are complete (managed by streamingLoading in useStreamingTranslation).
    if (currentRequestId.value === now && !streamingLoading.value) { // Only set to false if all streams are complete
         try {
           await invoke('set_main_loading', { loading: false })
         } catch (_) {}
    }
  }
}

onMounted(async () => {
  // Load pinned state
  try {
    pinned.value = await invoke('get_main_pinned')
  } catch (_) {}

  // Load settings
  try {
    const loadedSettings: any = await invoke('get_settings')
    settingsStore.setPrimaryTarget(loadedSettings.primary_target || 'zh-CN')
    settingsStore.setSecondaryTarget(loadedSettings.secondary_target || 'en')
  } catch (_) {}

  await loadEnabledProviders()

  // Listen for provider config changes
  providerConfigUnlisten = await listen('provider-config-changed', async () => {
    // If we were showing the "no provider" prompt, we can dismiss it now
    // assuming the user just enabled a provider.
    if (showApiKeyPrompt.value) {
      showApiKeyPrompt.value = false
    }
    await loadEnabledProviders()
  })

  // When the window regains focus, refresh providers to reflect settings changes made elsewhere
  focusUnlisten = await listen('tauri://focus', async () => {
    await loadEnabledProviders()
  })

  // Listen for main-show event (triggered by double-click copy)
  unlisten = await listen<string>('main-show', async (event) => {
    sourcePreview.value = event.payload || ''
    if (sourcePreview.value) {
      await startTranslation(sourcePreview.value)
    }
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (providerConfigUnlisten) {
    providerConfigUnlisten()
  }
  if (focusUnlisten) {
    focusUnlisten()
  }
})

// Window height auto-resize
let resizeTimer: number | null = null
const updateWindowHeight = async () => {
  await nextTick()
  if (!mainContainer.value) return

  // Calculate the content height needed
  // We use scrollHeight of the container to get the full content height
  const contentHeight = mainContainer.value.scrollHeight
  
  // Add some padding for safety (e.g. borders/shadows)
  // Note: If the content is smaller than min height, backend will handle it.
  const targetHeight = contentHeight + 2

  try {
    await invoke('resize_main_window', { height: targetHeight })
  } catch (_) {}
}

// Watch for content changes to update window height
watch([translationResults, streamingResults, sourcePreview, streamingLoading, enabledProviders], () => {
  if (resizeTimer) {
    window.clearTimeout(resizeTimer)
  }
  resizeTimer = window.setTimeout(() => {
    updateWindowHeight()
    resizeTimer = null
  }, 20)
}, { deep: true })

// Sync tauri loading flag with streaming state
watch(streamingLoading, async (loading) => {
  if (!loading) {
    try {
      await invoke('set_main_loading', { loading: false })
    } catch (_) {}
  }
})

const togglePin = async () => {
  const next = !pinned.value
  pinned.value = next
  try {
    await invoke('set_main_pinned', { pinned: next })
    showToast(next ? t('translator.pinned') : t('translator.unpinned'), 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('common.error')}：${errMsg}`, 'error')
  }
}

const cancelCurrent = async () => {
  if (!streamingLoading.value) return // Only cancel if there are active streams

  try {
    await invoke('cancel_all_translations')
  } catch (_) {}

  // Reset request ID to ignore any ongoing requests
  currentRequestId.value = 0
  // streamingLoading is managed internally by useStreamingTranslation

  try {
    await invoke('set_main_loading', { loading: false })
  } catch (_) {}

  try {
    await invoke('hide_window')
  } catch (_) {}
}

// Auto-resize source textarea within bounds
const autoResizeTextarea = () => {
  const el = sourceTextarea.value
  if (!el) return
  el.style.height = 'auto'
  const lineHeight = parseInt(getComputedStyle(el).lineHeight || '18', 10)
  const maxHeight = lineHeight * 20 // 20 rows
  const nextHeight = Math.min(el.scrollHeight, maxHeight)
  el.style.height = `${nextHeight}px`
}

watch(sourcePreview, () => {
  nextTick(() => autoResizeTextarea())
})

</script>


<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
