<template>
  <div class="min-h-screen bg-background text-foreground">
    <div class="mx-auto flex max-w-3xl flex-col gap-4 p-6">
      <Card class="border-primary/10 shadow-sm">
        <CardHeader class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <CardTitle>{{ t('permissions.title') }}</CardTitle>
            <CardDescription>{{ t('permissions.description') }}</CardDescription>
          </div>
          <Badge :variant="authorized ? 'default' : 'outline'">
            {{ authorized ? t('permissions.authorized') : 'macOS' }}
          </Badge>
        </CardHeader>

        <CardContent class="space-y-4">
          <Alert v-if="!isMac" variant="destructive">
            <AlertTitle>{{ t('permissions.macOnly') }}</AlertTitle>
            <AlertDescription>{{ t('permissions.macOnlyDesc') }}</AlertDescription>
          </Alert>
          <Alert v-else-if="authorized" variant="default">
            <AlertTitle>{{ t('permissions.alreadyAuthorized') }}</AlertTitle>
            <AlertDescription>{{ t('permissions.alreadyAuthorizedDesc') }}</AlertDescription>
          </Alert>

          <!-- Step 1 -->
          <div class="rounded-md border bg-muted/30 p-4">
            <div class="flex items-start gap-3">
              <div
                class="mt-0.5 flex h-6 w-6 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
                1
              </div>
              <div class="flex-1 space-y-2">
                <p class="font-medium">{{ t('permissions.step1.title') }}</p>
                <p class="text-sm text-muted-foreground">
                  {{ t('permissions.step1.desc') }}
                </p>
                <div class="flex flex-wrap gap-2 pt-1">
                  <Button :disabled="openingPanel || !isMac || !isTauriEnv() || authorized" @click="openPrivacyPanel">
                    <Loader2 v-if="openingPanel" class="mr-2 h-4 w-4 animate-spin" />
                    <ExternalLink v-else class="mr-2 h-4 w-4" />
                    {{ t('permissions.step1.button') }}
                  </Button>
                  <span class="text-xs text-muted-foreground">
                    {{ t('permissions.step1.note') }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Step 2 -->
          <div class="rounded-md border bg-muted/30 p-4">
            <div class="flex items-start gap-3">
              <div
                class="mt-0.5 flex h-6 w-6 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
                2
              </div>
              <div class="flex-1 space-y-2">
                <p class="font-medium">{{ t('permissions.step2.title') }}</p>
                <p class="text-sm text-muted-foreground">
                  {{ t('permissions.step2.desc') }}
                </p>
                <div class="flex flex-wrap gap-2 pt-1">
                  <Button variant="outline" :disabled="revealing || !isMac || !isTauriEnv() || authorized"
                    @click="revealInFinder">
                    <Loader2 v-if="revealing" class="mr-2 h-4 w-4 animate-spin" />
                    <FolderOpen v-else class="mr-2 h-4 w-4" />
                    {{ t('permissions.step2.button') }}
                  </Button>
                  <div class="flex flex-col gap-1 text-xs text-muted-foreground">
                    <span>{{ t('permissions.step2.note') }}</span>
                    <span v-if="loadingPath">{{ t('permissions.step2.loading') }}</span>
                    <span v-else-if="!appBundlePath">{{ t('permissions.step2.unavailable') }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { showToast } from '@/lib/toast'
import { isTauriEnv } from '@/utils/env'
import { ExternalLink, FolderOpen, Loader2 } from 'lucide-vue-next'

const { t } = useI18n()

const isMac = navigator.platform.toUpperCase().includes('MAC')
const openingPanel = ref(false)
const revealing = ref(false)
const loadingPath = ref(false)
const appBundlePath = ref('')
const appIcon = '/tauri.svg'
const authorized = ref(false)
let authInterval: number | null = null

const bundleFileUrl = computed(() => {
  if (!appBundlePath.value) return ''
  return `file://${appBundlePath.value}`
})

const checkAuthorized = async () => {
  if (!isTauriEnv() || !isMac) return
  try {
    const val = await invoke<boolean>('check_accessibility')
    authorized.value = Boolean(val)
  } catch (_) { }
}

const startAuthPolling = () => {
  if (authInterval !== null || authorized.value) return
  authInterval = window.setInterval(async () => {
    await checkAuthorized()
    if (authorized.value && authInterval !== null) {
      window.clearInterval(authInterval)
      authInterval = null
    }
  }, 1500)
}

const fetchBundlePath = async () => {
  if (!isTauriEnv() || !isMac) return
  loadingPath.value = true
  try {
    const path = await invoke<string>('get_app_bundle_path')
    appBundlePath.value = path
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(errMsg, 'error')
  } finally {
    loadingPath.value = false
  }
}

const openPrivacyPanel = async () => {
  if (authorized.value) return
  if (!isTauriEnv() || !isMac) {
    showToast(t('settings.notInTauri'), 'error')
    return
  }
  openingPanel.value = true
  try {
    await invoke('open_privacy_panel', { panel: 'accessibility' })
    showToast(t('permissions.openedSettings'), 'info')
    startAuthPolling()
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(errMsg, 'error')
  } finally {
    openingPanel.value = false
  }
}

const revealInFinder = async () => {
  if (authorized.value) return
  if (!isTauriEnv() || !isMac) {
    showToast(t('settings.notInTauri'), 'error')
    return
  }
  revealing.value = true
  try {
    await invoke('reveal_app_in_finder')
    showToast(t('permissions.revealSuccess'), 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(errMsg, 'error')
  } finally {
    revealing.value = false
  }
}

onMounted(() => {
  if (!isMac) return
  checkAuthorized()
  startAuthPolling()
  fetchBundlePath()
})

onUnmounted(() => {
  if (authInterval !== null) {
    window.clearInterval(authInterval)
    authInterval = null
  }
})
</script>
