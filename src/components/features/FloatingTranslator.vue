<template>
  <div class="relative flex h-full w-full flex-col bg-background text-foreground rounded-xl border shadow-lg">
    <!-- Header -->
    <header class="grid grid-cols-1 gap-2 border-b bg-card px-3 py-2 text-sm shadow-sm">
      <div class="flex items-center justify-between">
        <span class="text-xs text-muted-foreground">检测语言</span>
        <span class="rounded-md border bg-muted/40 px-2 py-1 text-xs">{{ detectedLabel }}</span>
      </div>
      <div class="flex items-center justify-between">
        <span class="text-xs text-muted-foreground">目标列表</span>
        <div class="flex flex-wrap justify-end gap-1">
          <span
            v-for="tag in targetListDisplay"
            :key="tag.value"
            class="inline-flex items-center rounded-full border bg-accent/60 px-2 py-1 text-[12px] text-accent-foreground"
          >
            {{ tag.label }}
          </span>
        </div>
      </div>
    </header>

    <!-- 原文 -->
    <section v-if="sourcePreview" class="shrink-0 border-b bg-muted/30 px-3 py-2">
      <p class="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">原文</p>
      <div class="max-h-[200px] overflow-auto rounded-md border bg-background/80 p-2 text-sm leading-relaxed shadow-inner">
        <p class="whitespace-pre-wrap break-words text-foreground/90">
          {{ sourcePreview }}
        </p>
      </div>
    </section>

    <!-- 结果列表 -->
    <section class="min-h-0 flex-1 px-3 py-2">
      <div class="space-y-2">
        <div
          v-if="isLoading"
          class="flex flex-col items-center justify-center gap-2 rounded-md border border-dashed bg-muted/30 px-3 py-4"
        >
          <LoadingSpinner />
          <button
            class="inline-flex items-center gap-1 rounded border bg-muted/60 px-2 py-1 text-xs text-muted-foreground transition hover:bg-accent hover:text-accent-foreground"
            @click="cancelCurrent"
          >
            取消
          </button>
        </div>
        <template v-else>
          <div
            v-for="item in orderedTranslations"
            :key="item.lang"
            class="rounded-lg border bg-card px-3 py-2 shadow-sm"
          >
            <div class="mb-1 flex items-center justify-between text-xs font-semibold text-muted-foreground">
              <span>{{ langLabel(item.lang) }}</span>
              <button
                class="inline-flex items-center gap-1 rounded border bg-muted/60 px-2 py-1 text-[11px] text-muted-foreground transition hover:bg-accent hover:text-accent-foreground"
                @click="copyText(item.text)"
                :title="`复制 ${item.lang} 内容`"
              >
                ⧉ 复制
              </button>
            </div>
            <div class="max-h-[200px] overflow-auto rounded-md border border-dashed bg-background/70 p-2">
              <p class="text-sm leading-relaxed text-foreground/90 whitespace-pre-wrap break-words">
                {{ item.text }}
              </p>
            </div>
          </div>
          <p v-if="!orderedTranslations.length" class="py-6 text-center text-sm text-muted-foreground">等待翻译中...</p>
        </template>
      </div>
    </section>

    <!-- Pin button -->
    <div class="pointer-events-none absolute inset-0">
      <div class="pointer-events-auto fixed bottom-3 right-3">
        <button
          class="inline-flex items-center justify-center rounded-full border border-red-500 bg-red-500 text-white p-2 shadow-lg transition hover:bg-red-600"
          @click="togglePin"
        >
          <component :is="pinned ? PinOff : Pin" class="h-4 w-4" />
        </button>
      </div>
    </div>

    <!-- 全屏 Loading 遮罩 (避免失焦关闭) -->
    <transition name="fade">
      <div
        v-if="isLoading"
        class="pointer-events-auto fixed inset-0 z-10 flex items-center justify-center bg-background/70 backdrop-blur"
      >
        <div class="flex flex-col items-center gap-3 rounded-lg border bg-card px-4 py-3 shadow-lg">
          <LoadingSpinner />
          <div class="flex gap-2">
            <button
              class="inline-flex items-center gap-1 rounded border bg-muted/60 px-3 py-1 text-xs text-muted-foreground transition hover:bg-accent hover:text-accent-foreground"
              @click="cancelCurrent"
            >
              取消
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- API Key 提示对话框 -->
    <transition name="fade">
      <div
        v-if="showApiKeyPrompt"
        class="fixed inset-0 z-50 flex items-center justify-center bg-background/70 backdrop-blur"
      >
        <div class="w-[320px] rounded-lg border bg-card p-4 shadow-lg">
          <h3 class="text-base font-semibold text-foreground">需要设置 API Key</h3>
          <p class="mt-2 text-sm text-muted-foreground">请先前往设置页面填写并保存 API Key 后再翻译。</p>
          <div class="mt-4 flex justify-end gap-2">
            <button
              class="rounded border px-3 py-1 text-sm text-muted-foreground transition hover:bg-muted/60"
              @click="dismissApiPrompt"
            >
              稍后
            </button>
            <button
              class="rounded bg-primary px-3 py-1 text-sm text-primary-foreground transition hover:brightness-110"
              @click="goToSettings"
            >
              前往设置
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import LoadingSpinner from '../common/LoadingSpinner.vue'
import { useSettingsStore } from '@/stores/settings'
import { defaultCommonTargets, languageOptions } from '@/constants/languages'
import { showToast } from '@/lib/toast'
import { Pin, PinOff } from 'lucide-vue-next'

