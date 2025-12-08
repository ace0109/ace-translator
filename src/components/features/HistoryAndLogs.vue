<template>
  <Tabs v-model="activeTab" class="flex min-h-screen flex-col bg-background text-foreground">
    <TabsList class="grid w-full grid-cols-3 rounded-none border-b bg-card px-2 py-2">
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
          <Card
            v-for="(item, index) in history"
            :key="item.id"
            class="cursor-pointer transition hover:border-primary/40"
            @click="toggleHistoryExpand(index)"
          >
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
                <ChevronDown
                  class="h-4 w-4 text-muted-foreground transition-transform"
                  :class="{ 'rotate-180': expandedHistory === index }"
                />
              </Button>
            </CardHeader>
            <CardContent v-if="expandedHistory === index" class="space-y-3 border-t pt-3">
              <div class="flex items-center justify-between gap-2">
                <p class="text-xs font-medium text-muted-foreground">{{ t('history.translationHistory.result') }}</p>
                <div class="flex gap-2">
                  <Button variant="ghost" size="sm" class="h-8 px-2 text-xs" @click.stop="copyText(item.translated_text)">
                    <Copy class="h-4 w-4" />
                    <span class="ml-1">{{ t('common.copy') }}</span>
                  </Button>
                  <Button variant="destructive" size="sm" class="h-8 px-2 text-xs" @click.stop="deleteHistoryItem(item.id)">
                    <Trash2 class="h-4 w-4" />
                    <span class="ml-1">{{ t('common.delete') }}</span>
                  </Button>
                </div>
              </div>
              <p class="whitespace-pre-wrap rounded-md bg-muted/40 p-3 text-sm leading-relaxed text-foreground">{{ item.translated_text }}</p>
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
            <span class="font-medium text-foreground">{{ cacheStats.count }} {{ t('history.cacheManagement.entries') }}</span>
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

    <TabsContent value="logs" class="flex-1 overflow-hidden">
      <Card class="m-4 flex h-[calc(100vh-140px)] flex-col">
        <CardHeader class="flex flex-row items-center justify-between gap-2">
          <CardTitle class="text-base">{{ t('history.appLogs.title') }}</CardTitle>
          <div class="flex gap-2">
            <Button @click="refreshLogs" variant="outline" size="sm">
              {{ t('common.refresh') }}
            </Button>
            <Button @click="clearLogs" variant="destructive" size="sm">
              {{ t('history.appLogs.clear') }}
            </Button>
          </div>
        </CardHeader>
        <CardContent class="flex flex-1 flex-col gap-3 overflow-hidden">
          <div
            ref="logContainer"
            class="flex-1 overflow-auto rounded-lg border bg-muted/30 p-4 font-mono text-sm"
            @scroll="handleScroll"
          >
            <div v-if="logs.length === 0" class="py-8 text-center text-muted-foreground">
              {{ t('history.appLogs.empty') }}
            </div>
            <div v-else>
              <div
                v-for="(log, index) in logs"
                :key="index"
                class="border-b border-border py-1 last:border-0"
                :class="{
                  'text-destructive': log.level === 'ERROR',
                  'text-yellow-500 dark:text-yellow-400': log.level === 'DEBUG',
                  'text-green-600 dark:text-green-400': log.level === 'INFO',
                }"
              >
                <span class="text-muted-foreground">{{ log.timestamp }}</span>
                <Badge :variant="log.level === 'ERROR' ? 'destructive' : 'secondary'" class="mx-2 px-2 py-0 text-[10px]">
                  {{ log.level }}
                </Badge>
                <span>{{ log.message }}</span>
              </div>
            </div>
          </div>

          <div class="flex flex-wrap items-center justify-between gap-3 text-xs text-muted-foreground">
            <span>{{ t('history.appLogs.total', { count: logs.length }) }} | {{ t('history.appLogs.autoRefresh') }}{{ autoRefresh ? t('history.appLogs.enabled') : t('history.appLogs.disabled') }}</span>
            <div class="flex items-center gap-4">
              <div class="flex items-center gap-2">
                <Checkbox id="auto-scroll" v-model:checked="autoScroll" />
                <Label for="auto-scroll" class="cursor-pointer">
                  {{ t('history.appLogs.autoScroll') }}
                </Label>
              </div>
              <div class="flex items-center gap-2">
                <Checkbox id="auto-refresh" v-model:checked="autoRefresh" />
                <Label for="auto-refresh" class="cursor-pointer">
                  {{ t('history.appLogs.autoRefresh') }}
                </Label>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </TabsContent>
  </Tabs>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ChevronDown, Copy, Trash2 } from 'lucide-vue-next'
import { showToast } from '@/lib/toast'

const { t } = useI18n()

interface LogEntry {
  timestamp: string
  level: string
  message: string
}

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
  { id: 'logs', label: t('history.tabs.logs') },
])

const activeTab = ref('history')
const logs = ref<LogEntry[]>([])
const history = ref<HistoryEntry[]>([])
const expandedHistory = ref<number | null>(null)
const cacheStats = ref({ count: 0, size: 0 })
const autoRefresh = ref(true)
const autoScroll = ref(true)
const logContainer = ref<HTMLElement | null>(null)
let refreshInterval: number | null = null

// Logs functions
async function refreshLogs() {
  try {
    const prevLength = logs.value.length
    logs.value = await invoke<LogEntry[]>('get_logs')

    if (autoScroll.value && logs.value.length > prevLength) {
      await nextTick()
      scrollToBottom()
    }
  } catch (e) {
    console.error('获取日志失败:', e)
  }
}

async function clearLogs() {
  try {
    await invoke('clear_logs')
    logs.value = []
    showToast(t('history.appLogs.cleared'), 'info')
  } catch (e) {
    console.error('清空日志失败:', e)
  }
}

function scrollToBottom() {
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

function handleScroll() {
  if (!logContainer.value) return

  const { scrollTop, scrollHeight, clientHeight } = logContainer.value
  const isNearBottom = scrollHeight - scrollTop - clientHeight < 50

  if (!isNearBottom && autoScroll.value) {
    autoScroll.value = false
  }
}

watch(autoScroll, (newVal) => {
  if (newVal) {
    nextTick(() => scrollToBottom())
  }
})

// Cache functions
async function loadCacheStats() {
  try {
    const [count, size] = await invoke<[number, number]>('cache_stats')
    cacheStats.value = { count, size }
  } catch (e) {
    console.error('获取缓存统计失败:', e)
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
    console.error('获取历史记录失败:', e)
    history.value = []
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

onMounted(() => {
  refreshLogs()
  loadCacheStats()
  loadHistory()

  refreshInterval = window.setInterval(() => {
    if (autoRefresh.value && activeTab.value === 'logs') {
      refreshLogs()
    }
  }, 2000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

// Refresh data when tab changes
watch(activeTab, (newTab) => {
  if (newTab === 'logs') {
    refreshLogs()
  } else if (newTab === 'cache') {
    loadCacheStats()
  } else if (newTab === 'history') {
    loadHistory()
  }
})
</script>
