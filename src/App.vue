<script setup lang="ts">
import { onMounted } from 'vue';
import { useSettingsStore } from './stores/settings';
import { isTauriEnv } from './utils/env';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import i18n from './locales';
import type { Ref } from 'vue';
import Toaster from './components/common/Toaster.vue';

// Cast locale to Ref for type safety in non-legacy mode
const locale = i18n.global.locale as unknown as Ref<string>;

const settingsStore = useSettingsStore();

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
      settingsStore.setTheme(settings.theme || 'dark');
      settingsStore.setPrimaryTarget(settings.primary_target || 'zh-CN');
      settingsStore.setSecondaryTarget(settings.secondary_target || 'en');
      settingsStore.setLocale(settings.locale || 'zh-CN');
      locale.value = settings.locale || 'zh-CN';

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
      settingsStore.setTheme(s.theme || 'dark');
      settingsStore.setPrimaryTarget(s.primary_target || 'zh-CN');
      settingsStore.setSecondaryTarget(s.secondary_target || 'en');
      settingsStore.setLocale(s.locale || 'zh-CN');
      locale.value = s.locale || 'zh-CN';

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
  <div class="h-screen w-screen text-foreground antialiased overflow-y-auto">
    <router-view></router-view>
    <Toaster />
  </div>
</template>
