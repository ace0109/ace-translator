<template>
  <div class="floating-layout">
    <router-view></router-view>
  </div>
</template>

<script setup lang="ts">
// No specific logic needed here
</script>

<style scoped>
.floating-layout {
  width: 100%;
  height: 100%;
  background-color: blue; /* Debug: Make it obviously visible */
  border-radius: 8px;
  border: 1px solid #333;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  box-sizing: border-box;
}
</style>