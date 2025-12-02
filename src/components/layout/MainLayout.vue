<template>
  <n-layout has-sider style="height: 100vh;">
    <n-layout-sider
      bordered
      collapse-mode="width"
      :collapsed-width="64"
      :width="240"
      :native-scrollbar="false"
      show-trigger="arrow-circle"
    >
      <div style="height: 64px; display: flex; align-items: center; justify-content: center;">
        <!-- Logo or App Name -->
        <n-text strong depth="1" style="font-size: 24px;">Ace</n-text>
      </div>
      <n-menu
        :collapsed-width="64"
        :collapsed-icon-size="22"
        :options="menuOptions"
        :render-label="renderMenuLabel"
        :render-icon="renderMenuIcon"
        :expand-icon="expandIcon"
        :value="activeMenu"
        @update:value="handleMenuUpdate"
      />
    </n-layout-sider>
    <n-layout>
      <n-layout-header bordered style="height: 64px; display: flex; align-items: center; padding: 0 24px;">
        <!-- Header content, e.g., current route title -->
        <n-text strong :depth="1" style="font-size: 18px;">
          {{ currentRouteTitle }}
        </n-text>
      </n-layout-header>
      <n-layout-content
        :native-scrollbar="false"
        style="padding: 24px;"
      >
        <router-view></router-view>
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import { h, computed } from 'vue';
import { RouterLink, useRoute } from 'vue-router';
import { NLayout, NLayoutSider, NLayoutHeader, NLayoutContent, NMenu, NText, NIcon } from 'naive-ui';
import { Language, Settings } from '@vicons/ionicons5'; // 引入图标

const route = useRoute();

const activeMenu = computed(() => {
  if (route.name === 'Translator') return 'translator';
  if (route.name === 'Settings') return 'settings';
  return '';
});

const currentRouteTitle = computed(() => {
  switch (route.name) {
    case 'Translator':
      return '主翻译';
    case 'Settings':
      return '设置';
    default:
      return 'Ace Translator';
  }
});

const menuOptions = [
  {
    label: () => h(
      RouterLink,
      {
        to: {
          name: 'Translator'
        }
      },
      { default: () => '主翻译' }
    ),
    key: 'translator',
    icon: () => h(NIcon, null, { default: () => h(Language) })
  },
  {
    label: () => h(
      RouterLink,
      {
        to: {
          name: 'Settings'
        }
      },
      { default: () => '设置' }
    ),
    key: 'settings',
    icon: () => h(NIcon, null, { default: () => h(Settings) })
  }
];

function renderMenuLabel(option: any) {
  return option.label();
}

function renderMenuIcon(option: any) {
  return option.icon();
}

const expandIcon = () => h(NIcon, null, { default: () => h('div', '>') }); // 示例展开图标

function handleMenuUpdate(key: string) {
  // 菜单点击时的逻辑，因为使用了 RouterLink，这里可能不需要额外操作
  // 如果需要更复杂的逻辑，可以在这里处理
  console.log('Menu updated:', key);
}
</script>

<style scoped>
/* 可以在这里添加 MainLayout 的局部样式 */
</style>