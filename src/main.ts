import { createApp } from "vue";
import "./assets/index.css";
import App from "./App.vue";
import { createPinia } from "pinia";
import { createRouter, createWebHashHistory } from "vue-router";
import i18n from "./locales";

const pinia = createPinia();

const routes = [
  {
    path: "/main",
    name: "Main",
    component: () => import("./components/features/MainTranslator.vue"),
  },
  {
    path: "/settings",
    name: "Settings",
    component: () => import("./components/features/Settings.vue"),
  },
  {
    path: "/history",
    name: "History",
    component: () => import("./components/features/HistoryAndLogs.vue"),
  },
  {
    path: "/logs",
    name: "Logs",
    component: () => import("./components/features/LogsWindow.vue"),
  },
  {
    path: "/about",
    name: "About",
    component: () => import("./components/features/AboutWindow.vue"),
  },
  {
    path: "/permissions",
    name: "Permissions",
    component: () => import("./components/features/PermissionGuide.vue"),
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

createApp(App).use(pinia).use(router).use(i18n).mount("#app");
