<template>
  <div class="loading-spinner">
    <n-spin :show="show" size="large" />
  </div>
</template>

<script setup lang="ts">
import { NSpin } from 'naive-ui';

interface Props {
  show: boolean;
}

defineProps<Props>();
</script>

<style scoped>
.loading-spinner {
  display: flex;
  justify-content: center;
  align-items: center;
  /* 确保占据足够的空间以居中显示 */
  width: 100%;
  height: 100%;
}
</style>