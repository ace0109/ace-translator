<template>
  <div class="p-6 bg-background">
    <div class="mx-auto flex max-w-3xl flex-col gap-6">
      <!-- 服务商配置 -->
      <Card>
        <CardHeader>
          <CardTitle>{{ t('settings.providerConfig.title') }}</CardTitle>
          <CardDescription>{{ t('settings.providerConfig.description') }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <!-- 服务商 Tab 栏 -->
          <div class="flex border-b">
            <button v-for="provider in providers" :key="provider.name"
              class="relative px-4 py-2 text-sm font-medium transition-colors" :class="[
                activeProvider === provider.name
                  ? 'text-primary'
                  : 'text-muted-foreground hover:text-foreground'
              ]" @click="activeProvider = provider.name">
              {{ provider.display_name }}
              <span v-if="provider.config.enabled" class="absolute -right-1 -top-1 h-2 w-2 rounded-full bg-green-500" />
              <span v-if="activeProvider === provider.name" class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />
            </button>
          </div>

          <!-- 当前服务商配置 -->
          <div v-if="currentProvider" class="space-y-4 pt-2">
            <!-- 启用开关 -->
            <div class="flex items-center justify-between">
              <div>
                <Label>{{ t('settings.providerConfig.enable') }} {{ currentProvider.display_name }}</Label>
                <p class="text-xs text-muted-foreground">
                  {{
                    currentProvider.name === 'zhipu'
                      ? t('settings.providerConfig.zhipuForceEnabled')
                      : t('settings.providerConfig.enableDesc')
                  }}
                </p>
              </div>
              <Switch :checked="currentProvider.name === 'zhipu' ? true : currentProvider.config.enabled"
                :disabled="currentProvider.name === 'zhipu'" @update:checked="(v) => updateProviderEnabled(v)" />
            </div>

            <!-- API Key -->
            <div v-if="currentProvider.name !== 'ollama'" class="space-y-2">
              <Label :for="`${currentProvider.name}-apikey`">{{ t('settings.providerConfig.apiKey') }}</Label>
              <div class="flex gap-2">
                <Input :id="`${currentProvider.name}-apikey`" v-model="currentProvider.config.api_key" type="password"
                  :placeholder="t('settings.providerConfig.apiKeyPlaceholder', { provider: currentProvider.display_name })" />
              </div>
              <p v-if="currentProvider.name === 'zhipu' && currentProvider.config.model.includes('flash')"
                class="text-xs text-muted-foreground">
                {{ t('settings.providerConfig.zhipuFlashHint') }}
              </p>
            </div>

            <!-- 模型选择 -->
            <div class="space-y-2">
              <Label :for="`${currentProvider.name}-model`">{{ t('settings.providerConfig.model') }}</Label>
              <Select :id="`${currentProvider.name}-model`"
                :model-value="currentProvider.config.model || currentProvider.available_models[0]"
                :options="currentProvider.available_models.map(m => ({ label: m, value: m }))"
                :placeholder="t('settings.providerConfig.selectModel')"
                @update:model-value="(v) => updateProviderModel(v)" />
              <p v-if="currentProvider.name === 'ollama'" class="text-xs text-muted-foreground">
                {{ t('settings.providerConfig.ollamaModelHint') }}
              </p>
            </div>

            <!-- 自定义 API 地址（仅 OpenAI 和 Ollama） -->
            <div v-if="currentProvider.supports_base_url" class="space-y-2">
              <Label :for="`${currentProvider.name}-baseurl`">{{ t('settings.providerConfig.baseUrl') }}</Label>
              <Input :id="`${currentProvider.name}-baseurl`" :model-value="currentProvider.config.base_url || ''"
                :placeholder="currentProvider.name === 'ollama' ? 'http://localhost:11434/api/chat' : 'https://api.openai.com/v1/chat/completions'"
                @update:model-value="updateProviderBaseUrl" />
              <p class="text-xs text-muted-foreground">
                {{ currentProvider.name === 'ollama' ? t('settings.providerConfig.baseUrlHintOllama') :
                  t('settings.providerConfig.baseUrlHintOpenAI') }}
              </p>
            </div>

            <!-- 保存和测试按钮 -->
            <div class="flex gap-2 pt-2">
              <Button :disabled="isSavingProvider" @click="saveCurrentProvider">
                <Loader2 v-if="isSavingProvider" class="mr-2 h-4 w-4 animate-spin" />
                {{ t('settings.providerConfig.saveConfig') }}
              </Button>
              <Button variant="outline" :disabled="isTestingProvider" @click="testCurrentProvider">
                <Loader2 v-if="isTestingProvider" class="mr-2 h-4 w-4 animate-spin" />
                {{ t('settings.providerConfig.testConnection') }}
              </Button>
            </div>

            <!-- 测试结果 -->
            <div v-if="testResult" class="space-y-2 rounded-md border p-3">
              <div class="flex items-center gap-2">
                <span :class="testResult.success ? 'text-green-500' : 'text-red-500'" class="text-sm font-medium">
                  {{ testResult.success ? t('settings.providerConfig.connectionSuccess') :
                    t('settings.providerConfig.connectionFailed') }}
                </span>
                <span class="text-xs text-muted-foreground">
                  HTTP {{ testResult.status_code }} | {{ testResult.response_time_ms }}ms
                </span>
              </div>
              <div v-if="testResult.error" class="text-xs text-red-500">
                {{ testResult.error }}
              </div>
              <details v-if="testResult.request_payload" class="text-xs">
                <summary class="cursor-pointer text-muted-foreground hover:text-foreground">
                  {{ t('settings.providerConfig.viewRequestPayload') }}
                </summary>
                <pre
                  class="mt-2 max-h-48 overflow-auto rounded bg-muted/40 p-2">{{ JSON.stringify(testResult.request_payload, null, 2) }}</pre>
              </details>
              <details v-if="testResult.raw_response" class="text-xs">
                <summary class="cursor-pointer text-muted-foreground hover:text-foreground">
                  {{ t('settings.providerConfig.viewRawResponse') }}
                </summary>
                <pre
                  class="mt-2 max-h-48 overflow-auto rounded bg-muted/40 p-2">{{ JSON.stringify(testResult.raw_response, null, 2) }}</pre>
              </details>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- 基础设置 -->
      <Card>
        <CardHeader class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <CardTitle>{{ t('settings.basic.title') }}</CardTitle>
            <CardDescription>{{ t('settings.basic.description') }}</CardDescription>
          </div>
          <Button variant="outline" :disabled="isSaving" @click="resetDefaults">
            {{ t('settings.basic.resetDefaults') }}
          </Button>
        </CardHeader>
        <CardContent class="space-y-4">
          <!-- 界面语言 -->
          <div class="space-y-2">
            <Label>{{ t('settings.interfaceLanguage.label') }}</Label>
            <Select :model-value="locale" :options="localeOptions" @update:model-value="changeLocale" />
            <p class="text-xs text-muted-foreground">{{ t('settings.interfaceLanguage.description') }}</p>
          </div>

          <div class="space-y-2">
            <Label>{{ t('settings.basic.theme') }}</Label>
            <div class="flex flex-wrap gap-2">
              <Button :variant="settingsForm.theme === 'light' ? 'default' : 'outline'" size="sm"
                @click="updateTheme('light')">
                {{ t('settings.basic.themeLight') }}
              </Button>
              <Button :variant="settingsForm.theme === 'dark' ? 'default' : 'outline'" size="sm"
                @click="updateTheme('dark')">
                {{ t('settings.basic.themeDark') }}
              </Button>
            </div>
            <p class="text-xs text-muted-foreground">{{ t('settings.basic.themeHint') }}</p>
          </div>


        </CardContent>
      </Card>

      <!-- 翻译语言配置 -->
      <Card>
        <CardHeader>
          <CardTitle>{{ t('settings.language.title') }}</CardTitle>
          <CardDescription>{{ t('settings.language.description') }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="rounded-md border bg-muted/30 p-3">
            <p class="text-sm text-muted-foreground">
              <strong>{{ t('settings.language.howItWorks') }}</strong>{{ t('settings.language.howItWorksDesc') }}
            </p>
          </div>
          <div class="grid gap-4 sm:grid-cols-2">
            <div class="space-y-2">
              <Label for="primaryTarget">{{ t('settings.language.primaryLanguage') }}</Label>
              <LanguageSelector id="primaryTarget" v-model="settingsForm.primaryTarget"
                :placeholder="t('settings.language.selectLanguage')" @update:modelValue="savePrimaryTarget" />
              <p class="text-xs text-muted-foreground">{{ t('settings.language.primaryLanguageHint') }}</p>
            </div>
            <div class="space-y-2">
              <Label for="secondaryTarget">{{ t('settings.language.secondaryLanguage') }}</Label>
              <LanguageSelector id="secondaryTarget" v-model="settingsForm.secondaryTarget"
                :placeholder="t('settings.language.selectLanguage')" @update:modelValue="saveSecondaryTarget" />
              <p class="text-xs text-muted-foreground">{{ t('settings.language.secondaryLanguageHint') }}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- 快捷键配置 -->
      <Card>
        <CardHeader>
          <CardTitle>{{ t('settings.hotkey.title') }}</CardTitle>
          <CardDescription>{{ t('settings.hotkey.description') }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium">{{ t('settings.hotkey.doubleCopy') }}</p>
              <p class="text-xs text-muted-foreground">
                {{ isMac ? t('settings.hotkey.doubleCopyDescMac') : t('settings.hotkey.doubleCopyDescWin') }}
              </p>
            </div>
            <Switch :checked="hotkeyConfig.double_copy_enabled"
              @update:checked="(v) => updateHotkeyConfig('double_copy_enabled', v)" />
          </div>
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium">{{ t('settings.hotkey.altSpace') }}</p>
              <p class="text-xs text-muted-foreground">
                {{ isMac ? t('settings.hotkey.altSpaceDescMac') : t('settings.hotkey.altSpaceDescWin') }}
              </p>
            </div>
            <Switch :checked="hotkeyConfig.alt_space_enabled"
              @update:checked="(v) => updateHotkeyConfig('alt_space_enabled', v)" />
          </div>
        </CardContent>
      </Card>




    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { Loader2 } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settings'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Select } from '@/components/ui/select'
import LanguageSelector from '../common/LanguageSelector.vue'
import { showToast } from '@/lib/toast'
import { isTauriEnv } from '@/utils/env'
import { supportedLocales, saveLocale, type SupportedLocale } from '@/locales'

const { t, locale } = useI18n()

interface ProviderConfig {
  provider_name: string
  enabled: boolean
  api_key: string
  model: string
  base_url: string | null
}

interface ProviderInfo {
  name: string
  display_name: string
  config: ProviderConfig
  available_models: string[]
  supports_base_url: boolean
}

interface ApiTestResponse {
  success: boolean
  status_code: number
  response_time_ms: number
  raw_response: any
  request_payload: any
  error: string | null
  provider: string
  model: string
}

interface SettingsForm {
  theme: 'light' | 'dark'
  primaryTarget: string
  secondaryTarget: string
}

interface HotkeyConfig {
  double_copy_enabled: boolean
  alt_space_enabled: boolean
}

const settingsStore = useSettingsStore()
const settingsForm = ref<SettingsForm>({
  theme: 'dark',
  primaryTarget: 'zh-CN',
  secondaryTarget: 'en',
})

const hotkeyConfig = ref<HotkeyConfig>({
  double_copy_enabled: true,
  alt_space_enabled: true,
})

// 检测是否为 macOS
const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0

const isSaving = ref(false)

// 服务商相关状态
const providers = ref<ProviderInfo[]>([])
const activeProvider = ref('zhipu')
const isSavingProvider = ref(false)
const isTestingProvider = ref(false)
const testResult = ref<ApiTestResponse | null>(null)

const currentProvider = computed(() => {
  return providers.value.find(p => p.name === activeProvider.value)
})

const applyThemeClass = (theme: string) => {
  if (theme === 'dark') {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

const loadProviderConfigs = async () => {
  if (!isTauriEnv()) return
  try {
    const configs = await invoke<ProviderInfo[]>('get_provider_configs')
    providers.value = configs
    if (configs.length > 0 && !configs.find(p => p.name === activeProvider.value)) {
      activeProvider.value = configs[0].name
    }
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('settings.loadFailed')}：${errMsg}`, 'error')
  }
}

const updateProviderEnabled = (enabled: boolean) => {
  if (currentProvider.value) {
    if (currentProvider.value.name === 'zhipu') {
      currentProvider.value.config.enabled = true
      return
    }
    currentProvider.value.config.enabled = enabled
  }
}

const updateProviderModel = (model: string) => {
  if (currentProvider.value) {
    currentProvider.value.config.model = model
  }
}

const updateProviderBaseUrl = (url: string | number) => {
  if (currentProvider.value) {
    currentProvider.value.config.base_url = typeof url === 'string' && url ? url : null
  }
}

const saveCurrentProvider = async () => {
  if (!isTauriEnv() || !currentProvider.value) return
  isSavingProvider.value = true
  try {
    await invoke('save_provider_config', {
      config: {
        provider_name: currentProvider.value.name,
        enabled: currentProvider.value.config.enabled,
        api_key: currentProvider.value.config.api_key,
        model: currentProvider.value.config.model || currentProvider.value.available_models[0],
        base_url: currentProvider.value.config.base_url || null,
      }
    })
    showToast(t('settings.providerConfig.configSaved', { provider: currentProvider.value.display_name }), 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('settings.providerConfig.saveFailed')}：${errMsg}`, 'error')
  } finally {
    isSavingProvider.value = false
  }
}

const testCurrentProvider = async () => {
  if (!isTauriEnv() || !currentProvider.value) return
  isTestingProvider.value = true
  testResult.value = null
  try {
    const result = await invoke<ApiTestResponse>('test_provider', {
      config: {
        provider_name: currentProvider.value.name,
        enabled: true,
        api_key: currentProvider.value.config.api_key,
        model: currentProvider.value.config.model || currentProvider.value.available_models[0],
        base_url: currentProvider.value.config.base_url || null,
      }
    })
    testResult.value = result
    if (result.success) {
      showToast(`${currentProvider.value.display_name} ${t('settings.providerConfig.connectionSuccess')}`, 'info')
    } else {
      if (result.status_code === 401) {
        showToast(`${t('settings.providerConfig.connectionFailed')}：API Key 无效或过期，请检查配置`, 'error')
      } else {
        showToast(`${currentProvider.value.display_name} ${t('settings.providerConfig.connectionFailed')}`, 'error')
      }
    }
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    testResult.value = {
      success: false,
      status_code: 0,
      response_time_ms: 0,
      raw_response: null,
      request_payload: null,
      error: errMsg,
      provider: currentProvider.value.name,
      model: currentProvider.value.config.model,
    }
    showToast(`${t('settings.providerConfig.testFailed')}：${errMsg}`, 'error')
  } finally {
    isTestingProvider.value = false
  }
}



const loadSettings = async () => {
  if (!isTauriEnv()) return
  try {
    const loadedSettings: any = await invoke('get_settings')
    settingsForm.value.theme = loadedSettings.theme || 'dark'
    settingsForm.value.primaryTarget = loadedSettings.primary_target || 'zh-CN'
    settingsForm.value.secondaryTarget = loadedSettings.secondary_target || 'en'
    settingsStore.setTheme(settingsForm.value.theme)
    settingsStore.setPrimaryTarget(settingsForm.value.primaryTarget)
    settingsStore.setSecondaryTarget(settingsForm.value.secondaryTarget)
    applyThemeClass(settingsForm.value.theme)
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('settings.loadFailed')}：${errMsg}`, 'error')
  }
}

const loadHotkeyConfig = async () => {
  if (!isTauriEnv()) return
  try {
    const config = await invoke<HotkeyConfig>('get_hotkey_config')
    hotkeyConfig.value = config
  } catch (error: any) {
    console.error('加载快捷键配置失败:', error)
  }
}

const updateHotkeyConfig = async (key: keyof HotkeyConfig, value: boolean) => {
  hotkeyConfig.value[key] = value
  if (!isTauriEnv()) return
  try {
    await invoke('save_hotkey_config', { config: hotkeyConfig.value })
    showToast(t('settings.hotkey.configSaved'), 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('settings.saveFailed')}：${errMsg}`, 'error')
  }
}

