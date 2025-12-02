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

// Import layout components
import MainLayout from "./components/layout/MainLayout.vue";
import FloatingLayout from "./components/layout/FloatingLayout.vue";

// Import feature components
import Translator from "./components/features/Translator.vue";
import FloatingTranslator from "./components/features/FloatingTranslator.vue";
import Settings from "./components/features/Settings.vue";

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
    component: MainLayout,
    children: [
      { path: "", name: "Translator", component: Translator },
      { path: "settings", name: "Settings", component: Settings },
    ],
  },
  {
    path: "/floating",
    name: "Floating",
    component: FloatingLayout,
    children: [{ path: "", name: "FloatingTranslator", component: FloatingTranslator }],
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

createApp(App).use(naive).use(pinia).use(router).mount("#app");
