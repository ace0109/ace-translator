<template>
  <Select v-model="selectedLanguage">
    <SelectTrigger :id="id" class="h-10 w-full">
      <SelectValue :placeholder="placeholder" />
    </SelectTrigger>
    <SelectContent>
      <SelectGroup>
        <SelectItem v-for="option in languageOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </SelectItem>
      </SelectGroup>
    </SelectContent>
  </Select>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { languageOptions } from '@/constants/languages'

interface Props {
  modelValue: string
  placeholder?: string
  id?: string
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
