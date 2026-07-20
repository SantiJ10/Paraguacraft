<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useFavoritesStore } from "@/stores/favorites";
import { useAppStore } from "@/stores/app";
import BaseButton from "@/components/common/BaseButton.vue";
import { isTauri } from "@/lib/ipc";
import type { FavoriteServer } from "@/lib/types";

const favorites = useFavoritesStore();
const app = useAppStore();

const showAdd = ref(false);
const name = ref("");
const address = ref("");
const notes = ref("");
const loaderHint = ref<"" | "modern" | "189">("");
const bedrockPort = ref("");
const busy = ref(false);
const error = ref<string | null>(null);
const message = ref<string | null>(null);

onMounted(() => {
  void favorites.load();
});

async function submitAdd() {
  error.value = null;
  busy.value = true;
  try {
    const port = bedrockPort.value.trim() ? Number(bedrockPort.value.trim()) : undefined;
    await favorites.add(
      name.value.trim(),
      address.value.trim(),
      notes.value.trim() || undefined,
      loaderHint.value || undefined,
      undefined,
      port && port > 0 ? port : undefined,
    );
    name.value = "";
    address.value = "";
    notes.value = "";
    loaderHint.value = "";
    bedrockPort.value = "";
    showAdd.value = false;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function join(fav: FavoriteServer) {
  error.value = null;
  try {
    await favorites.join(fav);
  } catch (e) {
    error.value = String(e);
  }
}

async function joinBedrock(fav: FavoriteServer) {
  error.value = null;
  message.value = null;
  try {
    const addr = await favorites.joinBedrock(fav);
    try {
      await navigator.clipboard.writeText(addr);
      message.value = `IP Bedrock copiada (${addr}). Agregala como servidor manualmente en Minecraft.`;
    } catch {
      message.value = `Abrí Minecraft y agregá el servidor manualmente: ${addr}`;
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function remove(id: string) {
  try {
    await favorites.remove(id);
  } catch (e) {
    error.value = String(e);
  }
}

function loaderLabel(fav: FavoriteServer) {
  if (fav.loaderHint === "modern") return "PvP 1.21.11";
  if (fav.loaderHint === "189") return "PvP 1.8.9";
  return "Auto-detectar";
}
</script>

<template>
  <section class="rounded-xl border border-surface-4 bg-surface-2 p-5">
    <div class="mb-4 flex flex-wrap items-center justify-between gap-2">
      <div>
        <h2 class="text-lg font-bold">Servidores favoritos</h2>
        <p class="text-xs text-gray-500">
          Unirse detecta automáticamente si es PvP 1.8.9 o 1.21.11 y lanza el cliente correcto.
        </p>
      </div>
      <BaseButton v-if="isTauri()" size="sm" variant="secondary" @click="showAdd = !showAdd">
        {{ showAdd ? "Cancelar" : "+ Agregar" }}
      </BaseButton>
    </div>

    <form v-if="showAdd" class="mb-4 grid gap-3 sm:grid-cols-2" @submit.prevent="submitAdd">
      <input
        v-model="name"
        type="text"
        placeholder="Nombre (ej. Survival amigos)"
        class="rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
      />
      <input
        v-model="address"
        type="text"
        placeholder="IP o host:puerto"
        class="rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
      />
      <select
        v-model="loaderHint"
        class="rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
      >
        <option value="">Auto-detectar versión</option>
        <option value="189">PvP 1.8.9</option>
        <option value="modern">PvP 1.21.11</option>
      </select>
      <input
        v-model="notes"
        type="text"
        placeholder="Notas (opcional)"
        class="rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
      />
      <input
        v-model="bedrockPort"
        type="number"
        min="1"
        max="65535"
        placeholder="Puerto Bedrock/Geyser (opcional, ej. 19132)"
        class="rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
      />
      <div class="sm:col-span-2 flex justify-end">
        <BaseButton size="sm" :disabled="busy" type="submit">{{ busy ? "Guardando…" : "Guardar" }}</BaseButton>
      </div>
    </form>

    <p v-if="message" class="mb-3 text-sm text-pc-green">{{ message }}</p>
    <p v-if="error" class="mb-3 text-sm text-red-400">{{ error }}</p>

    <p v-if="!favorites.servers.length" class="py-6 text-center text-sm text-gray-500">
      Sin favoritos. Agregá la IP de tu servidor para unirte con un clic.
    </p>

    <ul v-else class="space-y-2">
      <li
        v-for="fav in favorites.servers"
        :key="fav.id"
        class="flex flex-wrap items-center gap-3 rounded-lg border border-surface-4 bg-surface-3 px-4 py-3"
      >
        <div class="min-w-0 flex-1">
          <p class="font-semibold">
            {{ fav.name }}
            <span v-if="fav.fromPlayit" class="ml-1 rounded bg-pc-purple/20 px-1.5 py-0.5 text-[10px] font-bold uppercase text-pc-purple">
              Playit
            </span>
          </p>
          <p class="font-mono text-xs text-gray-400">{{ fav.address }}</p>
          <p class="mt-0.5 text-xs text-gray-500">
            {{ loaderLabel(fav) }}
            <span v-if="fav.notes"> · {{ fav.notes }}</span>
          </p>
        </div>
        <div class="flex gap-2">
          <BaseButton
            size="sm"
            :disabled="favorites.joinBusyId === fav.id || app.launchPhase === 'running'"
            @click="join(fav)"
          >
            {{ favorites.joinBusyId === fav.id ? "…" : "Unirse" }}
          </BaseButton>
          <BaseButton
            v-if="fav.bedrockPort"
            size="sm"
            variant="secondary"
            title="Abre Minecraft Bedrock y copia la IP para agregarla a mano (limitación de la app UWP)"
            :disabled="favorites.joinBusyId === fav.id"
            @click="joinBedrock(fav)"
          >
            Bedrock
          </BaseButton>
          <button
            class="rounded-lg px-2 py-1 text-xs text-gray-500 hover:bg-red-900/30 hover:text-red-300"
            title="Quitar"
            @click="remove(fav.id)"
          >
            ✕
          </button>
        </div>
      </li>
    </ul>
  </section>
</template>
