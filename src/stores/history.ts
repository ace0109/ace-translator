// src/stores/history.ts
import { defineStore } from 'pinia';

export const useHistoryStore = defineStore('history', {
  state: () => ({
    translationHistory: [] as any[],
  }),
  actions: {
    addTranslation(item: any) {
      this.translationHistory.unshift(item);
    },
    clearHistory() {
      this.translationHistory = [];
    }
  },
});