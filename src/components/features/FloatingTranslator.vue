<template>
  <div class="flex h-full w-full items-center justify-center bg-background text-foreground">
    <Card class="relative w-full max-w-xl overflow-hidden border bg-gradient-to-br from-background via-background to-muted/40 shadow-lg">
      <CardContent class="space-y-4 pt-6">
        <div class="rounded-md border bg-muted/40 px-3 py-4">
          <div class="flex items-center justify-between text-xs text-muted-foreground">
            <span>检测语言：{{ detectedLang || '...' }}</span>
            <span>目标：{{ targetListDisplay }}</span>
          </div>
          <div class="mt-3 grid gap-2">
            <div
              v-for="(text, lang) in filteredTranslations"
              :key="lang"
              class="rounded-md border border-dashed bg-background/80 px-3 py-2"
            >
              <div class="mb-1 text-xs font-semibold text-muted-foreground">{{ lang }}</div>
              <p class="text-sm leading-relaxed text-foreground/90">{{ text }}</p>
            </div>
            <div
              v-if="isLoading"
              class="flex items-center justify-center rounded-md border border-dashed bg-background/60 px-3 py-4"
            >
              <LoadingSpinner />
            </div>
            <p v-if="!isLoading && !Object.keys(filteredTranslations).length" class="text-sm text-muted-foreground">
              等待翻译中...
            </p>
          </div>
        </div>
        <div v-if="sourcePreview" class="rounded-md border bg-muted/30 p-3 text-left">
          <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">原文</p>
          <p class="mt-1 text-sm text-foreground/85">{{ sourcePreview }}</p>
        </div>
      </CardContent>
      <div class="absolute right-3 top-3">
        <Button
          variant="secondary"
          size="icon"
          class="h-8 w-8 rounded-full shadow-sm"
          :disabled="isLoading"
          @click="closeFloatingWindow"
        >
          <X class="h-4 w-4" />
        </Button>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { X } from 'lucide-vue-next'
import LoadingSpinner from '../common/LoadingSpinner.vue'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useSettingsStore } from '@/stores/settings'
import { defaultCommonTargets } from '@/constants/languages'

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

const targetListDisplay = computed(() => targetList().join(', '))

let unlisten: (() => void) | undefined;

onMounted(async () => {
  unlisten = await listen<string>('floating-show', async (event) => {
    sourcePreview.value = event.payload?.slice(0, 120) || ''
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

const closeFloatingWindow = async () => {
  await invoke('hide_window')
}
</script>
