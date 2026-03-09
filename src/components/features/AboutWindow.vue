<template>
  <div class="flex min-h-screen items-center justify-center bg-background text-foreground">
    <div v-if="isLoading" class="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
      <div class="flex flex-col items-center gap-4">
        <Loader2 class="h-8 w-8 animate-spin text-primary" />
        <p class="text-sm text-muted-foreground">{{ t('about.loading') }}</p>
      </div>
    </div>

    <Dialog v-model:open="showErrorDialog">
      <DialogContent class="max-w-md">
        <DialogHeader>
          <DialogTitle class="text-destructive">{{ t('about.loadFailed') }}</DialogTitle>
          <DialogDescription>
            {{ errorMessage }}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="showErrorDialog = false">
            {{ t('common.cancel') }}
          </Button>
          <Button @click="retryInit">
            {{ t('common.retry') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <Card ref="aboutContainer" class="w-full rounded-none" :class="{ 'opacity-50': isLoading }">
      <CardHeader>
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
        <div v-if="appInfo.devMode" class="flex items-center justify-between rounded-md border bg-muted/40 px-3 py-2">
          <span class="text-sm text-muted-foreground">{{ t('about.devMode') }}</span>
          <Badge :variant="appInfo.devMode ? 'default' : 'outline'">
            {{ appInfo.devMode ? t('about.enabled') : t('about.disabled') }}
          </Badge>
        </div>

        <div class="space-y-3 border-t pt-2">
          <div>
            <p class="text-sm font-medium">{{ t('about.links.title') }}</p>
            <p class="mt-1 text-xs text-muted-foreground">{{ t('about.links.description') }}</p>
          </div>
          <div class="flex flex-col gap-2">
            <Button class="w-full" @click="openRepository">
              <Github class="mr-2 h-4 w-4" />
              {{ t('about.links.openRepository') }}
            </Button>
            <Button class="w-full" variant="outline" @click="openReleases">
              <ExternalLink class="mr-2 h-4 w-4" />
              {{ t('about.links.openReleases') }}
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { showToast } from '@/lib/toast'
import { isTauriEnv } from '@/utils/env'
import { ExternalLink, Github, Loader2 } from 'lucide-vue-next'

const { t } = useI18n()
const GITHUB_REPOSITORY_URL = 'https://github.com/ace0109/ace-translator'
const GITHUB_RELEASES_URL = `${GITHUB_REPOSITORY_URL}/releases`

const tapCount = ref(0)
const appInfo = reactive({
  version: '',
  devMode: false,
})
const isLoading = ref(false)
const showErrorDialog = ref(false)
const errorMessage = ref('')

type ElementRef<T extends HTMLElement> = T | { $el?: T }
const resolveEl = <T extends HTMLElement>(el: ElementRef<T> | null) => {
  if (!el) return null
  // Vue component instance may expose root element as $el
  if (typeof el === 'object' && '$el' in el) {
    return (el as any).$el as T
  }
  return el as T
}

const aboutContainer = ref<ElementRef<HTMLElement> | null>(null)

// Window height auto-resize (same behavior as MainTranslator)
let resizeTimer: number | null = null
const updateWindowHeight = async () => {
  if (!isTauriEnv()) return
  await nextTick()
  const containerEl = resolveEl(aboutContainer.value)
  if (!containerEl) return

  const contentHeight = containerEl.scrollHeight
  const targetHeight = contentHeight + 2

  try {
    await invoke('resize_about_window', { height: targetHeight })
  } catch (_) { }
}

const scheduleWindowResize = () => {
  if (resizeTimer) {
    window.clearTimeout(resizeTimer)
  }
  resizeTimer = window.setTimeout(() => {
    updateWindowHeight()
    resizeTimer = null
  }, 20)
}

async function loadInfo() {
  try {
    const res = await invoke<{ version: string; dev_mode: boolean }>('get_app_info')
    appInfo.version = res.version
    appInfo.devMode = res.dev_mode
  } catch (e) {
    throw new Error(`获取应用信息失败: ${e}`)
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

async function openExternal(url: string) {
  try {
    if (isTauriEnv()) {
      await openUrl(url)
      return
    }
    window.open(url, '_blank', 'noopener,noreferrer')
  } catch (e) {
    console.error('打开外部链接失败', e)
    showToast(t('about.links.openFailed'), 'error')
  }
}

function openRepository() {
  return openExternal(GITHUB_REPOSITORY_URL)
}

function openReleases() {
  return openExternal(GITHUB_RELEASES_URL)
}

function onVersionTap() {
  if (appInfo.devMode) return
  tapCount.value += 1
  if (tapCount.value >= 7) {
    tapCount.value = 0
    enableDevMode()
  }
}

const init = async () => {
  if (!isTauriEnv()) return

  isLoading.value = true
  try {
    await loadInfo()
  } catch (error: any) {
    errorMessage.value = error.message || String(error)
    showErrorDialog.value = true
  } finally {
    isLoading.value = false
    scheduleWindowResize()
  }
}

const retryInit = async () => {
  showErrorDialog.value = false
  await init()
}

onMounted(() => {
  // 不要在 onMounted 时加载，等待窗口显示或聚焦时再加载
  scheduleWindowResize()

  // 监听窗口显示和聚焦事件
  let hasInitialized = false

  // 监听窗口显示事件
  window.addEventListener('DOMContentLoaded', () => {
    if (!hasInitialized) {
      setTimeout(() => init(), 100)
      hasInitialized = true
    }
  })

  // 监听窗口获得焦点
  window.addEventListener('focus', () => {
    if (!hasInitialized) {
      init()
      hasInitialized = true
    }
  })

  // 监听窗口可见性变化
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden && !hasInitialized) {
      init()
      hasInitialized = true
    }
  })
})

watch(appInfo, () => {
  scheduleWindowResize()
}, { deep: true })

onUnmounted(() => {
  if (resizeTimer) {
    window.clearTimeout(resizeTimer)
    resizeTimer = null
  }
})
</script>
