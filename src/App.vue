<script setup lang="ts">
import { onMounted } from 'vue';
import { useSettingsStore } from './stores/settings';
import { useTranslationStore } from './stores/translation';
import { isTauriEnv } from './utils/env';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import Toaster from './components/common/Toaster.vue';
import { defaultCommonTargets } from './constants/languages';

const settingsStore = useSettingsStore();
const translationStore = useTranslationStore();

// Apply default theme immediately to avoid light flash before settings load
if ((settingsStore.theme || 'dark') === 'dark') {
  document.documentElement.classList.add('dark');
} else {
  document.documentElement.classList.remove('dark');
}

onMounted(async () => {
  if (!isTauriEnv()) return;
  // 1. Load initial settings
  try {
    const settings: any = await invoke('get_settings');
    if (settings) {
      settingsStore.setApiKey(settings.api_key);
      settingsStore.setTheme(settings.theme || 'dark');
      settingsStore.setCommonTargetLanguages(settings.common_target_languages || defaultCommonTargets);
      translationStore.setTargetLang(settings.target_language || 'zh-CN');
      
      // Apply dark mode class to html element
      if ((settings.theme || 'dark') === 'dark') {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    }
  } catch (error) {
    console.error('Failed to load settings:', error);
  }

  // 2. Listen for settings changes from other windows
  await listen<any>('settings-changed', (event) => {
    const s = event.payload;
    if (s) {
      settingsStore.setApiKey(s.api_key);
      settingsStore.setTheme(s.theme || 'dark');
      settingsStore.setCommonTargetLanguages(s.common_target_languages || defaultCommonTargets);
      translationStore.setTargetLang(s.target_language || 'zh-CN');
      
      // Sync dark mode class
      if ((s.theme || 'dark') === 'dark') {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    }
  });
});
</script>

<template>
  <div class="h-screen w-screen bg-background text-foreground antialiased overflow-auto">
    <router-view></router-view>
    <Toaster />
  </div>
</template>
