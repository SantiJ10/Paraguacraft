<script setup lang="ts">
import ProfileChip from "@/components/layout/ProfileChip.vue";
import MusicWidget from "@/components/layout/MusicWidget.vue";
import MusicPanel from "@/components/layout/MusicPanel.vue";
import { openUrl } from "@/lib/ipc";
import buttonCafecito from "@/assets/button_5.svg";

const CAFECITO_URL = "https://cafecito.app/amin1001";

async function openCafecito() {
  // Navegador del sistema (Tauri shell), no el WebView interno.
  await openUrl(CAFECITO_URL);
}
</script>

<template>
  <header
    class="relative flex shrink-0 items-center justify-between gap-3 border-b border-surface-3/60 bg-surface-0/50 px-4 py-2"
  >
    <button
      type="button"
      class="flex shrink-0 items-center rounded-lg opacity-90 transition hover:opacity-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-pc-green/40"
      title="Invitame un cafecito"
      aria-label="Donar en Cafecito"
      @click="openCafecito"
    >
      <img
        :src="buttonCafecito"
        alt="Invitame un café en cafecito.app"
        class="h-8 w-auto max-w-[192px] object-contain"
        draggable="false"
      />
    </button>

    <div class="flex items-center gap-2">
      <div class="relative">
        <MusicWidget />
        <MusicPanel />
      </div>
      <ProfileChip />
    </div>
  </header>
</template>
