<template>
  <div class="flex min-h-screen items-center justify-center bg-background text-foreground">
    <Card class="w-[360px]">
      <CardHeader class="space-y-2">
        <CardTitle class="text-lg">{{ t('about.title') }}</CardTitle>
        <CardDescription>{{ t('about.subtitle') }}</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="flex items-center justify-between rounded-md border bg-muted/40 px-3 py-2">
          <span class="text-sm text-muted-foreground">{{ t('about.version') }}</span>
          <button class="text-sm font-medium hover:text-primary" @click="onVersionTap">
            {{ appInfo.version }}
          </button>
        </div>
        <div class="flex items-center justify-between rounded-md border bg-muted/40 px-3 py-2">
          <span class="text-sm text-muted-foreground">{{ t('about.devMode') }}</span>
          <Badge :variant="appInfo.devMode ? 'default' : 'outline'">
            {{ appInfo.devMode ? t('about.enabled') : t('about.disabled') }}
          </Badge>
        </div>
        <p class="text-xs text-muted-foreground">
          {{ t('about.devModeHint') }}
        </p>
      </CardContent>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { showToast } from '@/lib/toast'

const { t } = useI18n()

const tapCount = ref(0)
const appInfo = reactive({
  version: '',
  devMode: false,
})

async function loadInfo() {
  try {
    const res = await invoke<{ version: string; dev_mode: boolean }>('get_app_info')
    appInfo.version = res.version
    appInfo.devMode = res.dev_mode
  } catch (e) {
    console.error('获取应用信息失败', e)
  }
}

async function enableDevMode() {
  try {
    await invoke('enable_dev_mode')
    appInfo.devMode = true
    showToast(t('about.devModeUnlocked'), 'info')
  } catch (e) {
    console.error('启用开发者模式失败', e)
  }
}

function onVersionTap() {
  if (appInfo.devMode) return
  tapCount.value += 1
  if (tapCount.value >= 7) {
    tapCount.value = 0
    enableDevMode()
  }
}

onMounted(() => {
  loadInfo()
})
</script>
