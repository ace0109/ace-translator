import { defineConfig } from 'vitepress'

const repository = process.env.GITHUB_REPOSITORY ?? 'ace0109/ace-translator'
const repoName = repository.split('/')[1] ?? 'ace-translator'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'Ace Translator',
  description: '您的智能 AI 翻译助手',
  base: `/${repoName}/`,
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: '/logo.png',
  }
})
