<template>
  <div class="flex min-h-screen flex-col bg-background text-foreground">
    <!-- 全屏 Loading -->
    <div v-if="isLoading" class="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
      <div class="flex flex-col items-center gap-4">
        <Loader2 class="h-8 w-8 animate-spin text-primary" />
        <p class="text-sm text-muted-foreground">{{ t('history.loading') }}</p>
      </div>
    </div>

    <!-- 配置加载失败弹窗 -->
    <Dialog v-model:open="showErrorDialog">
      <DialogContent class="max-w-md">
        <DialogHeader>
          <DialogTitle class="text-destructive">{{ t('history.loadFailed') }}</DialogTitle>
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

    <Tabs v-model="activeTab" class="flex flex-1 flex-col" :class="{ 'opacity-50': isLoading }">
      <TabsList class="grid w-full grid-cols-2 rounded-none border-b bg-card px-2 py-2">
        <TabsTrigger v-for="tab in tabs" :key="tab.id" :value="tab.id" class="h-9">
          {{ tab.label }}
        </TabsTrigger>
      </TabsList>

      <TabsContent value="history" class="flex-1 overflow-hidden">
        <div class="h-full overflow-auto p-4">
          <div v-if="history.length === 0" class="py-8 text-center text-muted-foreground">
            {{ t('history.translationHistory.empty') }}
          </div>
          <div v-else class="space-y-3">
            <Card v-for="(item, index) in history" :key="item.id"
              class="cursor-pointer transition hover:border-primary/40" @click="toggleHistoryExpand(index)">
              <CardHeader class="flex flex-row items-start justify-between gap-2">
                <div class="min-w-0 flex-1 space-y-1">
                  <CardTitle class="truncate text-sm font-medium leading-tight">{{ item.source_text }}</CardTitle>
                  <CardDescription class="flex flex-wrap items-center gap-2 text-xs">
                    <span>{{ item.source_lang }} → {{ item.target_lang }}</span>
                    <Badge v-if="item.provider" variant="secondary">{{ item.provider }}</Badge>
                    <span class="text-muted-foreground">{{ formatTime(item.created_at) }}</span>
                  </CardDescription>
                </div>
                <Button variant="ghost" size="icon" class="shrink-0" @click.stop="toggleHistoryExpand(index)">
                  <ChevronDown class="h-4 w-4 text-muted-foreground transition-transform"
                    :class="{ 'rotate-180': expandedHistory === index }" />
                </Button>
              </CardHeader>
              <CardContent v-if="expandedHistory === index" class="space-y-3 border-t pt-3">
                <div class="flex items-center justify-between gap-2">
                  <p class="text-xs font-medium text-muted-foreground">{{ t('history.translationHistory.result') }}</p>
                  <div class="flex gap-2">
                    <Button variant="ghost" size="sm" class="h-8 px-2 text-xs"
                      @click.stop="copyText(item.translated_text)">
                      <Copy class="h-4 w-4" />
                      <span class="ml-1">{{ t('common.copy') }}</span>
                    </Button>
                    <Button variant="destructive" size="sm" class="h-8 px-2 text-xs"
                      @click.stop="deleteHistoryItem(item.id)">
                      <Trash2 class="h-4 w-4" />
                      <span class="ml-1">{{ t('common.delete') }}</span>
                    </Button>
                  </div>
                </div>
                <p class="whitespace-pre-wrap rounded-md bg-muted/40 p-3 text-sm leading-relaxed text-foreground">{{
                  item.translated_text }}</p>
                <p v-if="item.model" class="text-[10px] text-muted-foreground">
                  {{ t('history.translationHistory.model') }}: {{ item.model }}
                </p>
              </CardContent>
            </Card>
          </div>
        </div>
      </TabsContent>

      <TabsContent value="cache" class="flex-1 overflow-auto p-4">
        <Card class="max-w-xl">
          <CardHeader>
            <CardTitle>{{ t('history.cacheManagement.stats') }}</CardTitle>
          </CardHeader>
          <CardContent class="space-y-3">
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground">{{ t('history.cacheManagement.count') }}</span>
              <span class="font-medium text-foreground">{{ cacheStats.count }} {{ t('history.cacheManagement.entries')
                }}</span>
            </div>
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground">{{ t('history.cacheManagement.size') }}</span>
              <span class="font-medium text-foreground">{{ formatSize(cacheStats.size) }}</span>
            </div>
            <div class="flex justify-end pt-2">
              <Button @click="clearCache" variant="destructive" size="sm">
                {{ t('history.cacheManagement.clear') }}
              </Button>
            </div>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { ChevronDown, Copy, Trash2, Loader2 } from 'lucide-vue-next'
