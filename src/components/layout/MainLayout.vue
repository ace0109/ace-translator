<template>
  <n-layout style="height: 100vh;">
    <n-layout-content
      :native-scrollbar="false"
      content-style="height: 100%; display: flex; flex-direction: column;"
    >
      <router-view></router-view>
    </n-layout-content>
  </n-layout>
</template>

<script setup lang="ts">
import { NLayout, NLayoutContent } from 'naive-ui';
</script>

<style scoped>
/* 可以在这里添加 MainLayout 的局部样式 */
</style>