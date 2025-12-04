// src/stores/settings.ts
import { defineStore } from 'pinia';

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    apiKey: '',
    theme: 'light',
  }),
  actions: {
    setApiKey(key: string) {
      this.apiKey = key;
    },
    setTheme(theme: string) {
      this.theme = theme;
    },
  },
});