// src/stores/translation.ts
import { defineStore } from 'pinia';

export const useTranslationStore = defineStore('translation', {
  state: () => ({
    sourceText: '',
    targetLang: 'zh-CN',
    isLoading: false,
    detectedLang: '',
    translations: {} as Record<string, string>,
  }),
  actions: {
    setSourceText(text: string) {
      this.sourceText = text;
    },
    setTargetLang(lang: string) {
      this.targetLang = lang;
    },
    setLoading(loading: boolean) {
      this.isLoading = loading;
    },
    setDetectedLang(lang: string) {
      this.detectedLang = lang;
    },
    setTranslations(map: Record<string, string>) {
      this.translations = map;
    },
    mergeTranslations(map: Record<string, string>) {
      this.translations = { ...this.translations, ...map };
    },
    clearTranslations() {
      this.translations = {};
      this.detectedLang = '';
    },
  },
});
