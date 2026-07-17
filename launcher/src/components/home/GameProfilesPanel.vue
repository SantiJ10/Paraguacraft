<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAppStore } from "@/stores/app";
import { api, isTauri } from "@/lib/ipc";
import type { GameProfile } from "@/lib/types";

const app = useAppStore();

const profiles = ref<GameProfile[]>([]);
const loading = ref(true);
const busyId = ref<string | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  if (!isTauri()) {
    loading.value = false;
    return;
  }
  try {
    profiles.value = await api.listGameProfiles();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
});

async function launch(profile: GameProfile) {
  if (!profile.available) return;
  busyId.value = profile.id;
  error.value = null;
  try {
    await app.initGameEvents();
    app.setLaunch(
      "preparing",
      profile.competeMode ? `Competir — ${profile.name}…` : `Perfil ${profile.name}…`,
    );
    await api.launchGameProfile(profile.id);
  } catch (e) {
    error.value = String(e);
    app.setLaunch("idle", "Listo para jugar");
  } finally {
    busyId.value = null;
  }
}

function tierClass(available: boolean) {
  return available
    ? "border-pc-green/30 bg-pc-green/5 hover:border-pc-green/50"
    : "border-surface-4 bg-surface-3/40 opacity-60";
}
</script>

<template>
  <section class="rounded-xl border border-surface-4 bg-surface-2 p-5">
    <div class="mb-4">
      <h2 class="text-lg font-bold">Perfiles de juego</h2>
      <p class="text-xs text-gray-500">Un clic: instancia + Competir + servidor (sin menús extra)</p>
    </div>

    <p v-if="loading" class="py-4 text-center text-sm text-gray-500">Cargando perfiles…</p>
    <p v-else-if="error" class="text-sm text-red-400">{{ error }}</p>

    <div v-else class="grid gap-3 sm:grid-cols-2">
      <button
        v-for="profile in profiles"
        :key="profile.id"
        type="button"
        class="rounded-xl border p-4 text-left transition-colors"
        :class="tierClass(profile.available)"
        :disabled="!profile.available || busyId !== null || app.launchPhase === 'running'"
        @click="launch(profile)"
      >
        <div class="flex items-start justify-between gap-2">
          <div class="min-w-0">
            <p class="font-bold text-white">{{ profile.name }}</p>
            <p class="mt-1 text-xs text-gray-400">{{ profile.description }}</p>
            <p v-if="profile.resolvedInstanceName" class="mt-2 text-xs text-pc-green">
              {{ profile.resolvedInstanceName }}
              <span v-if="profile.competeMode"> · Competir</span>
              <span v-if="profile.serverAddress"> · {{ profile.serverAddress }}</span>
            </p>
            <p v-else class="mt-2 text-xs text-amber-400/90">Sin instancia compatible</p>
          </div>
          <span
            v-if="busyId === profile.id"
            class="shrink-0 text-xs text-pc-green"
          >…</span>
          <span
            v-else-if="profile.competeMode && profile.available"
            class="shrink-0 rounded bg-pc-green/20 px-2 py-0.5 text-[10px] font-bold uppercase text-pc-green"
          >PvP</span>
        </div>
      </button>
    </div>
  </section>
</template>
