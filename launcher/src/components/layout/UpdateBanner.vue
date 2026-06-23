<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "@/stores/app";

const app = useAppStore();

const progressPct = computed(() => Math.round((app.updateProgress?.progress ?? 0) * 100));
</script>

<template>
  <div
    v-if="app.updateInfo?.updateAvailable"
    class="border-b border-pc-green/30 bg-pc-green/10 px-4 py-2 text-sm"
  >
    <div class="flex flex-wrap items-center justify-between gap-3">
      <span>
        Nueva versión {{ app.updateInfo.latestVersion }} disponible (actual: {{ app.updateInfo.currentVersion }})
        <span v-if="app.updateInfo.assetName" class="text-gray-400"> · {{ app.updateInfo.assetName }}</span>
      </span>
      <div class="flex flex-wrap gap-2">
        <button
          v-if="app.updateInfo.inAppInstall"
          class="rounded-lg bg-pc-green px-3 py-1 font-semibold text-black disabled:opacity-50"
          :disabled="app.updating"
          @click="app.installUpdate()"
        >
          {{ app.updating ? "Actualizando…" : "Actualizar ahora" }}
        </button>
        <button
          v-else
          class="rounded-lg bg-pc-green px-3 py-1 font-semibold text-black"
          @click="app.openUpdateDownload()"
        >
          Descargar
        </button>
        <button class="rounded-lg px-2 py-1 text-gray-400 hover:text-white" @click="app.dismissUpdateBanner()">×</button>
      </div>
    </div>
    <div v-if="app.updating && app.updateProgress" class="mt-2">
      <div class="mb-1 flex justify-between text-xs text-gray-400">
        <span>{{ app.updateProgress.message }}</span>
        <span>{{ progressPct }}%</span>
      </div>
      <div class="h-1.5 overflow-hidden rounded-full bg-surface-4">
        <div class="h-full bg-pc-green transition-all" :style="{ width: `${progressPct}%` }" />
      </div>
    </div>
  </div>
</template>
