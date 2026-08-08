<script setup lang="ts">
import { onMounted } from "vue";
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

/**
 * Vistas cacheadas (Fase Optimización UI): coincide con `defineOptions({ name })`
 * de cada vista. `instance-detail`/`server-detail` quedan afuera a propósito
 * (tienen datos por id que conviene refrescar al entrar).
 */
const KEEP_ALIVE_VIEWS = ["home", "instances", "store", "skins", "versions", "servers", "settings"];

const app = useAppStore();
const accounts = useAccountsStore();
const instances = useInstancesStore();
const settings = useSettingsStore();
const downloads = useDownloadsStore();
const music = useMusicStore();
const skins = useSkinsStore();

function keepAliveKey(route: { name?: string | symbol | null; fullPath: string }) {
  const name = typeof route.name === "string" ? route.name : "";
  if (KEEP_ALIVE_VIEWS.includes(name)) return name;
  // Detalle con :id → otra instancia = otra key
  return route.fullPath;
}

onMounted(() => {
  app.initGameEvents();
  downloads.initEvents();

  // Prioridad: settings + cuentas (UI usable). Hardware/skins después (PowerShell puede tardar).
  void Promise.all([settings.load(), accounts.load(), instances.load()]).then(() => {
    void app.checkUpdate();
    void music.init();
  });
  window.setTimeout(() => {
    void app.loadHardware();
    void skins.refresh();
  }, 80);
  window.setTimeout(() => {
    void instances.scan();
  }, 3000);
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
          <!--
            KeepAlive solo con route.name: :key="fullPath" destruía el cache al cambiar de vista
            (Ajustes / Inicio se recreaban cada vez → freeze "No responde" con IPC pesado).
            Detalle instancia/server usan fullPath para no mezclar ids.
          -->
          <KeepAlive :include="KEEP_ALIVE_VIEWS" :max="8">
            <component
              :is="Component"
              :key="keepAliveKey(route)"
            />
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

