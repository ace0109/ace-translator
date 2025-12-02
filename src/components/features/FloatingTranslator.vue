<template>
  <div class="floating-translator-container">
    <n-card :bordered="false" size="small" class="floating-card">
      <div class="content">
        <LoadingSpinner v-if="isLoading" :show="isLoading" />
        <n-text v-else class="text">{{ translatedText || '等待翻译…' }}</n-text>
      </div>
      <div class="footer" v-if="sourcePreview">
        <n-text depth="3" class="preview-label">原文</n-text>
        <n-text depth="2" class="preview">{{ sourcePreview }}</n-text>
      </div>
      <n-button
        v-if="!isLoading && translatedText"
        class="close-button"
        size="tiny"
        circle
        @click="closeFloatingWindow"
      >
        <n-icon><CloseCircle /></n-icon>
      </n-button>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { NCard, NText, NButton, NIcon, useMessage } from "naive-ui";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import LoadingSpinner from "../common/LoadingSpinner.vue";
import { CloseCircle } from "@vicons/ionicons5";

const translatedText = ref("");
const sourcePreview = ref("");
const isLoading = ref(false);
const message = useMessage();

let unlisten: (() => void) | undefined;

onMounted(async () => {
  console.log("[floating] mounted, waiting for events");
  unlisten = await listen<string>("floating-show", async (event) => {
    console.log("[floating] event received", event.payload);
    sourcePreview.value = event.payload?.slice(0, 120) || "";
    translatedText.value = "";
    isLoading.value = true;

    try {
      const settings = await invoke<{ target_language: string }>("get_settings");
      const targetLang = settings?.target_language || "zh-CN";
      console.log("[floating] invoke translate_text", { targetLang, sourceLen: event.payload.length });
      const result: string = await invoke("translate_text", {
        text: event.payload,
        sourceLang: "auto",
        targetLang,
      });
      console.log("[floating] translate_text result len", result.length);
      translatedText.value = result;
    } catch (error: any) {
      const errMsg = error?.message || String(error);
      translatedText.value = `翻译失败: ${errMsg}`;
      message.error(`悬浮窗翻译失败: ${errMsg}`);
    } finally {
      isLoading.value = false;
    }
  });
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});

async function closeFloatingWindow() {
  await invoke("hide_window");
}
</script>

<style scoped>
.floating-translator-container {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
}

.floating-card {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: 10px;
  border-radius: 10px;
  position: relative;
  background: radial-gradient(circle at 20% 20%, rgba(255, 255, 255, 0.08), transparent),
    radial-gradient(circle at 80% 30%, rgba(255, 255, 255, 0.06), transparent),
    #0f172a;
  color: white;
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.45);
}

.content {
  flex-grow: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  text-align: center;
  padding: 0 12px;
  word-break: break-all;
  overflow: hidden;
  font-size: 14px;
}

.text {
  color: #e2e8f0;
}

.footer {
  width: 100%;
  padding-top: 6px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.preview-label {
  font-size: 12px;
  display: block;
}

.preview {
  display: block;
  font-size: 12px;
  margin-top: 2px;
  color: rgba(226, 232, 240, 0.85);
}

.close-button {
  position: absolute;
  top: 6px;
  right: 6px;
  background-color: transparent;
  color: white;
  border: none;
  opacity: 0.7;
}

.close-button:hover {
  opacity: 1;
  background-color: rgba(255, 255, 255, 0.12);
}
</style>
