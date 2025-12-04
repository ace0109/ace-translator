<template>
  <div class="flex min-h-screen flex-col gap-4 p-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 class="text-xl font-semibold leading-tight">Ace Translator</h1>
      </div>
      <Button variant="outline" class="gap-2" @click="openSettingsWindow">
        <Settings2 class="h-4 w-4" />
        打开设置窗口
      </Button>
    </div>

    <div class="grid flex-1 grid-cols-2 items-stretch gap-4">
      <Card class="flex h-full flex-col">
        <CardHeader class="pb-2">
          <div class="flex items-center justify-between gap-3">
            <CardTitle class="text-base">源语言</CardTitle>
            <div class="flex h-10 items-center rounded-md border px-3 text-sm text-muted-foreground">
              {{ langDisplay }}
            </div>
          </div>
          <CardDescription>输入或粘贴需要翻译的文本</CardDescription>
        </CardHeader>
        <CardContent class="flex-1">
          <Textarea
            v-model="sourceText"
            placeholder="输入要翻译的内容，支持快捷粘贴。"
            class="h-full min-h-[260px]"
            @keydown.enter.exact.prevent="handleTranslate"
          />
        </CardContent>
      </Card>

      <Card class="flex h-full flex-col">
        <CardHeader class="pb-2">
          <div class="grid grid-cols-2 items-center gap-3">
            <CardTitle class="text-base">目标语言</CardTitle>
            <LanguageSelector
              v-model="targetLang"
              placeholder="选择目标语言"
              class="w-full"
            />
          </div>
          <CardDescription>翻译结果</CardDescription>
        </CardHeader>
        <CardContent class="flex-1 space-y-2">
          <Textarea
            :model-value="displayedTranslation"
            placeholder="翻译结果将显示在这里"
            class="h-full min-h-[260px]"
            readonly
          />
        </CardContent>
      </Card>
    </div>

    <div class="flex flex-wrap items-center justify-center gap-3">
      <Button class="gap-2" :disabled="isLoading" @click="handleTranslate">
        <Loader2 v-if="isLoading" class="h-4 w-4 animate-spin" />
        <Languages v-else class="h-4 w-4" />
        <span>{{ isLoading ? '正在翻译...' : '开始翻译' }}</span>
      </Button>
      <Button variant="outline" class="gap-2" :disabled="isLoading" @click="clearText">
        <Trash2 class="h-4 w-4" />
        清空
      </Button>
      <Button
        v-if="displayedTranslation"
        variant="secondary"
        class="gap-2"
        :disabled="isLoading"
        @click="copyTranslated"
      >
        <Copy class="h-4 w-4" />
        复制当前结果
      </Button>
    </div>
    <transition name="fade">
      <div
        v-if="isLoading"
        class="pointer-events-auto fixed inset-0 z-40 flex items-center justify-center bg-background/70 backdrop-blur-sm"
      >
        <div class="flex items-center gap-3 rounded-lg border bg-card px-4 py-3 shadow-lg">
          <Loader2 class="h-5 w-5 animate-spin text-muted-foreground" />
          <span class="text-sm text-foreground">正在翻译，请稍候...</span>
        </div>
      </div>
    </transition>
    <!-- API Key 提示对话框 -->
    <transition name="fade">
      <div
        v-if="showApiKeyPrompt"
        class="fixed inset-0 z-50 flex items-center justify-center bg-background/70 backdrop-blur-sm"
      >
        <div class="w-[320px] rounded-lg border bg-card p-4 shadow-lg">
          <h3 class="text-base font-semibold text-foreground">需要设置 API Key</h3>
          <p class="mt-2 text-sm text-muted-foreground">请先前往设置页面填写并保存 API Key 后再进行翻译。</p>
          <div class="mt-4 flex justify-end gap-2">
            <Button variant="outline" @click="showApiKeyPrompt = false">稍后</Button>
            <Button @click="goToSettings">前往设置</Button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { storeToRefs } from 'pinia'
