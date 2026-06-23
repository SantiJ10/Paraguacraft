<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useFavoritesStore } from "@/stores/favorites";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";
import BaseButton from "@/components/common/BaseButton.vue";
import { isTauri } from "@/lib/ipc";

const favorites = useFavoritesStore();
const instances = useInstancesStore();
const app = useAppStore();

const showAdd = ref(false);
const name = ref("");
const address = ref("");
const notes = ref("");
const busy = ref(false);
const error = ref<string | null>(null);
const joinBusyId = ref<string | null>(null);

const launchTarget = computed(() => instances.selected ?? instances.instances[0] ?? null);

onMounted(() => {
  void favorites.load();
});

async function submitAdd() {
  error.value = null;
  busy.value = true;
  try {
    await favorites.add(name.value.trim(), address.value.trim(), notes.value.trim() || undefined);
    name.value = "";
    address.value = "";
    notes.value = "";
    showAdd.value = false;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function join(favId: string, favAddress: string) {
  const inst = launchTarget.value;
  if (!inst) {
    error.value = "Seleccioná una instancia en la biblioteca primero.";
    return;
  }
  joinBusyId.value = favId;
  error.value = null;
  try {
    await app.launch(inst.id, inst.name, favAddress);
  } catch (e) {
    error.value = String(e);
  } finally {
    joinBusyId.value = null;
  }
}

async function remove(id: string) {
  try {
    await favorites.remove(id);
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <section class="rounded-xl border border-surface-4 bg-surface-2 p-5">
    <div class="mb-4 flex flex-wrap items-center justify-between gap-2">
      <div>
        <h2 class="text-lg font-bold">Servidores favoritos</h2>
        <p class="text-xs text-gray-500">
          Conectá directo con la instancia
          <span v-if="launchTarget" class="text-pc-green">{{ launchTarget.name }}</span>
          <span v-else> (elegí una instancia)</span>
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
      <input
        v-model="notes"
        type="text"
        placeholder="Notas (opcional)"
        class="sm:col-span-2 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
      />
      <div class="sm:col-span-2 flex justify-end">
        <BaseButton size="sm" :disabled="busy" type="submit">{{ busy ? "Guardando…" : "Guardar" }}</BaseButton>
      </div>
    </form>

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
          <p class="font-semibold">{{ fav.name }}</p>
          <p class="font-mono text-xs text-gray-400">{{ fav.address }}</p>
          <p v-if="fav.notes" class="mt-0.5 text-xs text-gray-500">{{ fav.notes }}</p>
        </div>
        <div class="flex gap-2">
          <BaseButton
            size="sm"
            :disabled="!launchTarget || joinBusyId === fav.id || app.launchPhase === 'running'"
            @click="join(fav.id, fav.address)"
          >
            {{ joinBusyId === fav.id ? "…" : "Unirse" }}
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