const translations = ref<Record<string, string>>({})
const sourcePreview = ref('')
const isLoading = ref(false)
const detectedLang = ref('')
const pinned = ref(false)
const requestId = ref(0)

const settingsStore = useSettingsStore()

const targetList = () => {
  const list = settingsStore.commonTargetLanguages && settingsStore.commonTargetLanguages.length > 0
    ? settingsStore.commonTargetLanguages
    : defaultCommonTargets
  return Array.from(new Set(list)).slice(0, 5)
}

const filteredTranslations = computed(() => {
  const res: Record<string, string> = {}
  Object.entries(translations.value).forEach(([lang, text]) => {
    if (lang === detectedLang.value) return
    res[lang] = text
  })
  return res
})

const orderedTranslations = computed(() => {
  const targetOrder = targetList()
  const entries = Object.entries(filteredTranslations.value).map(([lang, text]) => ({ lang, text }))
  return entries.sort((a, b) => {
    const ia = targetOrder.indexOf(a.lang)
    const ib = targetOrder.indexOf(b.lang)
    return (ia === -1 ? 999 : ia) - (ib === -1 ? 999 : ib)
  })
})

const targetListDisplay = computed(() => {
  const list = targetList()
  return list.map((val) => {
    const found = languageOptions.find((o) => o.value === val)
    return { value: val, label: found ? found.label : val }
  })
})

const langLabel = (lang: string) => {
  const found = languageOptions.find((o) => o.value === lang)
  return found ? `${found.label} (${found.value})` : lang
}

const detectedLabel = computed(() => {
  if (!detectedLang.value) return '...'
  return langLabel(detectedLang.value)
})

const showApiKeyPrompt = ref(false)

const ensureApiKey = async () => {
  if (settingsStore.apiKey?.trim()) return true
  showToast('请先在设置中配置 API Key', 'error')
  showApiKeyPrompt.value = true
  return false
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

const copyText = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    showToast('已复制翻译内容', 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`复制失败：${errMsg}`, 'error')
  }
}

let unlisten: (() => void) | undefined;

onMounted(async () => {
  unlisten = await listen<string>('floating-show', async (event) => {
    if (isLoading.value) {
      showToast('正在翻译，请先等待或取消当前任务', 'info')
      return
    }
    if (!(await ensureApiKey())) {
      return
    }

    requestId.value += 1
    const thisReq = requestId.value

    sourcePreview.value = event.payload || ''
    translations.value = {}
    detectedLang.value = ''
    isLoading.value = true
    try {
      await invoke('set_floating_loading', { loading: true })
    } catch (_) {}
    try {
      pinned.value = await invoke('get_floating_pinned')
    } catch (_) {}

    try {
      const targets = targetList()
      const result: any = await invoke('translate_text', {
        text: event.payload,
        targetLangs: targets,
        requestId: thisReq,
      })
      if (thisReq !== requestId.value) {
        return
      }
      detectedLang.value =
        result?.detected_source_lang ||
        result?.detectedLang ||
        result?.detected_language ||
        ''
      translations.value = (result?.translations as Record<string, string>) || {}
    } catch (error: any) {
      if (thisReq !== requestId.value) {
        return
      }
      const errMsg = error?.message || String(error)
      translations.value = { error: `翻译失败：${errMsg}` }
    } finally {
      if (thisReq === requestId.value) {
        isLoading.value = false
        try {
          await invoke('set_floating_loading', { loading: false })
        } catch (_) {}
      }
    }
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
})

const togglePin = async () => {
  const next = !pinned.value
  pinned.value = next
  try {
    await invoke('set_floating_pinned', { pinned: next })
    showToast(next ? '已固定悬浮窗' : '已取消固定', 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`切换固定状态失败：${errMsg}`, 'error')
  }
}

const cancelCurrent = async () => {
  if (!isLoading.value) return
  requestId.value += 1
  isLoading.value = false
  try {
    await invoke('set_floating_loading', { loading: false })
  } catch (_) {}
  
  try {
    await invoke('hide_window')
  } catch (_) {}
}
</script>
