import { createApp } from "vue";
import App from "./App.vue";
import { createPinia } from "pinia";
import { createRouter, createWebHashHistory } from "vue-router";

// Naive UI
import {
  create,
  NButton,
  NInput,
  NConfigProvider,
  NMessageProvider,
  NLayout,
  NLayoutSider,
  NLayoutHeader,
  NLayoutContent,
  NMenu,
  NText,
  NIcon,
  NGrid,
  NGi,
  NCard,
  NSpace,
  NForm,
  NFormItem,
  NRadioGroup,
  NRadio,
  NSelect,
  NSpin,
} from "naive-ui";

const pinia = createPinia();

const naive = create({
  components: [
    NButton,
    NInput,
    NConfigProvider,
    NMessageProvider,
    NLayout,
    NLayoutSider,
    NLayoutHeader,
    NLayoutContent,
    NMenu,
    NText,
    NIcon,
    NGrid,
    NGi,
    NCard,
    NSpace,
    NForm,
    NFormItem,
    NRadioGroup,
    NRadio,
    NSelect,
    NSpin,
  ],
});

const routes = [
  {
    path: "/",
    component: () => import("./components/layout/MainLayout.vue"),
    children: [
      { path: "", name: "Translator", component: () => import("./components/features/Translator.vue") },
      { path: "settings", name: "Settings", component: () => import("./components/features/Settings.vue") },
    ],
  },
  {
    path: "/floating",
    name: "Floating",
    component: () => import("./components/layout/FloatingLayout.vue"),
    children: [{ path: "", name: "FloatingTranslator", component: () => import("./components/features/FloatingTranslator.vue") }],
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

createApp(App).use(naive).use(pinia).use(router).mount("#app");