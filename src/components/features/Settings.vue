<template>
  <div class="p-6">
    <div class="mx-auto flex max-w-3xl flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>基础信息</CardTitle>
          <CardDescription>确保 API Key 已填写，否则无法调用翻译服务。</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="space-y-2">
            <Label for="apiKey">AI API Key</Label>
            <Input
              id="apiKey"
              v-model="settingsForm.apiKey"
              type="password"
              placeholder="输入 API Key"
            />
          </div>
          <div class="space-y-2">
            <Label for="targetLanguage">默认目标语言</Label>
            <LanguageSelector
              id="targetLanguage"
              v-model="settingsForm.targetLanguage"
              placeholder="选择默认目标语言"
            />
          </div>
          <div class="space-y-2">
            <Label>主题</Label>
            <div class="flex flex-wrap gap-2">
              <Button
                :variant="settingsForm.theme === 'light' ? 'default' : 'outline'"
                size="sm"
                @click="settingsForm.theme = 'light'"
              >
                浅色
              </Button>
              <Button
                :variant="settingsForm.theme === 'dark' ? 'default' : 'outline'"
                size="sm"
                @click="settingsForm.theme = 'dark'"
              >
                深色
              </Button>
            </div>
            <p class="text-xs text-muted-foreground">主题会同步到窗口并在启动时自动应用。</p>
          </div>
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <Label>常用目标语言</Label>
              <span class="text-xs text-muted-foreground">{{ commonTargetCount }}/{{ maxCommonTargets }}</span>
            </div>
            <p class="text-xs text-muted-foreground">用于主窗口缓存与悬浮窗多语言翻译，最多选择 {{ maxCommonTargets }} 个。</p>
            <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
              <label
                v-for="opt in languageOptions.filter(o => o.value !== 'auto')"
                :key="opt.value"
                class="flex cursor-pointer items-center gap-2 rounded-md border px-3 py-2 text-sm hover:border-ring"
              >
                <input
                  type="checkbox"
                  class="h-4 w-4 accent-primary"
                  :value="opt.value"
                  :checked="settingsForm.commonTargetLanguages.includes(opt.value)"
                  @change="toggleCommonTarget(opt.value)"
                />
                <span class="text-foreground">{{ opt.label }}</span>
              </label>
            </div>
          </div>
        </CardContent>
        <CardFooter class="justify-end gap-2">
          <Button
            :disabled="isSaving"
            class="gap-2"
            @click="saveSettings"
          >
            <Loader2 v-if="isSaving" class="h-4 w-4 animate-spin" />
            <span>{{ isSaving ? '保存中...' : '保存设置' }}</span>
          </Button>
        </CardFooter>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>API 连通性测试</CardTitle>
          <CardDescription>查看发送给 AI 的参数和响应结果，便于排查。</CardDescription>
        </CardHeader>
        <CardContent class="space-y-3">
          <div class="flex items-center gap-3">
            <Button :disabled="testLoading" class="gap-2" @click="runApiTest">
              <Loader2 v-if="testLoading" class="h-4 w-4 animate-spin" />
              <span>{{ testLoading ? '测试中...' : '开始测试' }}</span>
            </Button>
            <p class="text-xs text-muted-foreground">使用常用目标语言作为批量目标，示例文本：This is a connectivity test...</p>
          </div>
          <div class="grid gap-3 md:grid-cols-2">
            <div class="space-y-1">
              <p class="text-xs font-semibold text-muted-foreground">请求参数</p>
              <pre class="max-h-56 overflow-auto rounded-md bg-muted/40 p-3 text-xs">{{ testPayload || '尚未发送' }}</pre>
            </div>
            <div class="space-y-1">
              <p class="text-xs font-semibold text-muted-foreground">响应结果</p>
              <pre class="max-h-56 overflow-auto rounded-md bg-muted/40 p-3 text-xs">
{{ testError ? `错误：${testError}` : (testResponse || '等待响应...') }}
              </pre>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>缓存</CardTitle>
          <CardDescription>查看并清理翻译缓存（保存在本地 SQLite）。</CardDescription>
        </CardHeader>
        <CardContent class="space-y-3">
          <div class="flex items-center justify-between rounded-md border bg-muted/40 px-3 py-2 text-sm">
            <span class="text-muted-foreground">缓存条数</span>
            <span class="font-mono text-foreground">{{ cacheCount }}</span>
          </div>
          <div class="flex items-center justify-between rounded-md border bg-muted/40 px-3 py-2 text-sm">
            <span class="text-muted-foreground">缓存大小（字节）</span>
            <span class="font-mono text-foreground">{{ cacheSize }}</span>
          </div>
        </CardContent>
        <CardFooter class="justify-end gap-2">
          <Button variant="secondary" :disabled="isSaving" @click="loadCacheStats">刷新</Button>
          <Button variant="destructive" :disabled="isSaving" @click="clearCache">清空缓存</Button>
        </CardFooter>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Loader2 } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settings'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import LanguageSelector from '../common/LanguageSelector.vue'
import { languageOptions, defaultCommonTargets } from '@/constants/languages'
import { showToast } from '@/lib/toast'
import { isTauriEnv } from '@/utils/env'