import { showToast } from '@/lib/toast'
import { isTauriEnv } from '@/utils/env'

const { t } = useI18n()

interface HistoryEntry {
  id: number
  source_text: string
  translated_text: string
  source_lang: string
  target_lang: string
  provider: string | null
  model: string | null
  created_at: string
}

const tabs = computed(() => [
  { id: 'history', label: t('history.tabs.history') },
  { id: 'cache', label: t('history.tabs.cache') },
])

const activeTab = ref('history')
const history = ref<HistoryEntry[]>([])
const expandedHistory = ref<number | null>(null)
const cacheStats = ref({ count: 0, size: 0 })
const isLoading = ref(false)
const showErrorDialog = ref(false)
const errorMessage = ref('')
let hasInitialized = false

// Cache functions
async function loadCacheStats() {
  try {
    const [count, size] = await invoke<[number, number]>('cache_stats')
    cacheStats.value = { count, size }
  } catch (e) {
    throw new Error(`获取缓存统计失败: ${e}`)
  }
}

async function clearCache() {
  try {
    await invoke('clear_cache')
    cacheStats.value = { count: 0, size: 0 }
    showToast(t('history.cacheManagement.cleared'), 'info')
  } catch (e) {
    console.error('清空缓存失败:', e)
    showToast(t('history.cacheManagement.clearFailed'), 'error')
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

// History functions
async function loadHistory() {
  try {
    history.value = await invoke<HistoryEntry[]>('get_translation_history', {
      limit: 100,
      offset: 0,
    })
  } catch (e) {
    throw new Error(`获取历史记录失败: ${e}`)
  }
}

async function deleteHistoryItem(id: number) {
  try {
    await invoke('delete_history_entry', { id })
    history.value = history.value.filter(item => item.id !== id)
    showToast(t('history.translationHistory.deleted'), 'info')
  } catch (e) {
    console.error('删除历史记录失败:', e)
    showToast(t('history.translationHistory.deleteFailed'), 'error')
  }
}

function toggleHistoryExpand(index: number) {
  expandedHistory.value = expandedHistory.value === index ? null : index
}

function formatTime(timestamp: string): string {
  try {
    const date = new Date(timestamp)
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return timestamp
  }
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    showToast(t('common.copied'), 'info')
  } catch (error: any) {
    showToast(t('translator.copyFailed'), 'error')
  }
}

const init = async () => {
  if (!isTauriEnv()) return

  isLoading.value = true
  try {
    await Promise.all([
      loadHistory(),
      loadCacheStats()
    ])
  } catch (error: any) {
    errorMessage.value = error.message || String(error)
    showErrorDialog.value = true
  } finally {
    isLoading.value = false
  }
}

const retryInit = async () => {
  showErrorDialog.value = false
  await init()
}

onMounted(() => {
  // 不要在 onMounted 时加载，等待窗口显示或聚焦时再加载

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

onUnmounted(() => {
  // no-op
})

// Refresh data when tab changes
watch(activeTab, (newTab) => {
  if (newTab === 'cache') {
    loadCacheStats()
  } else if (newTab === 'history') {
    loadHistory()
  }
})
</script>
