<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "@/stores/app";
import { useDownloadsStore } from "@/stores/downloads";
import ProgressBar from "@/components/common/ProgressBar.vue";

const app = useAppStore();
const downloads = useDownloadsStore();

const phaseColor = computed(
  () =>
    ({
      idle: "bg-gray-500",
      preparing: "bg-yellow-500",
      downloading: "bg-blue-500",
      launching: "bg-pc-green",
      running: "bg-pc-green",
    })[app.launchPhase],
);
</script>

<template>
  <footer
    class="flex h-9 shrink-0 items-center justify-between border-t border-surface-3 bg-surface-0 px-4 text-xs text-gray-400"
  >
    <div class="flex items-center gap-2">
      <span class="h-2 w-2 rounded-full" :class="phaseColor"></span>
      <span>{{ app.launchMessage }}</span>
    </div>

    <div v-if="downloads.hasActivity" class="flex items-center gap-3">
      <span class="truncate">{{ downloads.active[0]?.label }}</span>
      <div class="w-32"><ProgressBar :value="downloads.active[0]?.progress ?? 0" variant="ai" /></div>
      <span class="tabular-nums">{{ downloads.active[0]?.speed }}</span>
    </div>

    <div class="flex items-center gap-3">
      <span v-if="app.hardware">{{ app.hardware.cpuName }}</span>
      <span v-if="app.hardware" class="text-gray-600">|</span>
      <span v-if="app.hardware">{{ app.hardware.ramGb }} GB RAM</span>
      <span
        class="rounded px-1.5 py-0.5 text-[10px] font-bold"
        :class="app.runningInTauri ? 'bg-pc-green/20 text-pc-green' : 'bg-yellow-500/20 text-yellow-500'"
      >
        {{ app.runningInTauri ? "TAURI" : "WEB DEMO" }}
      </span>
    </div>
  </footer>
</template>
