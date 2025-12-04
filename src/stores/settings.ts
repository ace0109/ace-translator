// src/stores/settings.ts
import { defineStore } from 'pinia';

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    apiKey: '',
    theme: 'light',
    commonTargetLanguages: [] as string[],
  }),
  actions: {
    setApiKey(key: string) {
      this.apiKey = key;
    },
    setTheme(theme: string) {
      this.theme = theme;
    },
    setCommonTargetLanguages(langs: string[]) {
      this.commonTargetLanguages = langs;
    },
  },
});
