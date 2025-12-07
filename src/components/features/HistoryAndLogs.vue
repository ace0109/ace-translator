<template>
  <div class="min-h-screen bg-background text-foreground flex flex-col">
    <!-- Tab Bar -->
    <div class="flex border-b bg-card shrink-0">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="flex-1 px-4 py-3 text-sm font-medium transition-colors"
        :class="{
          'border-b-2 border-primary text-primary': activeTab === tab.id,
          'text-muted-foreground hover:text-foreground hover:bg-muted/50': activeTab !== tab.id
        }"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 overflow-hidden">
      <!-- History Tab -->
      <div v-show="activeTab === 'history'" class="h-full p-4 overflow-auto">
        <div v-if="history.length === 0" class="text-muted-foreground text-center py-8">
          {{ t('history.translationHistory.empty') }}
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="(item, index) in history"
            :key="item.id"
            class="rounded-lg border bg-card p-3 shadow-sm hover:bg-muted/30 transition cursor-pointer"
            @click="toggleHistoryExpand(index)"
          >
            <div class="flex items-center justify-between">
              <div class="flex-1 min-w-0">
                <p class="text-sm text-foreground truncate">{{ item.source_text }}</p>
                <p class="text-xs text-muted-foreground mt-1 flex items-center gap-2">
                  <span>{{ item.source_lang }} → {{ item.target_lang }}</span>
                  <span v-if="item.provider" class="px-1.5 py-0.5 rounded bg-muted text-[10px]">{{ item.provider }}</span>
                  <span>{{ formatTime(item.created_at) }}</span>
                </p>
              </div>
              <ChevronDown
                class="h-4 w-4 text-muted-foreground transition-transform shrink-0 ml-2"
                :class="{ 'rotate-180': expandedHistory === index }"
              />
            </div>
            <div v-if="expandedHistory === index" class="mt-3 pt-3 border-t">
              <div class="flex items-center justify-between mb-2">
                <p class="text-xs font-medium text-muted-foreground">{{ t('history.translationHistory.result') }}</p>
                <div class="flex gap-1">
                  <button
                    class="inline-flex items-center gap-1 rounded border bg-muted/60 px-2 py-1 text-xs text-muted-foreground transition hover:bg-accent hover:text-accent-foreground"
                    @click.stop="copyText(item.translated_text)"
                  >
                    <Copy class="h-3 w-3" /> {{ t('common.copy') }}
                  </button>
                  <button
                    class="inline-flex items-center gap-1 rounded border bg-destructive/10 px-2 py-1 text-xs text-destructive transition hover:bg-destructive hover:text-destructive-foreground"
                    @click.stop="deleteHistoryItem(item.id)"
                  >
                    <Trash2 class="h-3 w-3" /> {{ t('common.delete') }}
                  </button>
                </div>
              </div>
              <p class="text-sm text-foreground whitespace-pre-wrap bg-muted/30 rounded p-2">{{ item.translated_text }}</p>
              <p v-if="item.model" class="text-[10px] text-muted-foreground mt-2">
                {{ t('history.translationHistory.model') }}: {{ item.model }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- Cache Tab -->
      <div v-show="activeTab === 'cache'" class="h-full p-4">
        <div class="rounded-lg border bg-card p-4 shadow-sm">
          <h3 class="text-base font-semibold text-foreground mb-4">{{ t('history.cacheManagement.stats') }}</h3>
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-sm text-muted-foreground">{{ t('history.cacheManagement.count') }}</span>
              <span class="text-sm font-medium text-foreground">{{ cacheStats.count }} {{ t('history.cacheManagement.entries') }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-sm text-muted-foreground">{{ t('history.cacheManagement.size') }}</span>
              <span class="text-sm font-medium text-foreground">{{ formatSize(cacheStats.size) }}</span>
            </div>
          </div>
          <div class="mt-6 flex justify-end">
            <Button @click="clearCache" variant="destructive" size="sm">
              {{ t('history.cacheManagement.clear') }}
            </Button>
          </div>
        </div>
      </div>

      <!-- Logs Tab -->
      <div v-show="activeTab === 'logs'" class="h-full flex flex-col p-4">
        <div class="flex justify-between items-center mb-4">
          <h1 class="text-base font-semibold">{{ t('history.appLogs.title') }}</h1>
          <div class="flex gap-2">
            <Button @click="refreshLogs" variant="outline" size="sm">
              {{ t('common.refresh') }}
            </Button>
            <Button @click="clearLogs" variant="destructive" size="sm">
              {{ t('history.appLogs.clear') }}
            </Button>
          </div>
        </div>

        <div
          ref="logContainer"
          class="flex-1 bg-muted/30 rounded-lg p-4 overflow-auto font-mono text-sm border"
          @scroll="handleScroll"
        >
          <div v-if="logs.length === 0" class="text-muted-foreground text-center py-8">
            {{ t('history.appLogs.empty') }}
          </div>
          <div v-else>
            <div
              v-for="(log, index) in logs"
              :key="index"
              class="py-1 border-b border-border last:border-0"
              :class="{
                'text-destructive': log.level === 'ERROR',
                'text-yellow-500 dark:text-yellow-400': log.level === 'DEBUG',
                'text-green-600 dark:text-green-400': log.level === 'INFO',
              }"
            >
              <span class="text-muted-foreground">{{ log.timestamp }}</span>
              <span class="mx-2 px-1 rounded text-xs" :class="{
                'bg-destructive/20': log.level === 'ERROR',
                'bg-yellow-500/20': log.level === 'DEBUG',
                'bg-green-500/20': log.level === 'INFO',
              }">{{ log.level }}</span>
              <span>{{ log.message }}</span>
            </div>
          </div>
        </div>

        <div class="mt-2 text-xs text-muted-foreground flex items-center justify-between">
          <span>{{ t('history.appLogs.total', { count: logs.length }) }} | {{ t('history.appLogs.autoRefresh') }}{{ autoRefresh ? t('history.appLogs.enabled') : t('history.appLogs.disabled') }}</span>
          <div class="flex items-center gap-4">
            <label class="cursor-pointer flex items-center">
              <input type="checkbox" v-model="autoScroll" class="mr-1" />
              {{ t('history.appLogs.autoScroll') }}
            </label>
            <label class="cursor-pointer flex items-center">
              <input type="checkbox" v-model="autoRefresh" class="mr-1" />
              {{ t('history.appLogs.autoRefresh') }}
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
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
