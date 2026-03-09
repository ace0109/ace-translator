// src/stores/settings.ts
import { defineStore } from 'pinia';

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    theme: 'dark',
    /** 第一语言（默认翻译目标，如：中文） */
    primaryTarget: 'zh-CN',
    /** 第二语言（当源语言是第一语言时使用，如：英文） */
    secondaryTarget: 'en',
    /** 界面语言 */
    locale: 'zh-CN',
  }),
  actions: {
    setTheme(theme: string) {
      this.theme = theme;
    },
    setPrimaryTarget(lang: string) {
      this.primaryTarget = lang;
    },
    setSecondaryTarget(lang: string) {
      this.secondaryTarget = lang;
    },
    setLocale(locale: string) {
      this.locale = locale;
    },
  },
});