interface SettingsForm {
  apiKey: string
  targetLanguage: string
  theme: 'light' | 'dark'
  commonTargetLanguages: string[]
}

const settingsStore = useSettingsStore()
const settingsForm = ref<SettingsForm>({
  apiKey: '',
  targetLanguage: 'zh-CN',
  theme: 'light',
  commonTargetLanguages: defaultCommonTargets.slice(),
})

const isSaving = ref(false)
const maxCommonTargets = 5
const commonTargetCount = computed(() => settingsForm.value.commonTargetLanguages.length)
const testLoading = ref(false)
const testPayload = ref('')
const testResponse = ref('')
const testError = ref('')

const cacheCount = ref(0)
const cacheSize = ref(0)

const applyThemeClass = (theme: string) => {
  if (theme === 'dark') {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

const loadCacheStats = async () => {
  if (!isTauriEnv()) return
  try {
    const [count, size] = (await invoke('cache_stats')) as [number, number]
    cacheCount.value = count
    cacheSize.value = size
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`读取缓存失败：${errMsg}`, 'error')
  }
}

const clearCache = async () => {
  if (!isTauriEnv()) return
  try {
    await invoke('clear_cache')
    cacheCount.value = 0
    cacheSize.value = 0
    showToast('缓存已清理', 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`清理缓存失败：${errMsg}`, 'error')
  }
}

const loadSettings = async () => {
  if (!isTauriEnv()) return
  try {
    const loadedSettings: any = await invoke('get_settings')
    settingsForm.value.apiKey = loadedSettings.api_key || ''
    settingsForm.value.targetLanguage = loadedSettings.target_language || 'zh-CN'
    settingsForm.value.theme = loadedSettings.theme || 'light'
    settingsForm.value.commonTargetLanguages =
      loadedSettings.common_target_languages?.slice(0, maxCommonTargets) ||
      defaultCommonTargets.slice()
    settingsStore.setApiKey(settingsForm.value.apiKey)
    settingsStore.setTheme(settingsForm.value.theme)
    settingsStore.setCommonTargetLanguages(settingsForm.value.commonTargetLanguages)
    applyThemeClass(settingsForm.value.theme)
    loadCacheStats()
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`加载设置失败：${errMsg}`, 'error')
  }
}

const saveSettings = async () => {
  if (!isTauriEnv()) {
    showToast('当前不在 Tauri 环境，无法保存', 'error')
    return
  }
  if (!settingsForm.value.apiKey.trim()) {
    showToast('请填写 API Key 后再保存', 'error')
    return
  }

  isSaving.value = true
  try {
    await invoke('save_settings', {
      settings: {
        api_key: settingsForm.value.apiKey,
        target_language: settingsForm.value.targetLanguage,
        theme: settingsForm.value.theme,
        common_target_languages: settingsForm.value.commonTargetLanguages,
      },
    })
    settingsStore.setApiKey(settingsForm.value.apiKey)
    settingsStore.setTheme(settingsForm.value.theme)
    settingsStore.setCommonTargetLanguages(settingsForm.value.commonTargetLanguages)
    applyThemeClass(settingsForm.value.theme)
    showToast('设置已保存', 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`保存失败：${errMsg}`, 'error')
  } finally {
    isSaving.value = false
  }
}

onMounted(() => {
  loadSettings()
  loadCacheStats()
})

watch(
  () => settingsForm.value.theme,
  (theme) => applyThemeClass(theme),
)

const toggleCommonTarget = (lang: string) => {
  const exists = settingsForm.value.commonTargetLanguages.includes(lang)
  if (exists) {
    settingsForm.value.commonTargetLanguages = settingsForm.value.commonTargetLanguages.filter((l) => l !== lang)
  } else {
    if (settingsForm.value.commonTargetLanguages.length >= maxCommonTargets) {
      return
    }
    settingsForm.value.commonTargetLanguages = [...settingsForm.value.commonTargetLanguages, lang]
  }
}

const commonTargets = () => {
  return (
    settingsForm.value.commonTargetLanguages.slice(0, maxCommonTargets) ??
    defaultCommonTargets.slice()
  )
}

const runApiTest = async () => {
  testLoading.value = true
  testError.value = ''
  testPayload.value = ''
  testResponse.value = ''

  if (!isTauriEnv()) {
    testLoading.value = false
    testError.value = '当前不在 Tauri 环境，无法调用 API'
    return
  }

  if (!settingsForm.value.apiKey.trim()) {
    testLoading.value = false
    testError.value = '请先填写并保存 API Key'
    return
  }

  const payload = {
    text: 'This is a connectivity test for Ace Translator.',
    targetLangs: commonTargets(),
  }
  testPayload.value = JSON.stringify(payload, null, 2)

  try {
    const res: any = await invoke('translate_text', payload)
    testResponse.value = JSON.stringify(res, null, 2)
    showToast('API 测试成功', 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    testError.value = errMsg
    showToast(`API 测试失败：${errMsg}`, 'error')
  } finally {
    testLoading.value = false
  }
}
</script>
