<script setup lang="ts">
import { NConfigProvider, NMessageProvider } from 'naive-ui';
import { zhCN, dateZhCN } from 'naive-ui';

// 可以根据应用主题或其他设置动态改变 themeOverrides
const themeOverrides = {
  common: {
    primaryColor: '#646CFF',
    primaryColorHover: '#535BF2',
  },
  // ... 其他组件的全局主题配置
};

</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme-overrides="themeOverrides">
    <n-message-provider>
      <router-view></router-view>
    </n-message-provider>
  </n-config-provider>
</template>

<style>

/* 全局样式 */

html, body, #app {

  margin: 0;

  padding: 0;

  width: 100%;

  height: 100%;

  overflow: hidden; /* 防止滚动条 */

  background: transparent; /* Important for window transparency */

}

</style>
