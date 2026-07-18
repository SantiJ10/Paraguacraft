<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useAppStore } from "@/stores/app";
import { useFavoritesStore } from "@/stores/favorites";
import { api, isTauri } from "@/lib/ipc";
import type { GameProfile } from "@/lib/types";

const app = useAppStore();
const favorites = useFavoritesStore();

const profiles = ref<GameProfile[]>([]);
const loading = ref(true);
const busyId = ref<string | null>(null);
const error = ref<string | null>(null);

/** Destino elegido por perfil (id del perfil → id destino). */
const destinationByProfile = reactive<Record<string, string>>({});
const favoriteByProfile = reactive<Record<string, string>>({});

onMounted(async () => {
  if (!isTauri()) {
    loading.value = false;
    return;
  }
  try {
    const [, list] = await Promise.all([favorites.load(), api.listGameProfiles()]);
    profiles.value = list;
    for (const p of list) {
      if (p.destinations?.length) {
        destinationByProfile[p.id] = p.destinations[0]?.id ?? "menu";
      }
      if (favorites.servers.length) {
        favoriteByProfile[p.id] = favorites.servers[0]?.id ?? "";
      }
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
});

const favoriteOptions = computed(() =>
  favorites.servers.map((s) => ({ id: s.id, label: `${s.name} (${s.address})` })),
);

function selectedDestination(profile: GameProfile) {
  const list = profile.destinations ?? [];
  const id = destinationByProfile[profile.id] ?? list[0]?.id ?? "menu";
  return list.find((d) => d.id === id) ?? list[0];
}

function needsFavoritePicker(profile: GameProfile) {
  return selectedDestination(profile)?.needsFavorite;
}

async function launch(profile: GameProfile) {
  if (!profile.available) return;
  const dest = selectedDestination(profile);
  if (dest?.needsFavorite && !favoriteOptions.value.length) {
    error.value = "Agregá un servidor favorito en Inicio para usar este destino.";
    return;
  }
  busyId.value = profile.id;
  error.value = null;
  try {
    await app.initGameEvents();
    const destLabel = dest?.label ?? profile.name;
    app.setLaunch(
      "preparing",
      profile.competeMode && (dest?.id === "hypixel" || dest?.id === "favorite")
        ? `Competir — ${destLabel}…`
        : dest?.id === "training"
          ? `Práctica PvP — ${destLabel}…`
          : `${profile.name} — ${destLabel}…`,
    );
    await api.launchGameProfile(
      profile.id,
      dest?.id,
      dest?.needsFavorite ? favoriteByProfile[profile.id] : undefined,
    );
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
      <p class="text-xs text-gray-500">
        Un clic: instancia + destino + optimización (Competir en 1.8.9 · mods por PC en 1.21.11)
      </p>
    </div>

    <p v-if="loading" class="py-4 text-center text-sm text-gray-500">Cargando perfiles…</p>
    <p v-else-if="error" class="text-sm text-red-400">{{ error }}</p>

    <div v-else class="grid gap-3 sm:grid-cols-2">
      <div
        v-for="profile in profiles"
        :key="profile.id"
        class="rounded-xl border p-4 transition-colors"
        :class="tierClass(profile.available)"
      >
        <div class="flex items-start justify-between gap-2">
          <div class="min-w-0 flex-1">
            <p class="font-bold text-white">{{ profile.name }}</p>
            <p class="mt-1 text-xs text-gray-400">{{ profile.description }}</p>
            <p v-if="profile.modPackSummary" class="mt-1 text-xs text-pc-purple">
              {{ profile.modPackSummary }}
              <span v-if="profile.hardwareTier"> · PC {{ profile.hardwareTier }}</span>
            </p>
            <p v-if="profile.resolvedInstanceName" class="mt-2 text-xs text-pc-green">
              {{ profile.resolvedInstanceName }}
              <span v-if="profile.competeMode"> · Competir</span>
            </p>
            <p v-else class="mt-2 text-xs text-amber-400/90">Sin instancia compatible</p>
          </div>
          <span
            v-if="profile.competeMode && profile.available"
            class="shrink-0 rounded bg-pc-green/20 px-2 py-0.5 text-[10px] font-bold uppercase text-pc-green"
          >
            PvP
          </span>
          <span
            v-else-if="profile.id === 'modern-pvp'"
            class="shrink-0 rounded bg-pc-purple/20 px-2 py-0.5 text-[10px] font-bold uppercase text-pc-purple"
          >
            1.21
          </span>
        </div>

        <div v-if="profile.destinations?.length" class="mt-3 space-y-2">
          <label class="block text-[10px] font-bold uppercase tracking-wider text-gray-500">
            Destino
          </label>
          <select
            v-model="destinationByProfile[profile.id]"
            class="w-full rounded-lg border border-surface-4 bg-surface-3 px-2 py-1.5 text-xs text-gray-200"
            :disabled="busyId !== null || app.launchPhase === 'running'"
          >
            <option v-for="d in profile.destinations" :key="d.id" :value="d.id">
              {{ d.label }}{{ d.serverAddress ? ` · ${d.serverAddress}` : "" }}
            </option>
          </select>
          <select
            v-if="needsFavoritePicker(profile)"
            v-model="favoriteByProfile[profile.id]"
            class="w-full rounded-lg border border-surface-4 bg-surface-3 px-2 py-1.5 text-xs text-gray-200"
            :disabled="busyId !== null || app.launchPhase === 'running'"
          >
            <option v-for="f in favoriteOptions" :key="f.id" :value="f.id">{{ f.label }}</option>
          </select>
        </div>

        <button
          type="button"
          class="mt-3 w-full rounded-lg bg-pc-green/90 py-2 text-sm font-bold text-black transition hover:bg-pc-green disabled:opacity-40"
          :disabled="!profile.available || busyId !== null || app.launchPhase === 'running'"
          @click="launch(profile)"
        >
          {{ busyId === profile.id ? "Iniciando…" : "Jugar" }}
        </button>
      </div>
    </div>
  </section>
</template>
