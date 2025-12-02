<template>
  <n-card title="应用设置" :bordered="false">
    <n-form
      ref="formRef"
      :model="settingsForm"
      :rules="rules"
      label-placement="left"
      label-width="auto"
      require-mark-placement="right-hanging"
    >
      <n-form-item label="智谱 AI API Key" path="apiKey">
        <n-input
          v-model:value="settingsForm.apiKey"
          type="password"
          show-password-on="click"
          placeholder="请输入智谱 AI API Key"
        />
      </n-form-item>
      <n-form-item label="默认目标语言" path="targetLanguage">
        <LanguageSelector v-model="settingsForm.targetLanguage" placeholder="选择默认目标语言" />
      </n-form-item>
      <n-form-item label="全局快捷键" path="globalShortcut">
        <n-input
          v-model:value="settingsForm.globalShortcut"
          placeholder="例如：Alt+T"
          @keydown="handleShortcutInput"
          readonly
        />
        <n-button style="margin-left: 8px;" @click="resetGlobalShortcut">重置</n-button>
      </n-form-item>
      <n-form-item label="主题" path="theme">
        <n-radio-group v-model:value="settingsForm.theme">
          <n-radio value="light">浅色</n-radio>
          <n-radio value="dark">深色</n-radio>
        </n-radio-group>
      </n-form-item>
      <n-form-item>
        <n-button type="primary" @click="saveSettings">保存设置</n-button>
      </n-form-item>
    </n-form>
  </n-card>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NRadioGroup,
  NRadio,
  NButton,
  useMessage,
  FormInst,
  FormRules,
} from 'naive-ui';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../stores/settings';
import LanguageSelector from '../common/LanguageSelector.vue';
import { DEFAULT_SHORTCUT } from '../../utils/constants';

const message = useMessage();
const settingsStore = useSettingsStore();
const formRef = ref<FormInst | null>(null);

interface SettingsForm {
  apiKey: string;
  targetLanguage: string;
  globalShortcut: string;
  theme: string;
}

const settingsForm = ref<SettingsForm>({
  apiKey: '',
  targetLanguage: 'zh-CN', // 默认目标语言
  globalShortcut: DEFAULT_SHORTCUT,
  theme: 'light',
});

const rules: FormRules = {
  apiKey: {
    required: true,
    message: '请输入智谱 AI API Key',
    trigger: ['input', 'blur'],
  },
  globalShortcut: {
    required: true,
    message: '请输入全局快捷键',
    trigger: ['input', 'blur'],
  },
};

onMounted(async () => {
  try {
    const loadedSettings: any = await invoke('get_settings');
    settingsForm.value.apiKey = loadedSettings.api_key;
    settingsForm.value.targetLanguage = loadedSettings.target_language || 'zh-CN'; // 从后端获取的 target_language
    settingsForm.value.globalShortcut = loadedSettings.global_shortcut;
    settingsForm.value.theme = loadedSettings.theme;
  } catch (error) {
    message.error(`加载设置失败: ${error}`);
  }
});

async function saveSettings() {
  try {
    await formRef.value?.validate();
    await invoke('save_settings', {
      settings: {
        api_key: settingsForm.value.apiKey,
        target_language: settingsForm.value.targetLanguage,
        global_shortcut: settingsForm.value.globalShortcut,
        theme: settingsForm.value.theme,
      },
    });
    settingsStore.setApiKey(settingsForm.value.apiKey);
    settingsStore.setGlobalShortcut(settingsForm.value.globalShortcut);
    settingsStore.setTheme(settingsForm.value.theme);
    message.success('设置保存成功');
  } catch (error: any) {
    message.error(`保存设置失败: ${error.message || error}`);
  }
}

function handleShortcutInput(event: KeyboardEvent) {
  event.preventDefault(); // 阻止默认输入
  const keys: string[] = [];
  if (event.ctrlKey) keys.push('Ctrl');
  if (event.shiftKey) keys.push('Shift');
  if (event.altKey) keys.push('Alt');
  if (event.metaKey) keys.push('Meta'); // Command key on macOS

  // 避免修饰键单独作为快捷键
  if (event.key && event.key !== 'Control' && event.key !== 'Shift' && event.key !== 'Alt' && event.key !== 'Meta') {
    keys.push(event.key.toUpperCase());
  }
  settingsForm.value.globalShortcut = keys.join('+');
}

function resetGlobalShortcut() {
  settingsForm.value.globalShortcut = DEFAULT_SHORTCUT;
}
</script>