<template>
  <div class="min-h-screen bg-gray-900 text-gray-100 p-4">
    <div class="flex justify-between items-center mb-4">
      <h1 class="text-xl font-bold">应用日志</h1>
      <div class="flex gap-2">
        <Button @click="refreshLogs" variant="outline" size="sm">
          刷新
        </Button>
        <Button @click="clearLogs" variant="destructive" size="sm">
          清空日志
        </Button>
      </div>
    </div>

    <div
      ref="logContainer"
      class="bg-gray-800 rounded-lg p-4 h-[calc(100vh-120px)] overflow-auto font-mono text-sm"
      @scroll="handleScroll"
    >
      <div v-if="logs.length === 0" class="text-gray-500 text-center py-8">
        暂无日志
      </div>
      <div v-else>
        <div
          v-for="(log, index) in logs"
          :key="index"
          class="py-1 border-b border-gray-700 last:border-0"
          :class="{
            'text-red-400': log.level === 'ERROR',
            'text-yellow-400': log.level === 'DEBUG',
            'text-green-400': log.level === 'INFO',
          }"
        >
          <span class="text-gray-500">{{ log.timestamp }}</span>
          <span class="mx-2 px-1 rounded text-xs" :class="{
            'bg-red-900': log.level === 'ERROR',
            'bg-yellow-900': log.level === 'DEBUG',
            'bg-green-900': log.level === 'INFO',
          }">{{ log.level }}</span>
          <span>{{ log.message }}</span>
        </div>
      </div>
    </div>

    <div class="mt-2 text-xs text-gray-500 flex items-center justify-between">
      <span>共 {{ logs.length }} 条日志 | 自动刷新已{{ autoRefresh ? '开启' : '关闭' }}</span>
      <div class="flex items-center gap-4">
        <label class="cursor-pointer flex items-center">
          <input type="checkbox" v-model="autoScroll" class="mr-1" />
          自动滚动
        </label>
        <label class="cursor-pointer flex items-center">
          <input type="checkbox" v-model="autoRefresh" class="mr-1" />
          自动刷新
        </label>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'

interface LogEntry {
  timestamp: string
  level: string
  message: string
}

const logs = ref<LogEntry[]>([])
const autoRefresh = ref(true)
const autoScroll = ref(true)
const logContainer = ref<HTMLElement | null>(null)
let refreshInterval: number | null = null
let userScrolledUp = false

async function refreshLogs() {
  try {
    const prevLength = logs.value.length
    logs.value = await invoke<LogEntry[]>('get_logs')

    // 如果有新日志且开启了自动滚动，则滚动到底部
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
  // 如果用户向上滚动超过 50px，暂时禁用自动滚动
  const isNearBottom = scrollHeight - scrollTop - clientHeight < 50

  if (!isNearBottom && autoScroll.value) {
    userScrolledUp = true
    autoScroll.value = false
  }
}

// 当用户重新开启自动滚动时，立即滚动到底部
watch(autoScroll, (newVal) => {
  if (newVal) {
    userScrolledUp = false
    nextTick(() => scrollToBottom())
  }
})

onMounted(() => {
  refreshLogs()
  refreshInterval = window.setInterval(() => {
    if (autoRefresh.value) {
      refreshLogs()
    }
  }, 2000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>
