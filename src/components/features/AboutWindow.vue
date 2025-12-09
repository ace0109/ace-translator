<template>
  <div class="flex min-h-screen items-center justify-center bg-background text-foreground">
    <Card class="w-full min-h-screen rounded-none">
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

        <!-- Check for updates -->
        <div class="pt-2 border-t">
          <Button class="w-full" :disabled="updateState.checking" @click="checkForUpdates">
            <Loader2 v-if="updateState.checking" class="mr-2 h-4 w-4 animate-spin" />
            <RefreshCw v-else class="mr-2 h-4 w-4" />
            {{ updateState.checking ? t('about.update.checking') : t('about.update.checkForUpdates') }}
          </Button>

          <!-- Update available -->
          <div v-if="updateState.available" class="mt-3 space-y-2">
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground">{{ t('about.update.newVersion') }}</span>
              <Badge variant="default">{{ updateState.version }}</Badge>
            </div>
            <p v-if="updateState.notes" class="text-xs text-muted-foreground whitespace-pre-line">
              {{ updateState.notes }}
            </p>
            <Button class="w-full" :disabled="updateState.downloading" @click="downloadAndInstall">
              <Loader2 v-if="updateState.downloading" class="mr-2 h-4 w-4 animate-spin" />
              <Download v-else class="mr-2 h-4 w-4" />
              <span v-if="updateState.downloading && updateState.progress > 0">
                {{ t('about.update.downloading') }} {{ updateState.progress }}%
              </span>
              <span v-else-if="updateState.downloading">
                {{ t('about.update.downloading') }}
              </span>
              <span v-else>
                {{ t('about.update.downloadAndInstall') }}
              </span>
            </Button>
          </div>

          <!-- No update available -->
          <p v-else-if="updateState.checked && !updateState.available"
            class="mt-2 text-xs text-center text-muted-foreground">
            {{ t('about.update.upToDate') }}
          </p>

          <!-- Error message -->
          <p v-if="updateState.error" class="mt-2 text-xs text-center text-destructive">
            {{ updateState.error }}
          </p>
        </div>
      </CardContent>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { showToast } from '@/lib/toast'
import { Loader2, RefreshCw, Download } from 'lucide-vue-next'

const { t } = useI18n()

const tapCount = ref(0)
const appInfo = reactive({
  version: '',
  devMode: false,
})

const updateState = reactive({
  checking: false,
  checked: false,
  available: false,
  downloading: false,
  version: '',
  notes: '',
  progress: 0,
  error: '',
})

// Store the update object for later use
let pendingUpdate: Awaited<ReturnType<typeof check>> = null

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

async function checkForUpdates() {
  updateState.checking = true
  updateState.checked = false
  updateState.available = false
  updateState.error = ''
  updateState.version = ''
  updateState.notes = ''
  pendingUpdate = null

  try {
    const update = await check()
    updateState.checked = true

    if (update) {
      updateState.available = true
      updateState.version = update.version
      updateState.notes = update.body || ''
      pendingUpdate = update
      showToast(t('about.update.foundUpdate', { version: update.version }), 'info')
    } else {
      showToast(t('about.update.upToDate'), 'info')
    }
  } catch (e) {
    console.error('检查更新失败', e)
    updateState.error = e instanceof Error ? e.message : String(e)
    showToast(t('about.update.checkFailed'), 'error')
  } finally {
    updateState.checking = false
  }
}

async function downloadAndInstall() {
  if (!pendingUpdate) {
    showToast(t('about.update.noUpdate'), 'error')
    return
  }

  updateState.downloading = true
  updateState.progress = 0
  updateState.error = ''

  try {
    let downloaded = 0
    let contentLength = 0

    await pendingUpdate.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength ?? 0
          console.log(`开始下载，总大小: ${contentLength} 字节`)
          break
        case 'Progress':
          downloaded += event.data.chunkLength
          if (contentLength > 0) {
            updateState.progress = Math.round((downloaded / contentLength) * 100)
          }
          break
        case 'Finished':
          console.log('下载完成')
          updateState.progress = 100
          break
      }
    })

    showToast(t('about.update.installSuccess'), 'info')
    // Relaunch the app after a short delay
    setTimeout(async () => {
      await relaunch()
    }, 1000)
  } catch (e) {
    console.error('下载更新失败', e)
    updateState.error = e instanceof Error ? e.message : String(e)
    showToast(t('about.update.downloadFailed'), 'error')
  } finally {
    updateState.downloading = false
  }
}

onMounted(() => {
  loadInfo()
})
</script>
