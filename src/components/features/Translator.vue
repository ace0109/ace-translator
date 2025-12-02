<template>
  <n-layout has-sider style="height: 100%;">
    <n-layout-content content-style="padding: 24px;">
      <n-grid x-gap="12" :cols="2">
        <n-gi>
          <n-space vertical>
            <n-card :bordered="false" title="源语言">
              <template #header-extra>
                <LanguageSelector v-model="sourceLang" placeholder="选择源语言" />
              </template>
              <n-input
                v-model:value="sourceText"
                type="textarea"
                placeholder="输入或粘贴要翻译的文本"
                :autosize="{ minRows: 10 }"
              />
            </n-card>
          </n-space>
        </n-gi>
        <n-gi>
          <n-space vertical>
            <n-card :bordered="false" title="目标语言">
              <template #header-extra>
                <LanguageSelector v-model="targetLang" placeholder="选择目标语言" />
              </template>
              <n-input
                v-model:value="translatedText"
                type="textarea"
                placeholder="翻译结果将显示在这里"
                :autosize="{ minRows: 10 }"
                readonly
              />
            </n-card>
          </n-space>
        </n-gi>
      </n-grid>
      <n-space justify="center" style="margin-top: 24px;">
        <n-button type="primary" :loading="isLoading" @click="handleTranslate">
          <template #icon>
            <n-icon><Language /></n-icon>
          </template>
          翻译
        </n-button>
        <n-button @click="swapLanguages">
          <template #icon>
            <n-icon><SwapHorizontal /></n-icon>
          </template>
          交换语言
        </n-button>
        <n-button @click="clearText">
          <template #icon>
            <n-icon><Trash /></n-icon>
          </template>
          清空
        </n-button>
      </n-space>
    </n-layout-content>
  </n-layout>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { NLayout, NLayoutContent, NGrid, NGi, NCard, NInput, NSpace, NButton, NIcon, useMessage } from 'naive-ui';
import { Language, SwapHorizontal, Trash } from '@vicons/ionicons5';
import LanguageSelector from '../common/LanguageSelector.vue';
import { invoke } from '@tauri-apps/api/core';
import { useTranslationStore } from '../../stores/translation';

const message = useMessage();
const translationStore = useTranslationStore();

const sourceText = ref(translationStore.sourceText);
const translatedText = ref(translationStore.translatedText);
const sourceLang = ref(translationStore.sourceLang);
const targetLang = ref(translationStore.targetLang);
const isLoading = ref(translationStore.isLoading);

// 同步 store
watch(sourceText, (val) => { translationStore.setSourceText(val); });
watch(translatedText, (val) => { translationStore.setTranslatedText(val); });
watch(sourceLang, (val) => { translationStore.setSourceLang(val); });
watch(targetLang, (val) => { translationStore.setTargetLang(val); });
watch(isLoading, (val) => { translationStore.setLoading(val); });

async function handleTranslate() {
  if (!sourceText.value.trim()) {
    message.warning('请输入要翻译的文本');
    return;
  }
  isLoading.value = true;
  try {
    const result: string = await invoke('translate_text', {
      text: sourceText.value,
      sourceLang: sourceLang.value,
      targetLang: targetLang.value,
    });
    translatedText.value = result;
    message.success('翻译成功');
  } catch (error: any) {
    message.error(`翻译失败: ${error}`);
    translatedText.value = `翻译失败: ${error}`;
  } finally {
    isLoading.value = false;
  }
}

function swapLanguages() {
  const tempLang = sourceLang.value;
  sourceLang.value = targetLang.value;
  targetLang.value = tempLang;

  const tempText = sourceText.value;
  sourceText.value = translatedText.value;
  translatedText.value = tempText;
}

function clearText() {
  sourceText.value = '';
  translatedText.value = '';
}
</script>

<style scoped>
/* 可以在这里添加 Translator.vue 的局部样式 */
</style>