const saveSettings = async () => {
  if (!isTauriEnv()) {
    showToast(t('settings.notInTauri'), 'error')
    return
  }
  isSaving.value = true
  try {
    await invoke('save_settings', {
      settings: {
        api_key: '', // Legacy support
        theme: settingsForm.value.theme,
        primary_target: settingsForm.value.primaryTarget,
        secondary_target: settingsForm.value.secondaryTarget,
      },
    })
    settingsStore.setTheme(settingsForm.value.theme)
    settingsStore.setPrimaryTarget(settingsForm.value.primaryTarget)
    settingsStore.setSecondaryTarget(settingsForm.value.secondaryTarget)
    applyThemeClass(settingsForm.value.theme)
    showToast(t('settings.saved'), 'info')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('settings.saveFailed')}：${errMsg}`, 'error')
  } finally {
    isSaving.value = false
  }
}

const savePrimaryTarget = async (lang: string) => {
  settingsForm.value.primaryTarget = lang
  await saveSettings()
}

const saveSecondaryTarget = async (lang: string) => {
  settingsForm.value.secondaryTarget = lang
  await saveSettings()
}

const updateTheme = async (theme: 'light' | 'dark') => {
  settingsForm.value.theme = theme
  applyThemeClass(theme)
  await saveSettings()
}

onMounted(() => {
  loadSettings()
  loadProviderConfigs()
  loadHotkeyConfig()
})

watch(
  () => settingsForm.value.theme,
  (theme) => applyThemeClass(theme),
)

// 切换服务商时清除测试结果
watch(activeProvider, () => {
  testResult.value = null
})

const resetDefaults = async () => {
  settingsForm.value = {
    theme: 'dark',
    primaryTarget: 'zh-CN',
    secondaryTarget: 'en',
  }
  applyThemeClass(settingsForm.value.theme)
  await saveSettings()
  showToast(t('settings.basic.defaultsRestored'), 'info')
}



// 语言切换选项
const localeOptions = computed(() =>
  supportedLocales.map(l => ({ label: l.name, value: l.code }))
)

// 切换界面语言
const changeLocale = (newLocale: string) => {
  locale.value = newLocale as SupportedLocale
  saveLocale(newLocale as SupportedLocale)
  showToast(t('settings.interfaceLanguage.saved'), 'info')
}
</script>
