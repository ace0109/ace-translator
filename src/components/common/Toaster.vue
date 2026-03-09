<template>
  <div class="pointer-events-none fixed left-1/2 top-6 z-50 flex w-full -translate-x-1/2 flex-col items-center gap-2 px-4">
    <TransitionGroup name="fade">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="pointer-events-auto w-auto max-w-sm break-words rounded-md border px-4 py-3 shadow-lg backdrop-blur"
        :class="toast.type === 'error' ? 'bg-destructive text-destructive-foreground' : 'bg-white text-slate-900 dark:bg-slate-900 dark:text-white'"
      >
        <p class="text-sm font-medium">
          {{ toast.message }}
        </p>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { onToast, type ToastPayload } from '@/lib/toast'

const toasts = ref<ToastPayload[]>([])

let unsubscribe: (() => void) | null = null

const removeToast = (id: number) => {
  toasts.value = toasts.value.filter((t) => t.id !== id)
}

onMounted(() => {
  unsubscribe = onToast((payload) => {
    toasts.value.push(payload)
    setTimeout(() => removeToast(payload.id), 2400)
  })
})

onUnmounted(() => {
  unsubscribe?.()
})
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: all 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
