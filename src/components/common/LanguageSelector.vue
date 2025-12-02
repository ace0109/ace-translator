<template>
  <n-select
    v-model:value="selectedLanguage"
    :options="languageOptions"
    :placeholder="placeholder"
    filterable
    @update:value="handleUpdateValue"
  />
</template>

<script setup lang="ts">
import { ref, watchEffect } from 'vue';
import { NSelect } from 'naive-ui';

interface Props {
  modelValue: string;
  placeholder?: string;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);

const selectedLanguage = ref(props.modelValue);

watchEffect(() => {
  selectedLanguage.value = props.modelValue;
});

const languageOptions = ref([
  { label: '自动检测', value: 'auto' },
  { label: '简体中文', value: 'zh-CN' },
  { label: '繁体中文', value: 'zh-TW' },
  { label: '英语', value: 'en' },
  { label: '日语', value: 'ja' },
  { label: '韩语', value: 'ko' },
  { label: '法语', value: 'fr' },
  { label: '德语', value: 'de' },
  { label: '西班牙语', value: 'es' },
  { label: '俄语', value: 'ru' },
  { label: '阿拉伯语', value: 'ar' },
  { label: '葡萄牙语', value: 'pt' },
  { label: '意大利语', value: 'it' },
  { label: '荷兰语', value: 'nl' },
  { label: '瑞典语', value: 'sv' },
  { label: '挪威语', value: 'no' },
  { label: '丹麦语', value: 'da' },
  { label: '芬兰语', value: 'fi' },
  { label: '波兰语', value: 'pl' },
  { label: '捷克语', value: 'cs' },
  { label: '匈牙利语', value: 'hu' },
  { label: '罗马尼亚语', value: 'ro' },
]);

const handleUpdateValue = (value: string) => {
  emit('update:modelValue', value);
};
</script>