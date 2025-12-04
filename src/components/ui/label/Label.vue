<script setup lang="ts">
import { computed } from 'vue'
import { Primitive, type PrimitiveProps } from 'radix-vue'
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = withDefaults(
  defineProps<PrimitiveProps & { class?: HTMLAttributes['class'] }>(),
  {
    as: 'label',
  },
)

const delegatedProps = computed(() => {
  const { as, asChild, class: _class, ...delegated } = props
  return delegated
})
</script>

<template>
  <Primitive
    :as="as"
    :as-child="asChild"
    v-bind="delegatedProps"
    :class="cn('text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70', props.class)"
  >
    <slot />
  </Primitive>
</template>