import { Copy, Languages, Loader2, Settings2, Trash2 } from 'lucide-vue-next'
import { useTranslationStore } from '@/stores/translation'
import { useSettingsStore } from '@/stores/settings'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Textarea } from '@/components/ui/textarea'
import LanguageSelector from '../common/LanguageSelector.vue'
import { showToast } from '@/lib/toast'
import { defaultCommonTargets, languageOptions } from '@/constants/languages'

const translationStore = useTranslationStore()
const settingsStore = useSettingsStore()

const { sourceText, targetLang, isLoading, detectedLang, translations } =
  storeToRefs(translationStore)

const displayedTranslation = computed(() => translations.value[targetLang.value] || '')
const langDisplay = computed(() => {
  if (!detectedLang.value) return '自动检测'
  const found = languageOptions.find((o) => o.value === detectedLang.value)
  return found ? found.label : detectedLang.value
})

const showApiKeyPrompt = ref(false)

const ensureCommonTargets = () => {
  const list = settingsStore.commonTargetLanguages && settingsStore.commonTargetLanguages.length > 0
    ? settingsStore.commonTargetLanguages
    : defaultCommonTargets
  return Array.from(new Set(list)).slice(0, 5)
}

const goToSettings = () => {
  showApiKeyPrompt.value = false
  openSettingsWindow()
}

const callTranslate = async (targets: string[]) => {
  if (!settingsStore.apiKey?.trim()) {
    showToast('请先前往设置中配置 API Key', 'error')
    showApiKeyPrompt.value = true
    return null
  }
  if (!sourceText.value.trim()) {
    showToast('请输入要翻译的文本', 'error')
    return null
  }
  isLoading.value = true
  try {
    const response: any = await invoke('translate_text', {
      text: sourceText.value,
      targetLangs: targets,
    })
    return response
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`翻译失败：${errMsg}`, 'error')
    return null
  } finally {
    isLoading.value = false
  }
}

const handleTranslate = async () => {
  const targets = Array.from(new Set([...ensureCommonTargets(), targetLang.value])).slice(0, 5)
  const result = await callTranslate(targets)
  if (!result) return

  const { detected, map } = normalizeResult(result)
  translationStore.setDetectedLang(detected)
  translationStore.setTranslations(map)

  // 如果检测语言与当前目标相同，自动切换
  if (detected && detected === targetLang.value) {
    const fallback = detected.startsWith('zh') ? 'en' : 'zh-CN'
    targetLang.value = fallback
    if (!map[fallback]) {
      const extra = await callTranslate([fallback])
      if (extra?.translations) {
        translationStore.mergeTranslations(extra.translations as Record<string, string>)
      }
    }
    showToast(`检测到源语言与目标相同，已切换为 ${targetLang.value}`, 'info')
  }
}

// 切换目标语言时，如已有缓存直接展示，否则请求单语言翻译并合并
watch(
  () => targetLang.value,
  async (newLang) => {
    if (!sourceText.value.trim()) return
    if (translations.value[newLang]) return
    const result = await callTranslate([newLang])
    const { map } = normalizeResult(result)
    if (Object.keys(map).length) {
      translationStore.mergeTranslations(map)
    }
  },
)

const clearText = () => {
  sourceText.value = ''
  translationStore.clearTranslations()
}

watch(
  () => sourceText.value,
  (val, old) => {
    if (val !== old) {
      translationStore.clearTranslations()
    }
  },
)

const copyTranslated = async () => {
  if (!displayedTranslation.value) return
  try {
    await navigator.clipboard.writeText(displayedTranslation.value)
    showToast('已复制到剪贴板', 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`复制失败：${errMsg}`, 'error')
  }
}

const openSettingsWindow = async () => {
  try {
    await invoke('show_settings_window')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`打开设置窗口失败：${errMsg}`, 'error')
  }
}

const normalizeResult = (result: any) => {
  if (!result) return { detected: '', map: {} as Record<string, string> }
  const detected =
    result.detected_source_lang ||
    result.detectedLang ||
    result.detected_language ||
    ''
  const map: Record<string, string> = {}
  if (result.translations && typeof result.translations === 'object') {
    Object.assign(map, result.translations as Record<string, string>)
  } else if (result.translation && targetLang.value) {
    map[targetLang.value] = result.translation as string
  }
  return { detected, map }
}
</script>
