<template>
  <div class="p-6 bg-background">
    <!-- 全屏 Loading -->
    <div v-if="isLoading" class="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
      <div class="flex flex-col items-center gap-4">
        <Loader2 class="h-8 w-8 animate-spin text-primary" />
        <p class="text-sm text-muted-foreground">{{ t('settings.loading') }}</p>
      </div>
    </div>

    <!-- 配置加载失败弹窗 -->
    <Dialog v-model:open="showErrorDialog">
      <DialogContent class="max-w-md">
        <DialogHeader>
          <DialogTitle class="text-destructive">{{ t('settings.loadFailed') }}</DialogTitle>
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

    <!-- 新增自定义服务商 -->
    <Dialog v-model:open="showCreateProviderDialog">
      <DialogContent class="max-w-md">
        <DialogHeader>
          <DialogTitle>{{ t('settings.providerConfig.addProviderTitle') }}</DialogTitle>
          <DialogDescription>
            {{ t('settings.providerConfig.addProviderDesc') }}
          </DialogDescription>
        </DialogHeader>
        <div class="space-y-3">
          <div class="space-y-2">
            <Label for="new-provider-name">{{ t('settings.providerConfig.providerName') }}</Label>
            <Input id="new-provider-name" v-model="createProviderForm.providerName"
              :placeholder="t('settings.providerConfig.providerNamePlaceholder')" />
          </div>
          <div class="space-y-2">
            <Label for="new-provider-model">{{ t('settings.providerConfig.model') }}</Label>
            <Input id="new-provider-model" v-model="createProviderForm.model"
              :placeholder="t('settings.providerConfig.modelPlaceholder')" />
          </div>
          <div class="space-y-2">
            <Label for="new-provider-url">{{ t('settings.providerConfig.baseUrl') }}</Label>
            <Input id="new-provider-url" v-model="createProviderForm.baseUrl"
              :placeholder="'https://api.example.com/v1/chat/completions'" />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" @click="showCreateProviderDialog = false">
            {{ t('common.cancel') }}
          </Button>
          <Button :disabled="isCreatingProvider" @click="createCustomProvider">
            <Loader2 v-if="isCreatingProvider" class="mr-2 h-4 w-4 animate-spin" />
            {{ t('settings.providerConfig.createProvider') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- 模型测速配置 -->
    <Dialog v-model:open="showBenchmarkDialog">
      <DialogContent class="max-w-lg">
        <DialogHeader>
          <DialogTitle>{{ t('settings.providerConfig.benchmarkDialogTitle') }}</DialogTitle>
          <DialogDescription>
            {{ t('settings.providerConfig.benchmarkDialogDesc') }}
          </DialogDescription>
        </DialogHeader>

        <div class="max-h-72 space-y-2 overflow-y-auto pr-1">
          <div
            v-for="(model, index) in benchmarkModelsDraft"
            :key="`benchmark-model-${index}`"
            class="flex items-center gap-2"
          >
            <Input
              :model-value="model"
              :placeholder="t('settings.providerConfig.modelPlaceholder')"
              @update:model-value="(value) => updateBenchmarkModel(index, value)"
            />
            <Button variant="outline" size="sm" @click="removeBenchmarkModel(index)">
              {{ t('common.delete') }}
            </Button>
          </div>
          <Button variant="outline" size="sm" @click="addBenchmarkModel">
            {{ t('settings.providerConfig.addBenchmarkModel') }}
          </Button>
        </div>

        <DialogFooter>
          <Button variant="outline" :disabled="isBenchmarkingModels" @click="showBenchmarkDialog = false">
            {{ t('common.cancel') }}
          </Button>
          <Button :disabled="isBenchmarkingModels" @click="runBenchmarkWithDialogModels">
            <Loader2 v-if="isBenchmarkingModels" class="mr-2 h-4 w-4 animate-spin" />
            {{ t('settings.providerConfig.startBenchmark') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <div class="mx-auto flex max-w-3xl flex-col gap-6" :class="{ 'opacity-50': isLoading }">
      <!-- 服务商配置 -->
      <Card>
        <CardHeader>
          <CardTitle>{{ t('settings.providerConfig.title') }}</CardTitle>
          <CardDescription>{{ t('settings.providerConfig.description') }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <!-- 服务商 Tab 栏 -->
          <Tabs v-model="activeProvider" class="w-full">
            <TabsList class="flex w-full flex-wrap gap-2 rounded-md border bg-muted/40 p-1">
              <TabsTrigger v-for="provider in providers" :key="provider.name" :value="provider.name"
                class="relative flex-1 min-w-[120px] justify-center">
                <span class="truncate">{{ provider.display_name }}</span>
                <span v-if="provider.config.enabled" class="absolute right-2 h-2 w-2 rounded-full bg-green-500" />
              </TabsTrigger>
              <Button variant="outline" size="sm" class="h-8 shrink-0" @click="openCreateProviderDialog">
                {{ t('settings.providerConfig.addProvider') }}
              </Button>
            </TabsList>
          </Tabs>

          <!-- 当前服务商配置 -->
          <div v-if="currentProvider" class="space-y-4 pt-2">
            <!-- 启用开关 -->
            <div class="flex items-center justify-between">
              <div>
                <Label>{{ t('settings.providerConfig.enable') }} {{ currentProvider.display_name }}</Label>
                <p class="text-xs text-muted-foreground">
                  {{ t('settings.providerConfig.enableDesc') }}
                </p>
              </div>
              <div class="flex items-center gap-2">
                <Button v-if="!currentProvider.is_preset" variant="outline" size="sm" @click="deleteCurrentProvider">
                  {{ t('settings.providerConfig.deleteProvider') }}
                </Button>
                <Switch :model-value="currentProvider.config.enabled" @update:model-value="onProviderToggle" />
              </div>
            </div>

            <!-- API Key -->
            <div v-if="currentProvider.name !== 'ollama'" class="space-y-2">
              <Label :for="`${currentProvider.name}-apikey`">
                {{ t('settings.providerConfig.apiKey') }}
              </Label>
              <div class="flex gap-2">
                <Input :id="`${currentProvider.name}-apikey`" v-model="currentProvider.config.api_key" type="password"
                  :placeholder="t('settings.providerConfig.apiKeyPlaceholder', { provider: currentProvider.display_name })" />
              </div>
              <p v-if="currentProvider.name === 'zhipu' && currentProvider.config.model.toLowerCase().includes('flash')"
                class="text-xs text-muted-foreground">
                {{ t('settings.providerConfig.zhipuFlashHint') }}
              </p>
              <p v-if="currentProvider.api_key_optional" class="text-xs text-muted-foreground">
                {{ t('settings.providerConfig.apiKeyOptionalHint') }}
              </p>
              <a v-if="currentProvider.name === 'zhipu'"
                class="text-xs font-semibold text-orange-500 underline underline-offset-2 hover:text-orange-400"
                href="https://www.bigmodel.cn/glm-coding?ic=TJOZG6ZJMM" target="_blank" rel="noopener noreferrer">
                推荐填写个人 Key，避免共用系统 Key 导致限速，影响体验。前往注册获取Api Key
              </a>
            </div>

            <!-- 模型配置 -->
            <div class="space-y-2">
              <Label :for="`${currentProvider.name}-model`">{{ t('settings.providerConfig.model') }}</Label>
              <Input :id="`${currentProvider.name}-model`" v-model="currentProvider.config.model"
                :placeholder="t('settings.providerConfig.modelPlaceholder')" />
              <p v-if="currentProvider.name === 'ollama'" class="text-xs text-muted-foreground">
                {{ t('settings.providerConfig.ollamaModelHint') }}
              </p>
              <p class="text-xs text-muted-foreground">{{ t('settings.providerConfig.customModelHint') }}</p>
            </div>

            <!-- 自定义 API 地址 -->
            <div v-if="currentProvider.supports_base_url" class="space-y-2">
              <Label :for="`${currentProvider.name}-baseurl`">{{ t('settings.providerConfig.baseUrl') }}</Label>
              <Input :id="`${currentProvider.name}-baseurl`" :model-value="currentProvider.config.base_url || ''"
                :placeholder="currentProvider.default_base_url || 'https://api.openai.com/v1/chat/completions'"
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
              <Button variant="outline" :disabled="isBenchmarkingModels" @click="openBenchmarkDialog">
                <Loader2 v-if="isBenchmarkingModels" class="mr-2 h-4 w-4 animate-spin" />
                {{ t('settings.providerConfig.testAllModels') }}
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

            <!-- 模型测速结果 -->
            <div class="space-y-3 rounded-md border p-3">
              <div class="flex items-center justify-between">
                <span class="font-medium">{{ t('settings.providerConfig.modelLatencyTitle') }}</span>
                <span class="text-xs text-muted-foreground">{{ t('settings.providerConfig.modelLatencyLegend') }}</span>
              </div>
              <div v-if="isBenchmarkingModels" class="flex items-center gap-2 text-sm text-muted-foreground">
                <Loader2 class="h-4 w-4 animate-spin" />
                <span>{{ t('settings.providerConfig.testingModels') }}</span>
              </div>
              <div v-if="modelSpeedResults.length" class="space-y-2">
                <div v-for="result in modelSpeedResults" :key="`${result.provider}-${result.model}`"
                  class="space-y-2 rounded-md border px-3 py-2">
                  <div class="flex items-center justify-between">
                    <div class="flex flex-col">
                      <span class="font-medium">{{ result.model }}</span>
                      <span v-if="!result.success" class="text-xs text-red-500">
                        {{ t('settings.providerConfig.modelRequestFailed') }}
                      </span>
                      <span v-else class="text-xs text-muted-foreground">HTTP {{ result.status_code }}</span>
                    </div>
                    <div class="text-right">
                      <span v-if="result.success"
                        :class="['text-sm font-semibold', getLatencyColor(result.response_time_ms)]">
                        {{ (result.response_time_ms / 1000).toFixed(2) }}s
                      </span>
                      <div v-else class="text-right">
                        <div class="text-sm font-semibold text-red-500">
                          {{ t('settings.providerConfig.modelRequestFailed') }}
                        </div>
                        <div v-if="result.error" class="text-xs text-muted-foreground max-w-[220px] truncate"
                          :title="result.error">
                          {{ result.error }}
                        </div>
                      </div>
                    </div>
                  </div>
                  <details v-if="result.request_payload" class="text-xs">
                    <summary class="cursor-pointer text-muted-foreground hover:text-foreground">
                      {{ t('settings.providerConfig.viewRequestPayload') }}
                    </summary>
                    <pre
                      class="mt-2 max-h-48 overflow-auto rounded bg-muted/40 p-2">{{ JSON.stringify(result.request_payload, null, 2) }}</pre>
                  </details>
                  <details v-if="result.raw_response" class="text-xs">
                    <summary class="cursor-pointer text-muted-foreground hover:text-foreground">
                      {{ t('settings.providerConfig.viewRawResponse') }}
                    </summary>
                    <pre
                      class="mt-2 max-h-48 overflow-auto rounded bg-muted/40 p-2">{{ JSON.stringify(result.raw_response, null, 2) }}</pre>
                  </details>
                </div>
              </div>
              <p v-else class="text-sm text-muted-foreground">
                {{ t('settings.providerConfig.modelSpeedEmpty') }}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- 语音合成配置 -->
      <Card>
        <CardHeader>
          <CardTitle>{{ t('settings.speechConfig.title') }}</CardTitle>
          <CardDescription>{{ t('settings.speechConfig.description') }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <Tabs v-model="activeSpeechProvider" class="w-full">
            <TabsList class="flex w-full flex-wrap gap-2 rounded-md border bg-muted/40 p-1">
              <TabsTrigger
                v-for="provider in speechProviders"
                :key="provider.name"
                :value="provider.name"
                class="relative flex-1 min-w-[120px] justify-center"
              >
                <span class="truncate">{{ provider.display_name }}</span>
                <span v-if="provider.config.enabled" class="absolute right-2 h-2 w-2 rounded-full bg-green-500" />
              </TabsTrigger>
            </TabsList>
          </Tabs>

          <div v-if="currentSpeechProvider" class="space-y-4 pt-2">
            <div class="flex items-center justify-between">
              <div>
                <Label>{{ t('settings.speechConfig.enable') }} {{ currentSpeechProvider.display_name }}</Label>
                <p class="text-xs text-muted-foreground">
                  {{ t('settings.speechConfig.enableDesc') }}
                </p>
              </div>
              <Switch :model-value="currentSpeechProvider.config.enabled" @update:model-value="onSpeechProviderToggle" />
            </div>

            <div class="space-y-2">
              <Label :for="`${currentSpeechProvider.name}-speech-apikey`">
                {{ t('settings.speechConfig.apiKey') }}
              </Label>
              <Input
                :id="`${currentSpeechProvider.name}-speech-apikey`"
                v-model="currentSpeechProvider.config.api_key"
                type="password"
                :placeholder="t('settings.speechConfig.apiKeyPlaceholder', { provider: currentSpeechProvider.display_name })"
              />
              <p v-if="currentSpeechProvider.api_key_optional" class="text-xs text-muted-foreground">
                {{ t('settings.speechConfig.apiKeyOptionalHint') }}
              </p>
            </div>

            <div class="space-y-2">
              <Label :for="`${currentSpeechProvider.name}-speech-model`">{{ t('settings.speechConfig.model') }}</Label>
              <Input
                :id="`${currentSpeechProvider.name}-speech-model`"
                v-model="currentSpeechProvider.config.model"
                :placeholder="t('settings.speechConfig.modelPlaceholder')"
              />
            </div>

            <div class="space-y-2">
              <Label :for="`${currentSpeechProvider.name}-speech-voice`">{{ t('settings.speechConfig.voice') }}</Label>
              <Input
                :id="`${currentSpeechProvider.name}-speech-voice`"
                v-model="currentSpeechProvider.config.voice"
                :placeholder="t('settings.speechConfig.voicePlaceholder')"
              />
            </div>

            <div class="space-y-2">
              <Label :for="`${currentSpeechProvider.name}-speech-format`">{{ t('settings.speechConfig.audioFormat') }}</Label>
              <Input
                :id="`${currentSpeechProvider.name}-speech-format`"
                v-model="currentSpeechProvider.config.audio_format"
                :placeholder="t('settings.speechConfig.audioFormatPlaceholder')"
              />
            </div>

            <div class="space-y-2">
              <Label :for="`${currentSpeechProvider.name}-speech-baseurl`">{{ t('settings.speechConfig.baseUrl') }}</Label>
              <Input
                :id="`${currentSpeechProvider.name}-speech-baseurl`"
                :model-value="currentSpeechProvider.config.base_url || ''"
                :placeholder="currentSpeechProvider.default_base_url || 'https://api.xiaomimimo.com/v1/chat/completions'"
                @update:model-value="updateSpeechProviderBaseUrl"
              />
              <p class="text-xs text-muted-foreground">{{ t('settings.speechConfig.baseUrlHint') }}</p>
            </div>

            <div class="flex gap-2 pt-2">
              <Button :disabled="isSavingSpeechProvider" @click="saveCurrentSpeechProvider">
                <Loader2 v-if="isSavingSpeechProvider" class="mr-2 h-4 w-4 animate-spin" />
                {{ t('settings.speechConfig.saveConfig') }}
              </Button>
              <Button variant="outline" :disabled="isTestingSpeechProvider" @click="testCurrentSpeechProvider">
                <Loader2 v-if="isTestingSpeechProvider" class="mr-2 h-4 w-4 animate-spin" />
                {{ t('settings.speechConfig.testConnection') }}
              </Button>
            </div>

            <div v-if="speechTestResult" class="space-y-2 rounded-md border p-3">
              <div class="flex items-center gap-2">
                <span :class="speechTestResult.success ? 'text-green-500' : 'text-red-500'" class="text-sm font-medium">
                  {{ speechTestResult.success ? t('settings.speechConfig.connectionSuccess') :
                    t('settings.speechConfig.connectionFailed') }}
                </span>
                <span class="text-xs text-muted-foreground">
                  HTTP {{ speechTestResult.status_code }} | {{ speechTestResult.response_time_ms }}ms
                </span>
              </div>
              <div v-if="speechTestResult.error" class="text-xs text-red-500">
                {{ speechTestResult.error }}
              </div>
              <details v-if="speechTestResult.request_payload" class="text-xs">
                <summary class="cursor-pointer text-muted-foreground hover:text-foreground">
                  {{ t('settings.speechConfig.viewRequestPayload') }}
                </summary>
                <pre
                  class="mt-2 max-h-48 overflow-auto rounded bg-muted/40 p-2">{{ JSON.stringify(speechTestResult.request_payload, null, 2) }}</pre>
              </details>
              <details v-if="speechTestResult.raw_response" class="text-xs">
                <summary class="cursor-pointer text-muted-foreground hover:text-foreground">
                  {{ t('settings.speechConfig.viewRawResponse') }}
                </summary>
                <pre
                  class="mt-2 max-h-48 overflow-auto rounded bg-muted/40 p-2">{{ JSON.stringify(speechTestResult.raw_response, null, 2) }}</pre>
              </details>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- 权限助手（macOS） -->
      <Card v-if="isMac">
        <CardHeader class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <CardTitle>{{ t('settings.permissions.title') }}</CardTitle>
            <CardDescription>{{ t('settings.permissions.description') }}</CardDescription>
          </div>
          <Button variant="outline" @click="openPermissionHelper">
            {{ t('settings.permissions.open') }}
          </Button>
        </CardHeader>
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
            <Select v-model="selectedLocale">
              <SelectTrigger>
                <SelectValue :placeholder="t('settings.interfaceLanguage.select')" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem v-for="option in localeOptions" :key="option.value" :value="option.value">
                    {{ option.label }}
                  </SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
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
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import LanguageSelector from '../common/LanguageSelector.vue'
import { showToast } from '@/lib/toast'
import { isTauriEnv } from '@/utils/env'
import { parseBackendError } from '@/utils/backendError'
import { getFromStorage, saveToStorage } from '@/services/storage'
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
  is_preset: boolean
  api_key_optional: boolean
  default_base_url: string | null
}

interface SpeechProviderConfig {
  provider_name: string
  enabled: boolean
  api_key: string
  model: string
  base_url: string | null
  voice: string
  audio_format: string
}

interface SpeechProviderInfo {
  name: string
  display_name: string
  config: SpeechProviderConfig
  is_preset: boolean
  api_key_optional: boolean
  default_base_url: string | null
  default_model: string
  default_voice: string
  default_audio_format: string
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
  locale: SupportedLocale
}

const settingsStore = useSettingsStore()
const settingsForm = ref<SettingsForm>({
  theme: 'dark',
  primaryTarget: 'zh-CN',
  secondaryTarget: 'en',
  locale: 'zh-CN',
})

// 检测是否为 macOS
const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0

const isSaving = ref(false)
const isLoading = ref(false)
const showErrorDialog = ref(false)
const errorMessage = ref('')
const showCreateProviderDialog = ref(false)
const isCreatingProvider = ref(false)
const createProviderForm = ref({
  providerName: '',
  model: '',
  baseUrl: '',
})

// 服务商相关状态
const providers = ref<ProviderInfo[]>([])
const activeProvider = ref('zhipu')
const isSavingProvider = ref(false)
const isTestingProvider = ref(false)
const isBenchmarkingModels = ref(false)
const testResult = ref<ApiTestResponse | null>(null)
const modelSpeedResults = ref<ApiTestResponse[]>([])
const showBenchmarkDialog = ref(false)
const benchmarkModelsDraft = ref<string[]>([])
const benchmarkProviderName = ref('')
const speechProviders = ref<SpeechProviderInfo[]>([])
const activeSpeechProvider = ref('xiaomi')
const isSavingSpeechProvider = ref(false)
const isTestingSpeechProvider = ref(false)
const speechTestResult = ref<ApiTestResponse | null>(null)

const MODEL_BENCHMARK_CACHE_KEY = 'provider-model-benchmark-cache-v1'

const getErrorMessage = (error: unknown) => {
  const parsed = parseBackendError(error)
  return parsed.message || parsed.raw
}

const getErrorCode = (error: unknown) => {
  const parsed = parseBackendError(error)
  return parsed.code
}

type BenchmarkCache = Record<string, string[]>

const loadBenchmarkCache = (): BenchmarkCache => {
  try {
    const data = getFromStorage(MODEL_BENCHMARK_CACHE_KEY)
    if (!data || typeof data !== 'object') return {}
    return data as BenchmarkCache
  } catch (_) {
    return {}
  }
}

const saveBenchmarkCache = (cache: BenchmarkCache) => {
  saveToStorage(MODEL_BENCHMARK_CACHE_KEY, cache)
}

const normalizeBenchmarkModels = (models: string[]) => {
  const seen = new Set<string>()
  const result: string[] = []
  models.forEach((raw) => {
    const model = raw.trim()
    if (!model || seen.has(model)) return
    seen.add(model)
    result.push(model)
  })
  return result
}

const isLocalhostBaseUrl = (baseUrl: string) => {
  try {
    const url = new URL(baseUrl)
    const host = url.hostname
    return host === 'localhost' || host === '127.0.0.1' || host === '::1'
  } catch (_) {
    return false
  }
}

const canEnableProvider = (provider: ProviderInfo) => {
  const model = provider.config.model.trim()
  if (!model) {
    showToast(t('settings.providerConfig.enableValidationModel'), 'error')
    return false
  }

  const resolvedBaseUrl = (provider.config.base_url || provider.default_base_url || '').trim()
  if (!resolvedBaseUrl) {
    showToast(t('settings.providerConfig.enableValidationBaseUrl'), 'error')
    return false
  }

  if (!resolvedBaseUrl.startsWith('http://') && !resolvedBaseUrl.startsWith('https://')) {
    showToast(t('settings.providerConfig.enableValidationBaseUrlFormat'), 'error')
    return false
  }

  if (!provider.api_key_optional && !provider.config.api_key.trim() && !isLocalhostBaseUrl(resolvedBaseUrl)) {
    showToast(t('settings.providerConfig.enableValidationApiKey'), 'error')
    return false
  }

  return true
}

const currentProvider = computed(() => {
  return providers.value.find(p => p.name === activeProvider.value)
})

const currentSpeechProvider = computed(() => {
  return speechProviders.value.find(p => p.name === activeSpeechProvider.value)
})

const applyThemeClass = (theme: string) => {
  if (theme === 'dark') {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

const loadProviderConfigs = async (): Promise<void> => {
  if (!isTauriEnv()) return
  try {
    const configs = await invoke<ProviderInfo[]>('get_provider_configs')
    providers.value = configs
    if (configs.length > 0 && !configs.find(p => p.name === activeProvider.value)) {
      activeProvider.value = configs[0].name
    }
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    throw new Error(errMsg)
  }
}

const loadSpeechProviderConfigs = async (): Promise<void> => {
  if (!isTauriEnv()) return
  try {
    const configs = await invoke<SpeechProviderInfo[]>('get_speech_provider_configs')
    speechProviders.value = configs
    if (configs.length > 0 && !configs.find(p => p.name === activeSpeechProvider.value)) {
      activeSpeechProvider.value = configs[0].name
    }
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    throw new Error(errMsg)
  }
}

const updateProviderEnabled = (enabled: boolean) => {
  const current = currentProvider.value
  if (!current) return

  if (enabled) {
    providers.value = providers.value.map((provider) => ({
      ...provider,
      config: {
        ...provider.config,
        enabled: provider.name === current.name,
      },
    }))
    return
  }

  current.config.enabled = false
}

const onProviderToggle = async (enabled: boolean) => {
  if (enabled && currentProvider.value && !canEnableProvider(currentProvider.value)) {
    return
  }
  updateProviderEnabled(enabled)
  await saveCurrentProvider()
}

const canEnableSpeechProvider = (provider: SpeechProviderInfo) => {
  const model = provider.config.model.trim()
  if (!model) {
    showToast(t('settings.speechConfig.enableValidationModel'), 'error')
    return false
  }

  const voice = provider.config.voice.trim()
  if (!voice) {
    showToast(t('settings.speechConfig.enableValidationVoice'), 'error')
    return false
  }

  const format = provider.config.audio_format.trim()
  if (!format) {
    showToast(t('settings.speechConfig.enableValidationAudioFormat'), 'error')
    return false
  }

  const resolvedBaseUrl = (provider.config.base_url || provider.default_base_url || '').trim()
  if (!resolvedBaseUrl) {
    showToast(t('settings.speechConfig.enableValidationBaseUrl'), 'error')
    return false
  }

  if (!resolvedBaseUrl.startsWith('http://') && !resolvedBaseUrl.startsWith('https://')) {
    showToast(t('settings.speechConfig.enableValidationBaseUrlFormat'), 'error')
    return false
  }

  if (!provider.api_key_optional && !provider.config.api_key.trim() && !isLocalhostBaseUrl(resolvedBaseUrl)) {
    showToast(t('settings.speechConfig.enableValidationApiKey'), 'error')
    return false
  }

  return true
}

const updateSpeechProviderEnabled = (enabled: boolean) => {
  const current = currentSpeechProvider.value
  if (!current) return

  if (enabled) {
    speechProviders.value = speechProviders.value.map((provider) => ({
      ...provider,
      config: {
        ...provider.config,
        enabled: provider.name === current.name,
      },
    }))
    return
  }

  current.config.enabled = false
}

const onSpeechProviderToggle = async (enabled: boolean) => {
  if (enabled && currentSpeechProvider.value && !canEnableSpeechProvider(currentSpeechProvider.value)) {
    return
  }
  updateSpeechProviderEnabled(enabled)
  await saveCurrentSpeechProvider()
}

const updateProviderBaseUrl = (url: string | number) => {
  if (currentProvider.value) {
    currentProvider.value.config.base_url = typeof url === 'string' && url.trim() ? url.trim() : null
  }
}

const updateSpeechProviderBaseUrl = (url: string | number) => {
  if (currentSpeechProvider.value) {
    currentSpeechProvider.value.config.base_url = typeof url === 'string' && url.trim() ? url.trim() : null
  }
}

const saveCurrentProvider = async () => {
  if (!isTauriEnv() || !currentProvider.value) return
  if (currentProvider.value.config.enabled && !canEnableProvider(currentProvider.value)) return
  isSavingProvider.value = true
  const currentProviderName = currentProvider.value.name
  const currentProviderDisplayName = currentProvider.value.display_name
  try {
    await invoke('save_provider_config', {
      config: {
        provider_name: currentProvider.value.name,
        enabled: currentProvider.value.config.enabled,
        api_key: currentProvider.value.config.api_key,
        model: currentProvider.value.config.model.trim(),
        base_url: currentProvider.value.config.base_url || null,
      }
    })
    await loadProviderConfigs()
    activeProvider.value = currentProviderName
    showToast(t('settings.providerConfig.configSaved', { provider: currentProviderDisplayName }), 'info')
  } catch (error: any) {
    const errorCode = getErrorCode(error)
    const errMsg = getErrorMessage(error)
    await loadProviderConfigs()
    activeProvider.value = currentProviderName
    if (errorCode === 'INVALID_PROVIDER_CONFIG') {
      showToast(errMsg, 'error')
    } else {
      showToast(`${t('settings.providerConfig.saveFailed')}：${errMsg}`, 'error')
    }
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
        model: currentProvider.value.config.model.trim(),
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
    const errMsg = getErrorMessage(error)
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

const saveCurrentSpeechProvider = async () => {
  if (!isTauriEnv() || !currentSpeechProvider.value) return
  if (currentSpeechProvider.value.config.enabled && !canEnableSpeechProvider(currentSpeechProvider.value)) return
  isSavingSpeechProvider.value = true
  const currentProviderName = currentSpeechProvider.value.name
  const currentProviderDisplayName = currentSpeechProvider.value.display_name
  try {
    await invoke('save_speech_provider_config', {
      config: {
        provider_name: currentSpeechProvider.value.name,
        enabled: currentSpeechProvider.value.config.enabled,
        api_key: currentSpeechProvider.value.config.api_key,
        model: currentSpeechProvider.value.config.model.trim(),
        base_url: currentSpeechProvider.value.config.base_url || null,
        voice: currentSpeechProvider.value.config.voice.trim(),
        audio_format: currentSpeechProvider.value.config.audio_format.trim().toLowerCase(),
      },
    })
    await loadSpeechProviderConfigs()
    activeSpeechProvider.value = currentProviderName
    showToast(t('settings.speechConfig.configSaved', { provider: currentProviderDisplayName }), 'info')
  } catch (error: any) {
    const errMsg = getErrorMessage(error)
    await loadSpeechProviderConfigs()
    activeSpeechProvider.value = currentProviderName
    showToast(`${t('settings.speechConfig.saveFailed')}：${errMsg}`, 'error')
  } finally {
    isSavingSpeechProvider.value = false
  }
}

const testCurrentSpeechProvider = async () => {
  if (!isTauriEnv() || !currentSpeechProvider.value) return
  isTestingSpeechProvider.value = true
  speechTestResult.value = null
  try {
    const result = await invoke<ApiTestResponse>('test_speech_provider', {
      config: {
        provider_name: currentSpeechProvider.value.name,
        enabled: true,
        api_key: currentSpeechProvider.value.config.api_key,
        model: currentSpeechProvider.value.config.model.trim(),
        base_url: currentSpeechProvider.value.config.base_url || null,
        voice: currentSpeechProvider.value.config.voice.trim(),
        audio_format: currentSpeechProvider.value.config.audio_format.trim().toLowerCase(),
      },
    })
    speechTestResult.value = result
    if (result.success) {
      showToast(`${currentSpeechProvider.value.display_name} ${t('settings.speechConfig.connectionSuccess')}`, 'info')
    } else {
      showToast(`${currentSpeechProvider.value.display_name} ${t('settings.speechConfig.connectionFailed')}`, 'error')
    }
  } catch (error: any) {
    const errMsg = getErrorMessage(error)
    speechTestResult.value = {
      success: false,
      status_code: 0,
      response_time_ms: 0,
      raw_response: null,
      request_payload: null,
      error: errMsg,
      provider: currentSpeechProvider.value.name,
      model: currentSpeechProvider.value.config.model,
    }
    showToast(`${t('settings.speechConfig.testFailed')}：${errMsg}`, 'error')
  } finally {
    isTestingSpeechProvider.value = false
  }
}

const openBenchmarkDialog = () => {
  if (!currentProvider.value) return
  const provider = currentProvider.value
  benchmarkProviderName.value = provider.name

  const cache = loadBenchmarkCache()
  const cachedModels = Array.isArray(cache[provider.name]) ? cache[provider.name] : []
  const configModel = provider.config.model.trim()
  const merged = normalizeBenchmarkModels(
    configModel ? [configModel, ...cachedModels] : cachedModels,
  )

  benchmarkModelsDraft.value = merged.length ? merged : ['']
  showBenchmarkDialog.value = true
}

const addBenchmarkModel = () => {
  benchmarkModelsDraft.value.push('')
}

const removeBenchmarkModel = (index: number) => {
  if (benchmarkModelsDraft.value.length <= 1) {
    benchmarkModelsDraft.value = ['']
    return
  }
  benchmarkModelsDraft.value.splice(index, 1)
}

const updateBenchmarkModel = (index: number, value: string | number) => {
  benchmarkModelsDraft.value[index] = String(value ?? '')
}

const runBenchmarkWithDialogModels = async () => {
  if (!isTauriEnv()) return
  const providerName = benchmarkProviderName.value
  const provider = providers.value.find((item) => item.name === providerName)
  if (!provider) {
    showToast(t('settings.providerConfig.benchmarkProviderMissing'), 'error')
    return
  }

  const modelsToTest = normalizeBenchmarkModels(benchmarkModelsDraft.value)
  if (!modelsToTest.length) {
    showToast(t('settings.providerConfig.noModelForBenchmark'), 'error')
    return
  }

  benchmarkModelsDraft.value = modelsToTest
  const cache = loadBenchmarkCache()
  cache[provider.name] = modelsToTest
  saveBenchmarkCache(cache)

  isBenchmarkingModels.value = true
  modelSpeedResults.value = []

  try {
    for (const model of modelsToTest) {
      try {
        const result = await invoke<ApiTestResponse>('test_provider', {
          config: {
            provider_name: provider.name,
            enabled: true,
            api_key: provider.config.api_key,
            model,
            base_url: provider.config.base_url || null,
          },
        })
        modelSpeedResults.value.push(result)
      } catch (error: any) {
        const errMsg = getErrorMessage(error)
        modelSpeedResults.value.push({
          success: false,
          status_code: 0,
          response_time_ms: 0,
          raw_response: null,
          request_payload: null,
          error: errMsg,
          provider: provider.display_name,
          model,
        })
      }
    }

    showBenchmarkDialog.value = false
  } catch (error: any) {
    const errMsg = getErrorMessage(error)
    modelSpeedResults.value.push({
      success: false,
      status_code: 0,
      response_time_ms: 0,
      raw_response: null,
      request_payload: null,
      error: errMsg,
      provider: provider.display_name,
      model: 'N/A',
    })
    showToast(`${t('settings.providerConfig.testFailed')}：${errMsg}`, 'error')
  } finally {
    isBenchmarkingModels.value = false
  }
}

const openCreateProviderDialog = () => {
  createProviderForm.value = {
    providerName: '',
    model: '',
    baseUrl: '',
  }
  showCreateProviderDialog.value = true
}

const createCustomProvider = async () => {
  if (!isTauriEnv()) return
  const providerName = createProviderForm.value.providerName.trim()
  if (!providerName) {
    showToast(t('settings.providerConfig.providerNameRequired'), 'error')
    return
  }

  isCreatingProvider.value = true
  try {
    await invoke('create_custom_provider', {
      providerName,
      model: createProviderForm.value.model.trim() || null,
      baseUrl: createProviderForm.value.baseUrl.trim() || null,
    })
    await loadProviderConfigs()
    activeProvider.value = providerName
    showCreateProviderDialog.value = false
    showToast(t('settings.providerConfig.providerCreated', { provider: providerName }), 'info')
  } catch (error: any) {
    const errMsg = getErrorMessage(error)
    showToast(`${t('settings.providerConfig.createFailed')}：${errMsg}`, 'error')
  } finally {
    isCreatingProvider.value = false
  }
}

const deleteCurrentProvider = async () => {
  if (!isTauriEnv() || !currentProvider.value || currentProvider.value.is_preset) return
  const providerName = currentProvider.value.name
  const providerDisplayName = currentProvider.value.display_name
  if (!window.confirm(t('settings.providerConfig.deleteConfirm', { provider: providerDisplayName }))) {
    return
  }

  try {
    await invoke('delete_custom_provider', { providerName })
    await loadProviderConfigs()
    showToast(t('settings.providerConfig.deleteSuccess', { provider: providerDisplayName }), 'info')
  } catch (error: any) {
    const errMsg = getErrorMessage(error)
    showToast(`${t('settings.providerConfig.deleteFailed')}：${errMsg}`, 'error')
  }
}

const getLatencyColor = (ms: number) => {
  if (ms < 1000) return 'text-green-500'
  if (ms < 2000) return 'text-yellow-500'
  return 'text-red-500'
}

const openPermissionHelper = async () => {
  if (!isTauriEnv()) {
    showToast(t('settings.notInTauri'), 'error')
    return
  }
  try {
    await invoke('show_permissions_window')
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('settings.permissions.openFailed')}：${errMsg}`, 'error')
  }
}



const loadSettings = async (): Promise<void> => {
  if (!isTauriEnv()) return
  try {
    const loadedSettings: any = await invoke('get_settings')
    settingsForm.value.theme = loadedSettings.theme || 'dark'
    settingsForm.value.primaryTarget = loadedSettings.primary_target || 'zh-CN'
    settingsForm.value.secondaryTarget = loadedSettings.secondary_target || 'en'
    settingsForm.value.locale = loadedSettings.locale || (locale.value as SupportedLocale) || 'zh-CN'
    settingsStore.setTheme(settingsForm.value.theme)
    settingsStore.setPrimaryTarget(settingsForm.value.primaryTarget)
    settingsStore.setSecondaryTarget(settingsForm.value.secondaryTarget)
    settingsStore.setLocale(settingsForm.value.locale)
    locale.value = settingsForm.value.locale
    applyThemeClass(settingsForm.value.theme)
  } catch (error: any) {
    const errMsg = error?.message || String(error)
    showToast(`${t('settings.loadFailed')}：${errMsg}`, 'error')
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
        locale: settingsForm.value.locale,
      },
    })
    settingsStore.setTheme(settingsForm.value.theme)
    settingsStore.setPrimaryTarget(settingsForm.value.primaryTarget)
    settingsStore.setSecondaryTarget(settingsForm.value.secondaryTarget)
    settingsStore.setLocale(settingsForm.value.locale)
    locale.value = settingsForm.value.locale
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

onMounted(async () => {
  // 不要在 onMounted 时加载，等待窗口显示或聚焦时再加载
})

// 监听窗口显示和聚焦事件
onMounted(() => {
  let hasInitialized = false

  // 监听窗口显示事件
  window.addEventListener('DOMContentLoaded', () => {
    if (!hasInitialized) {
      setTimeout(() => initSettings(), 100)
      hasInitialized = true
    }
  })

  // 监听窗口获得焦点
  window.addEventListener('focus', () => {
    if (!hasInitialized) {
      initSettings()
      hasInitialized = true
    }
  })

  // 监听窗口可见性变化
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden && !hasInitialized) {
      initSettings()
      hasInitialized = true
    }
  })
})

watch(
  () => settingsForm.value.theme,
  (theme) => applyThemeClass(theme),
)

// 切换服务商时清除测试结果
watch(activeProvider, () => {
  testResult.value = null
  modelSpeedResults.value = []
  showBenchmarkDialog.value = false
})

watch(activeSpeechProvider, () => {
  speechTestResult.value = null
})

const resetDefaults = async () => {
  settingsForm.value = {
    theme: 'dark',
    primaryTarget: 'zh-CN',
    secondaryTarget: 'en',
    locale: 'zh-CN',
  }
  applyThemeClass(settingsForm.value.theme)
  await saveSettings()
  showToast(t('settings.basic.defaultsRestored'), 'info')
}



// 语言切换选项
const localeOptions = computed(() =>
  supportedLocales.map(l => ({ label: l.name, value: l.code }))
)

const selectedLocale = computed({
  get: () => locale.value as SupportedLocale,
  set: (newLocale: string) => changeLocale(newLocale),
})

// 切换界面语言
const changeLocale = (newLocale: string) => {
  locale.value = newLocale as SupportedLocale
  settingsForm.value.locale = newLocale as SupportedLocale
  saveLocale(newLocale as SupportedLocale)
  saveSettings()
  showToast(t('settings.interfaceLanguage.saved'), 'info')
}

const initSettings = async () => {
  if (!isTauriEnv()) return

  isLoading.value = true
  try {
    await Promise.all([
      loadSettings(),
      loadProviderConfigs(),
      loadSpeechProviderConfigs(),
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
  await initSettings()
}
</script>
