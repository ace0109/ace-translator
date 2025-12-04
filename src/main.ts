import { createApp } from "vue";
import "./assets/index.css";
import App from "./App.vue";
import { createPinia } from "pinia";
import { createRouter, createWebHashHistory } from "vue-router";

const pinia = createPinia();

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

createApp(App).use(pinia).use(router).mount("#app");
