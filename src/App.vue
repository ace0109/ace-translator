<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { NConfigProvider, NMessageProvider, darkTheme, lightTheme } from 'naive-ui';
import { zhCN, dateZhCN } from 'naive-ui';
import { useSettingsStore } from './stores/settings';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// 可以根据应用主题或其他设置动态改变 themeOverrides
const themeOverrides = {
  common: {
    primaryColor: '#646CFF',
    primaryColorHover: '#535BF2',
  },
  // ... 其他组件的全局主题配置
};

const settingsStore = useSettingsStore();
const currentTheme = computed(() => (settingsStore.theme === 'dark' ? darkTheme : lightTheme));

onMounted(async () => {
  // 1. Load initial settings
  try {
    const settings: any = await invoke('get_settings');
    if (settings) {
      settingsStore.setApiKey(settings.api_key);
      settingsStore.setTheme(settings.theme);
    }
  } catch (error) {
    console.error('Failed to load settings:', error);
  }

  // 2. Listen for settings changes from other windows
  await listen<any>('settings-changed', (event) => {
    const s = event.payload;
    if (s) {
      settingsStore.setApiKey(s.api_key);
      settingsStore.setTheme(s.theme);
    }
  });
});

</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme-overrides="themeOverrides" :theme="currentTheme">
    <n-message-provider>
      <router-view></router-view>
    </n-message-provider>
  </n-config-provider>
</template>

<style>

/* Global styles */

html, body, #app {

  margin: 0;

  padding: 0;

  width: 100%;

  height: 100%;

  overflow: hidden; /* Prevent scrollbars */

  background: transparent; /* Important for window transparency */

}

</style>