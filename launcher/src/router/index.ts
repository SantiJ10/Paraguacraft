import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";
import { api, isTauri } from "@/lib/ipc";
import { useAppStore } from "@/stores/app";
// Eager: 1er paint de Inicio sin esperar chunk async.
import MainLayout from "@/layouts/MainLayout.vue";
import HomeView from "@/views/HomeView.vue";

const routes: RouteRecordRaw[] = [
  {
    path: "/wizard",
    name: "wizard",
    component: () => import("@/views/WelcomeWizard.vue"),
    meta: { fullscreen: true },
  },
  {
    path: "/",
    component: MainLayout,
    children: [
      { path: "", name: "home", component: HomeView },
      { path: "instances", name: "instances", component: () => import("@/views/InstancesView.vue") },
      {
        path: "instances/:id",
        name: "instance-detail",
        component: () => import("@/views/InstanceDetailView.vue"),
      },
      { path: "store", name: "store", component: () => import("@/views/StoreView.vue") },
      { path: "skins", name: "skins", component: () => import("@/views/SkinsView.vue") },
      { path: "versions", name: "versions", component: () => import("@/views/VersionsView.vue") },
      { path: "servers", name: "servers", component: () => import("@/views/ServersView.vue") },
      {
        path: "servers/:id",
        name: "server-detail",
        component: () => import("@/views/ServerDetailView.vue"),
      },
      { path: "settings", name: "settings", component: () => import("@/views/SettingsView.vue") },
    ],
  },
  { path: "/:pathMatch(.*)*", redirect: "/" },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

// Guard de onboarding: si el wizard no se completo, llevamos al usuario alli.
router.beforeEach((to) => {
  const done = localStorage.getItem("pc.onboarding.done") === "1";
  if (!done && to.name !== "wizard") {
    return { name: "wizard" };
  }
  if (done && to.name === "wizard") {
    return { name: "home" };
  }
  return true;
});

let discordRpcTimer: ReturnType<typeof setTimeout> | null = null;
router.afterEach((to) => {
  if (!isTauri()) return;
  // No bloquear nav con Discord; debounce corto.
  if (discordRpcTimer) clearTimeout(discordRpcTimer);
  discordRpcTimer = setTimeout(() => {
    const app = useAppStore();
    if (app.launchPhase === "running") return;
    const screen = to.name === "settings" ? "settings" : "idle";
    void api.setDiscordRpcScreen(screen);
  }, 250);
});

export default router;
