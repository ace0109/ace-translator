<script setup lang="ts">
import { useVModel } from '@vueuse/core'
import { useAttrs, type HTMLAttributes } from 'vue'
import { ChevronDown } from 'lucide-vue-next'
import { cn } from '@/lib/utils'

defineOptions({
  inheritAttrs: false,
})

interface Option {
  label: string
  value: string
}

const props = withDefaults(
  defineProps<{
    modelValue?: string
    options: Option[]
    placeholder?: string
    class?: HTMLAttributes['class']
    disabled?: boolean
  }>(),
  {
    options: () => [],
    placeholder: '请选择',
    disabled: false,
  },
)

const emits = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
})

const attrs = useAttrs()
</script>

<template>
  <div class="relative w-full">
    <select
      v-model="modelValue"
      :disabled="disabled"
      v-bind="attrs"
      :class="cn('h-10 w-full appearance-none rounded-md border border-input bg-background px-3 py-2 text-left text-sm text-foreground shadow-sm ring-offset-background placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60', props.class)"
    >
      <option value="" disabled>
        {{ placeholder }}
      </option>
      <option
        v-for="option in options"
        :key="option.value"
        :value="option.value"
      >
        {{ option.label }}
      </option>
    </select>
    <ChevronDown
      class="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
    />
  </div>
</template>
