<script setup lang="ts">
import { onMounted } from "vue";
import Sidebar from "@/components/layout/Sidebar.vue";
import StatusBar from "@/components/layout/StatusBar.vue";
import AiPanel from "@/components/layout/AiPanel.vue";
import { useAppStore } from "@/stores/app";
import { useAccountsStore } from "@/stores/accounts";
import { useInstancesStore } from "@/stores/instances";
import { useSettingsStore } from "@/stores/settings";

const app = useAppStore();
const accounts = useAccountsStore();
const instances = useInstancesStore();
const settings = useSettingsStore();

onMounted(() => {
  app.loadHardware();
  accounts.load();
  instances.load();
  settings.load();
});
</script>

<template>
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <div class="relative flex flex-1 flex-col overflow-hidden">
      <main class="flex-1 overflow-y-auto">
        <RouterView v-slot="{ Component }">
          <Transition name="fade" mode="out-in">
            <component :is="Component" />
          </Transition>
        </RouterView>
      </main>
      <StatusBar />
      <AiPanel />
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
