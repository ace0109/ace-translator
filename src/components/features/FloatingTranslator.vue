<template>
  <div class="flex h-full w-full flex-col bg-background text-foreground">
    <!-- Header: 检测语言 & 目标列表 -->
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

    <!-- 翻译结果列表 -->
    <section class="min-h-0 flex-1 px-3 py-2">
      <div class="space-y-2">
        <div
          v-if="isLoading"
          class="flex items-center justify-center rounded-md border border-dashed bg-muted/30 px-3 py-4"
        >
          <LoadingSpinner />
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

const translations = ref<Record<string, string>>({})
const sourcePreview = ref('')
const isLoading = ref(false)
const detectedLang = ref('')

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
    sourcePreview.value = event.payload || ''
    translations.value = {}
    detectedLang.value = ''
    isLoading.value = true

    try {
      const targets = targetList()
      const result: any = await invoke('translate_text', {
        text: event.payload,
        targetLangs: targets,
      })
      detectedLang.value =
        result?.detected_source_lang ||
        result?.detectedLang ||
        result?.detected_language ||
        ''
      translations.value = (result?.translations as Record<string, string>) || {}
    } catch (error: any) {
      const errMsg = error?.message || String(error)
      translations.value = { error: `翻译失败：${errMsg}` }
    } finally {
      isLoading.value = false
    }
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
})
</script>
