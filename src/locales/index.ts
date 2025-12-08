import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN.json'
import en from './en.json'

export type MessageSchema = typeof zhCN

// 支持的语言列表
export const supportedLocales = [
  { code: 'zh-CN', name: '简体中文' },
  { code: 'en', name: 'English' },
] as const

export type SupportedLocale = typeof supportedLocales[number]['code']

// 获取保存的语言设置，默认使用中文
function getSavedLocale(): SupportedLocale {
  try {
    const saved = localStorage.getItem('app-locale')
    if (saved && supportedLocales.some(l => l.code === saved)) {
      return saved as SupportedLocale
    }
  } catch {
    // localStorage 可能不可用
  }
  return 'zh-CN'
}

// 保存语言设置
export function saveLocale(locale: SupportedLocale): void {
  try {
    localStorage.setItem('app-locale', locale)
  } catch {
    // localStorage 可能不可用
  }
}

const i18n = createI18n<[MessageSchema], SupportedLocale>({
  legacy: false, // 使用 Composition API 模式
  locale: getSavedLocale(),
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en': en,
  },
})

export default i18n
