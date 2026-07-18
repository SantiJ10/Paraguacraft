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

const app = useAppStore();
const accounts = useAccountsStore();
const instances = useInstancesStore();
const settings = useSettingsStore();
const downloads = useDownloadsStore();
const music = useMusicStore();
const skins = useSkinsStore();

onMounted(() => {
  app.initGameEvents();
  downloads.initEvents();

  void Promise.all([
    settings.load(),
    accounts.load(),
    instances.load(),
    app.loadHardware(),
    skins.refresh(),
  ]).then(() => {
    void app.checkUpdate();
    void music.init();
  });

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
        <RouterView v-slot="{ Component }">
          <Transition name="fade" mode="out-in">
            <component :is="Component" />
          </Transition>
        </RouterView>
      </main>
      <StatusBar />
      <AiPanel />
      <MusicPlayer />
      <MusicOverlay />
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
