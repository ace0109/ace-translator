<template>
  <Select
    v-model="selectedLanguage"
    :placeholder="placeholder"
    :options="languageOptions"
    class="h-10"
  />
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Select } from '@/components/ui/select'
import { languageOptions } from '@/constants/languages'

interface Props {
  modelValue: string
  placeholder?: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const selectedLanguage = ref(props.modelValue)

watch(
  () => props.modelValue,
  (val) => {
    selectedLanguage.value = val
  },
)

watch(
  selectedLanguage,
  (val) => {
    emit('update:modelValue', val)
  },
)
</script>
