<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import BaseButton from "@/components/common/BaseButton.vue";
import { api, isTauri } from "@/lib/ipc";
import type { HorizonStatus } from "@/lib/types";
import { coverKeyForMcVersion, versionCardImageUrl } from "@/lib/versionCatalog";
import { formatRelative } from "@/composables/useFormat";
import { useAppStore } from "@/stores/app";

const router = useRouter();
const app = useAppStore();
const status = ref<HorizonStatus | null>(null);
const busy = ref(false);
const error = ref<string | null>(null);

const lastPlayed = ref<string | null>(null);
try {
  lastPlayed.value = localStorage.getItem("pc.horizon.lastPlayed");
} catch {
  lastPlayed.value = null;
}

const imgUrl = versionCardImageUrl(coverKeyForMcVersion("26.3"));
const canDirectLaunch = computed(
  () =>
    isTauri() &&
    !!status.value?.platformSupported &&
    !!status.value?.premiumAllowed &&
    !!status.value?.installed,
);

onMounted(async () => {
  if (!isTauri()) return;
  try {
    status.value = await api.getHorizonStatus(false);
  } catch {
    status.value = null;
  }
});

function openHorizon() {
  router.push({ name: "horizon" });
}

async function play() {
  if (!canDirectLaunch.value) {
    openHorizon();
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    const pid = await api.launchHorizonEngine();
    const nowIso = new Date().toISOString();
    lastPlayed.value = nowIso;
    try {
      localStorage.setItem("pc.horizon.lastPlayed", nowIso);
    } catch {
      /* ignore */
    }
    app.setLaunch("running", `Horizon 26.3 activo (PID ${pid})`);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="lunar-card group relative flex h-full min-h-[220px] flex-col overflow-hidden">
    <div class="relative min-h-[5rem] flex-1 cursor-pointer overflow-hidden" @click="openHorizon">
      <img :src="imgUrl" alt="" class="absolute inset-0 h-full w-full object-cover opacity-80" />
      <div class="absolute inset-0 bg-gradient-to-t from-surface-2 via-emerald-950/35 to-transparent" />
      <span
        class="absolute right-2.5 top-2.5 rounded-full bg-black/75 px-2.5 py-0.5 text-[10px] font-bold text-amber-300 backdrop-blur-sm"
      >
        ⚡ Rust + Vulkan · 26.3
      </span>
    </div>
    <div class="relative bg-surface-2">
      <div class="flex items-center gap-3 p-4 pt-3">
        <div class="min-w-0 flex-1">
          <p class="truncate text-base font-bold text-white">PARAGUA Horizon</p>
          <p class="text-xs text-gray-400">
            {{
              !status?.premiumAllowed
                ? "Cuenta Microsoft requerida"
                : status?.installed
                  ? "Motor nativo listo · Hasta 1024 chunks"
                  : "Sin Java · Arranque en <1s"
            }}
          </p>
          <p class="mt-0.5 text-[11px] text-gray-500">{{ formatRelative(lastPlayed) }}</p>
        </div>
      </div>
      <div class="flex border-t border-surface-3">
        <BaseButton class="flex-1 rounded-none" :disabled="busy || app.launchPhase === 'running'" @click="play">
          {{ busy ? "…" : canDirectLaunch ? "Jugar" : "Configurar" }}
        </BaseButton>
        <button
          type="button"
          class="border-l border-surface-3 bg-surface-3 px-3 text-xs font-semibold text-gray-300 transition hover:bg-surface-4 hover:text-white"
          title="Abrir panel de Horizon (assets 26.3, motor Vulkan y estado)"
          @click="openHorizon"
        >
          Panel
        </button>
      </div>
      <p v-if="error" class="px-3 pb-2 text-[10px] text-red-400">{{ error }}</p>
    </div>
  </div>
</template>
