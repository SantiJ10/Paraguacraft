<script setup lang="ts">
import { onMounted } from "vue";
import type { RouteLocationNormalizedLoaded } from "vue-router";
import Sidebar from "@/components/layout/Sidebar.vue";
import TopBar from "@/components/layout/TopBar.vue";
import StatusBar from "@/components/layout/StatusBar.vue";
import AiPanel from "@/components/layout/AiPanel.vue";
import MusicPlayer from "@/components/layout/MusicPlayer.vue";
import MusicOverlay from "@/components/layout/MusicOverlay.vue";
import UpdateBanner from "@/components/layout/UpdateBanner.vue";
import { useAppStore } from "@/stores/app";
import { useAccountsStore } from "@/stores/accounts";
import { useInstancesStore } from "@/stores/instances";
import { useSettingsStore } from "@/stores/settings";
import { useDownloadsStore } from "@/stores/downloads";
import { useMusicStore } from "@/stores/music";
import { useSkinsStore } from "@/stores/skins";
import { useServersStore } from "@/stores/servers";

/**
 * Coincide con `defineOptions({ name })`.
 * server-detail se cachea para no perder consola al ir a Inicio y volver.
 */
const KEEP_ALIVE_VIEWS = [
  "home",
  "instances",
  "store",
  "skins",
  "versions",
  "servers",
  "settings",
  "server-detail",
];

const app = useAppStore();
const accounts = useAccountsStore();
const instances = useInstancesStore();
const settings = useSettingsStore();
const downloads = useDownloadsStore();
const music = useMusicStore();
const skins = useSkinsStore();
const servers = useServersStore();

function keepAliveKey(route: RouteLocationNormalizedLoaded) {
  const name = typeof route.name === "string" ? route.name : "";
  if (name === "server-detail") return "server-detail";
  if (KEEP_ALIVE_VIEWS.includes(name)) return name;
  if (name === "instance-detail") return route.fullPath;
  return route.fullPath;
}

function scheduleIdle(fn: () => void, fallbackMs = 400) {
  const ric = window.requestIdleCallback;
  if (typeof ric === "function") {
    ric(() => fn(), { timeout: fallbackMs + 800 });
  } else {
    window.setTimeout(fn, fallbackMs);
  }
}

onMounted(() => {
  app.initGameEvents();
  downloads.initEvents();

  // 1) Crítico para pintar Inicio: cuentas + instancias (settings ya en App.vue).
  void Promise.all([
    settings.loaded ? Promise.resolve() : settings.load(),
    accounts.load(),
    instances.load(),
  ]).then(() => {
    // 2) Tras primer paint: lista de servers (sidebar “continuar”) sin bloquear.
    scheduleIdle(() => {
      void servers.load(false);
    }, 120);
    scheduleIdle(() => {
      void music.init();
    }, 500);
    scheduleIdle(() => {
      void app.checkUpdate();
    }, 2000);
  });

  // 3) Pesados (PowerShell / skins / scan launchers ajenos) bien demorados.
  window.setTimeout(() => {
    void app.loadHardware();
  }, 600);
  window.setTimeout(() => {
    void skins.refresh();
  }, 900);
  window.setTimeout(() => {
    void instances.scan();
  }, 5000);
});
</script>

<template>
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <div class="relative flex flex-1 flex-col overflow-hidden">
      <UpdateBanner />
      <TopBar />
      <main class="flex-1 overflow-y-auto">
        <RouterView v-slot="{ Component, route }">
          <KeepAlive :include="KEEP_ALIVE_VIEWS" :max="10">
            <component :is="Component" :key="keepAliveKey(route)" />
          </KeepAlive>
        </RouterView>
      </main>
      <StatusBar />
      <AiPanel />
      <MusicPlayer />
      <MusicOverlay />
    </div>
  </div>
</template>
