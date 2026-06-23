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

const footerTask = computed(() => downloads.failed[0] ?? downloads.active[0] ?? null);
const isError = computed(() => footerTask.value?.status === "error");
</script>

<template>
  <footer
    class="flex h-9 shrink-0 items-center justify-between border-t border-surface-3 bg-surface-0 px-4 text-xs text-gray-400"
  >
    <div class="flex min-w-0 items-center gap-2">
      <span class="h-2 w-2 shrink-0 rounded-full" :class="phaseColor"></span>
      <span class="truncate">{{ app.launchMessage }}</span>
    </div>

    <div v-if="footerTask" class="flex min-w-0 max-w-[50%] items-center gap-3">
      <span class="truncate" :class="isError ? 'text-red-400' : ''">{{ footerTask.label }}</span>
      <div v-if="!isError" class="w-32"><ProgressBar :value="footerTask.progress ?? 0" variant="ai" /></div>
      <span v-if="!isError" class="shrink-0 tabular-nums">{{ footerTask.speed }}</span>
      <span v-if="isError && footerTask.error" class="truncate text-red-400" :title="footerTask.error">
        {{ footerTask.error }}
      </span>
      <span
        v-if="isError && footerTask.failedFile"
        class="hidden truncate font-mono text-[10px] text-gray-500 sm:inline"
        :title="footerTask.failedFile"
      >
        {{ footerTask.failedFile }}
      </span>
    </div>

    <div class="flex shrink-0 items-center gap-3">
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
