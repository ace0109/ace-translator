<template>
  <div class="flex min-h-screen flex-col bg-background text-foreground">
    <Card class="m-4 flex flex-1 flex-col">
      <CardHeader class="flex flex-row items-center justify-between gap-2">
        <div>
          <CardTitle class="text-base">{{ t('history.appLogs.title') }}</CardTitle>
          <CardDescription>{{ t('history.appLogs.total', { count: logs.length }) }}</CardDescription>
        </div>
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
          <span>{{ t('history.appLogs.autoRefresh') }}{{ autoRefresh ? t('history.appLogs.enabled') : t('history.appLogs.disabled') }}</span>
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'

const { t } = useI18n()

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
