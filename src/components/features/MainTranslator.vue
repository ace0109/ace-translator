<template>
  <Card ref="mainContainer"
    class="relative flex w-full flex-col overflow-hidden rounded-xl border bg-background text-foreground shadow-lg">
    <CardHeader data-tauri-drag-region
      class="h-10 flex-row items-center justify-between gap-2 space-y-0 border-b bg-card/70 px-3 py-2">
      <CardTitle data-tauri-drag-region class="text-xs font-semibold text-muted-foreground">
        {{ t('translator.title') }}
      </CardTitle>
      <CardAction class="flex items-center gap-1">
        <Button variant="ghost" size="icon" class="h-7 w-7 text-muted-foreground" :class="{ 'text-primary': pinned }"
          @click="togglePin" :title="pinned ? t('translator.unpinWindow') : t('translator.pinWindow')">
          <component :is="pinned ? PinOff : Pin" class="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" class="h-7 w-7 text-muted-foreground" @click="openSettings"
          :title="t('translator.openSettings')">
          <Settings class="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" class="h-7 w-7 text-muted-foreground hover:text-destructive"
          @click="hideWindow" :title="t('translator.closeWindow')">
          <X class="h-4 w-4" />
        </Button>
      </CardAction>
    </CardHeader>

    <CardContent class="flex flex-col gap-3 p-4">
      <div class="flex items-center justify-between rounded-lg border bg-muted/40 px-3 py-2 text-xs">
        <div class="flex items-center gap-2">
          <span class="text-muted-foreground">{{ t('translator.detected') }}</span>
          <Badge variant="secondary">{{ detectedLabel }}</Badge>
        </div>
        <ArrowRight class="h-3.5 w-3.5 text-muted-foreground" />
        <div class="flex items-center gap-2">
          <span class="text-muted-foreground">{{ t('translator.target') }}</span>
          <Badge>{{ targetLabel }}</Badge>
        </div>
      </div>

      <div class="space-y-2 rounded-lg border bg-card/60 p-3">
        <div class="flex items-center justify-between gap-2">
          <Label class="text-[14px] uppercase tracking-wide text-muted-foreground">
            {{ t('translator.original') }}
          </Label>
        </div>
        <Textarea v-model="sourcePreview" ref="sourceTextarea" :placeholder="t('translator.inputPlaceholder')"
          class="min-h-20 max-h-80 resize-none bg-background/80" @input="autoResizeTextarea"
          @keydown.enter.exact.prevent="startTranslation(sourcePreview)"
          @keydown.enter.ctrl.exact.prevent="startTranslation(sourcePreview)"
          @keydown.enter.meta.exact.prevent="startTranslation(sourcePreview)" />
        <div class="flex justify-end gap-2 pt-1">
          <Button size="sm" :disabled="!sourcePreview.trim()" @click="startTranslation(sourcePreview)">
            {{ t('translator.translateBtn') }}
          </Button>
        </div>
      </div>

      <div class="space-y-2">
        <Alert v-if="streamingError" variant="destructive">
          <AlertTitle>{{ t('translator.translationFailed') }}</AlertTitle>
          <AlertDescription>{{ streamingError }}</AlertDescription>
        </Alert>

        <div v-if="enabledProviders.length > 0 || streamingLoading" class="space-y-2">
          <Card v-for="provider in enabledProviders" :key="providerKey(provider)"
            class="overflow-hidden border shadow-sm">
            <CardHeader class="flex flex-row items-center justify-between gap-2 border-b bg-muted/30 py-2">
              <div class="flex items-center gap-2">
                <span class="h-2 w-2 rounded-full" :class="statusDot(provider)" />
                <div class="space-y-0.5">
                  <p class="text-sm font-medium leading-none">{{ provider.display_name }}</p>
                  <p class="text-[11px] text-muted-foreground">
                    {{ getProviderCardState(provider).model || provider.config.model }}
                  </p>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <Badge variant="outline" class="text-[10px] font-medium">
                  {{ provider.config.base_url ? 'Custom' : 'Default' }}
                </Badge>
                <Button v-if="getProviderCardState(provider).success && !getProviderCardState(provider).loading"
                  variant="ghost" size="sm" class="h-8 px-2 text-xs"
                  @click="copyText(getProviderCardState(provider).translation)" :title="t('common.copy')">
                  <Copy class="h-4 w-4" />
                  <span class="ml-1">{{ t('common.copy') }}</span>
                </Button>
              </div>
            </CardHeader>

            <CardContent class="space-y-2 py-3">
              <Alert v-if="getProviderCardState(provider).error" variant="destructive" class="py-2">
                <AlertDescription>
                  {{ getProviderCardState(provider).error || t('translator.translationFailed') }}
                </AlertDescription>
              </Alert>
              <div v-else class="relative rounded-md border border-dashed bg-background/70 p-3">
                <p class="whitespace-pre-wrap break-words text-sm leading-relaxed text-foreground/90">
                  <span v-if="getProviderCardState(provider).loading" class="inline-flex align-middle">
                    <LoadingSpinner />
                  </span>
                  {{
                    getProviderCardState(provider).translation
                  }}
                </p>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </CardContent>

    <Dialog :open="showApiKeyPrompt" @update:open="(open) => (showApiKeyPrompt = open)">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{{ t('translator.noProvider') }}</DialogTitle>
          <DialogDescription>{{ t('translator.noProviderDesc') }}</DialogDescription>
        </DialogHeader>
        <DialogFooter class="gap-2">
          <Button variant="outline" @click="dismissApiPrompt">
            {{ t('common.later') }}
          </Button>
          <Button @click="goToSettings">
            {{ t('common.goToSettings') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </Card>
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
import { Button } from '@/components/ui/button'
import { Card, CardAction, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
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

type ElementRef<T extends HTMLElement> = T | { $el?: T }

const mainContainer = ref<ElementRef<HTMLElement> | null>(null)
const translationResults = ref<ProviderTranslationResult[]>([])
const sourcePreview = ref('')
const sourceTextarea = ref<ElementRef<HTMLTextAreaElement> | null>(null)
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

const resolveEl = <T extends HTMLElement>(el: ElementRef<T> | null) => {
  if (!el) return null
  if (el instanceof HTMLElement) return el
  return el.$el instanceof HTMLElement ? el.$el : null
}

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
        // 只要已有内容就视为不再 loading
        loading: stream.loading && !stream.content,
        translation: stream.content,
        success: stream.isComplete && !stream.error,
        error: stream.error || null,
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

const statusDot = (provider: ProviderInfo) => {
  const state = getProviderCardState(provider)
  if (state.success && !state.loading) return 'bg-green-500'
  if (state.error && !state.loading) return 'bg-red-500'
  if (state.loading) return 'bg-gray-400 animate-pulse'
  return 'bg-muted-foreground'
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
    } catch (_) { }
  }
}

const dismissApiPrompt = async () => {
  showApiKeyPrompt.value = false
  try {
    await invoke('hide_window')
  } catch (_) { }
}

const hideWindow = async () => {
  try {
    await invoke('hide_window')
  } catch (_) { }
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
    } catch (_) { }
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
  } catch (_) { }
  try {
    pinned.value = await invoke('get_main_pinned')
  } catch (_) { }

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
      } catch (_) { }
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
    } catch (_) { }
  } finally {
    // `set_main_loading` still needs to be called to update the Tauri window's loading state.
    // It should be set to false once all streaming results are complete (managed by streamingLoading in useStreamingTranslation).
    if (currentRequestId.value === now && !streamingLoading.value) { // Only set to false if all streams are complete
      try {
        await invoke('set_main_loading', { loading: false })
      } catch (_) { }
    }
  }
}

onMounted(async () => {
  // Load pinned state
  try {
    pinned.value = await invoke('get_main_pinned')
  } catch (_) { }

  // Load settings
  try {
    const loadedSettings: any = await invoke('get_settings')
    settingsStore.setPrimaryTarget(loadedSettings.primary_target || 'zh-CN')
    settingsStore.setSecondaryTarget(loadedSettings.secondary_target || 'en')
  } catch (_) { }

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
  const containerEl = resolveEl(mainContainer.value)
  if (!containerEl) return

  // Calculate the content height needed using the rendered card element
  const contentHeight = containerEl.scrollHeight

  // Add some padding for safety (e.g. borders/shadows)
  // Note: If the content is smaller than min height, backend will handle it.
  const targetHeight = contentHeight + 2

  try {
    await invoke('resize_main_window', { height: targetHeight })
  } catch (_) { }
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
    } catch (_) { }
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

// Auto-resize source textarea within bounds
const autoResizeTextarea = () => {
  const el = resolveEl(sourceTextarea.value)
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
