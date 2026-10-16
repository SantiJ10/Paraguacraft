<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import BaseButton from "@/components/common/BaseButton.vue";
import { api, isTauri } from "@/lib/ipc";
import type { BedrockStatus } from "@/lib/types";
import { versionCardImageUrl } from "@/lib/versionCatalog";
import { bedrockLastPlayed, markBedrockPlayed } from "@/composables/useBedrockRecent";
import { formatRelative } from "@/composables/useFormat";
import { useAppStore } from "@/stores/app";

const router = useRouter();
const app = useAppStore();
const status = ref<BedrockStatus | null>(null);
const busy = ref(false);
const error = ref<string | null>(null);
const lastPlayed = ref(bedrockLastPlayed());

const imgUrl = versionCardImageUrl("bedrock");
const canLaunch = computed(
  () =>
    isTauri() &&
    !!status.value?.platformSupported &&
    !!status.value?.premiumAllowed &&
    !!status.value?.installed,
);

onMounted(async () => {
  if (!isTauri()) return;
  try {
    status.value = await api.getBedrockStatus();
  } catch {
    status.value = null;
  }
});

function openVersions() {
  router.push({ name: "versions", query: { card: "bedrock" } });
}

async function play() {
  if (!canLaunch.value) {
    openVersions();
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    markBedrockPlayed();
    lastPlayed.value = bedrockLastPlayed();
    await api.launchBedrock();
    app.setLaunch("running", "Bedrock activo");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="lunar-card group relative flex h-full min-h-[220px] flex-col overflow-hidden">
    <div class="relative min-h-[5rem] flex-1 overflow-hidden cursor-pointer" @click="openVersions">
      <img :src="imgUrl" alt="" class="absolute inset-0 h-full w-full object-cover opacity-80" />
      <div class="absolute inset-0 bg-gradient-to-t from-surface-2 via-surface-2/40 to-transparent" />
      <span
        class="absolute right-2.5 top-2.5 rounded-full bg-black/70 px-2 py-0.5 text-[10px] font-bold text-pc-green backdrop-blur-sm"
      >
        {{ status?.managedActive ? "Versión elegida" : "Última versión" }}
      </span>
    </div>
    <div class="relative bg-surface-2">
      <div class="flex items-center gap-3 p-4 pt-3">
        <div class="min-w-0 flex-1">
          <p class="truncate text-base font-bold text-white">PARAGUA Bedrock</p>
          <p class="text-xs text-gray-400">
            {{ status?.managedActive && status.activeVersion
              ? `v${status.activeVersion} (100% seguro)`
              : status?.activeVersion
                ? `Última ${status.activeVersion} (Store)`
                : "Última versión / Versiones viejas" }}
          </p>
          <p class="mt-0.5 text-[11px] text-gray-500">{{ formatRelative(lastPlayed) }}</p>
        </div>
      </div>
      <div class="flex border-t border-surface-3">
        <BaseButton class="flex-1 rounded-none" :disabled="busy || app.launchPhase === 'running'" @click="play">
          {{ busy ? "…" : canLaunch ? "Jugar" : "Abrir" }}
        </BaseButton>
        <button
          type="button"
          class="border-l border-surface-3 bg-surface-3 px-3 text-xs font-semibold text-gray-300 transition hover:bg-surface-4 hover:text-white"
          title="Elegir entre la última versión o versiones viejas"
          @click="openVersions"
        >
          Versión
        </button>
      </div>
      <p v-if="error" class="px-3 pb-2 text-[10px] text-red-400">{{ error }}</p>
    </div>
  </div>
</template>
