import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  base: '/ace-translator/',
  title: "Ace Translator",
  description: "您的智能 AI 翻译助手",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: '/logo.png',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/ace0109/ace-translator' },
    ],
  }
})
