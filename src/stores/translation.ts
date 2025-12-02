// src/stores/translation.ts
import { defineStore } from 'pinia';

export const useTranslationStore = defineStore('translation', {
  state: () => ({
    sourceText: '',
    translatedText: '',
    sourceLang: 'auto',
    targetLang: 'zh-CN',
    isLoading: false,
    history: [] as any[],
  }),
  actions: {
    setSourceText(text: string) {
      this.sourceText = text;
    },
    setTranslatedText(text: string) {
      this.translatedText = text;
    },
    setSourceLang(lang: string) {
      this.sourceLang = lang;
    },
    setTargetLang(lang: string) {
      this.targetLang = lang;
    },
    setLoading(loading: boolean) {
      this.isLoading = loading;
    },
    addHistory(item: any) {
      this.history.unshift(item);
    }
  },
});