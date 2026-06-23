<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useServersStore } from "@/stores/servers";
import { useAppStore } from "@/stores/app";
import { api, isTauri } from "@/lib/ipc";
import type { ServerType } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";

const SERVER_TYPES: Array<{ id: ServerType; label: string; icon: string; desc: string }> = [
  { id: "paper", label: "Paper", icon: "📄", desc: "Plugins · más estable" },
  { id: "paper-geyser", label: "Paper + Geyser", icon: "🌐", desc: "Plugins + Bedrock" },
  { id: "fabric", label: "Fabric", icon: "🧵", desc: "Mods · server side" },
  { id: "fabric-geyser", label: "Fabric + Geyser", icon: "🧵🌐", desc: "Mods + Bedrock" },
  { id: "forge", label: "Forge", icon: "🔧", desc: "Modpacks CurseForge" },
];

const SERVER_TYPE_LABELS: Record<string, string> = Object.fromEntries(
  SERVER_TYPES.map((t) => [t.id, t.label]),
);

function serverTypeLabel(t: string): string {
  return SERVER_TYPE_LABELS[t] ?? t;
}

const SERVER_RAM_PRESETS_MB = [2048, 4096, 6144, 8192, 12288, 16384] as const;

function formatRamGb(mb: number): string {
  return `${mb / 1024} GB`;
}

const router = useRouter();
const serversStore = useServersStore();
const app = useAppStore();

const loading = ref(true);
const busy = ref<string | null>(null);
const error = ref<string | null>(null);
const showCreate = ref(false);

const newName = ref("Mi servidor");
const newMc = ref("1.21.1");
const newType = ref<ServerType>("paper");
const newRam = ref<number>(4096);

const ramOptions = computed(() => {
  const maxMb = app.hardware ? Math.floor(app.hardware.ramGb * 1024 * 0.75) : Infinity;
  return SERVER_RAM_PRESETS_MB.filter((mb) => mb <= maxMb);
});

onMounted(async () => {
  await app.loadHardware();
  if (ramOptions.value.length && !ramOptions.value.includes(newRam.value as (typeof SERVER_RAM_PRESETS_MB)[number])) {
    newRam.value = ramOptions.value[ramOptions.value.length - 1] ?? 4096;
  }
  await refresh();
});

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    await serversStore.load(true);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function create() {
  busy.value = "create";
  error.value = null;
  try {
    const s = await serversStore.create({
      name: newName.value.trim(),
      mcVersion: newMc.value.trim(),
      serverType: newType.value,
      ramMb: newRam.value,
    });
    showCreate.value = false;
    router.push({ name: "server-detail", params: { id: s.id } });
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

async function importFolder() {
  if (!isTauri()) return;
  busy.value = "import";
  error.value = null;
  try {
    const path = await api.pickServerFolder();
    if (!path) return;
    const s = await serversStore.importFolder(path);
    router.push({ name: "server-detail", params: { id: s.id } });
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

function openServer(id: string) {
  router.push({ name: "server-detail", params: { id } });
}
</script>

<template>
  <div class="p-8">
    <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
      <div>
        <h1 class="text-2xl font-bold">Servidores</h1>
        <p class="text-sm text-gray-500">
          Panel local estilo Modrinth: consola, propiedades, Hangar, admin y Playit.gg.
        </p>
      </div>
      <div class="flex gap-2">
        <BaseButton variant="secondary" :disabled="!!busy || !isTauri()" @click="importFolder">
          Importar carpeta
        </BaseButton>
        <BaseButton @click="showCreate = !showCreate">
          {{ showCreate ? "Cancelar" : "Nuevo servidor" }}
        </BaseButton>
      </div>
    </div>

    <div v-if="showCreate" class="mb-6 rounded-xl border border-surface-3 bg-surface-2 p-4">
      <h2 class="mb-3 text-sm font-semibold uppercase tracking-wider text-gray-400">Nuevo servidor</h2>
      <div class="mb-4 flex flex-wrap gap-2">
        <button
          v-for="t in SERVER_TYPES"
          :key="t.id"
          type="button"
          class="flex min-w-[100px] flex-col items-center gap-0.5 rounded-lg border-2 px-3 py-2 text-xs font-bold transition"
          :class="
            newType === t.id
              ? 'border-pc-green bg-pc-green/10 text-pc-green'
              : 'border-surface-4 bg-surface-3 text-gray-300 hover:border-surface-6'
          "
          @click="newType = t.id"
        >
          <span class="text-base">{{ t.icon }}</span>
          <span>{{ t.label }}</span>
        </button>
      </div>
      <div class="flex flex-wrap items-end gap-3">
        <label class="text-sm">
          Nombre
          <input v-model="newName" class="mt-1 block rounded-lg border border-surface-5 bg-surface-3 px-3 py-2" />
        </label>
        <label class="text-sm">
          MC
          <input v-model="newMc" class="mt-1 block w-28 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2" />
        </label>
        <label class="text-sm">
          RAM
          <select v-model.number="newRam" class="mt-1 block min-w-[7rem] rounded-lg border border-surface-5 bg-surface-3 px-3 py-2">
            <option v-for="mb in ramOptions" :key="mb" :value="mb">{{ formatRamGb(mb) }}</option>
          </select>
        </label>
        <BaseButton :disabled="busy === 'create'" @click="create">Crear</BaseButton>
      </div>
    </div>

    <p v-if="error" class="mb-4 rounded-lg bg-red-500/10 px-4 py-2 text-sm text-red-400">{{ error }}</p>
    <p v-if="loading" class="text-sm text-gray-500">Cargando…</p>
    <p v-else-if="!serversStore.servers.length" class="text-sm text-gray-500">
      No hay servidores. Creá uno nuevo o importá una carpeta existente.
    </p>

    <div class="grid gap-3 sm:grid-cols-2">
      <article
        v-for="s in serversStore.servers"
        :key="s.id"
        class="lunar-card cursor-pointer p-4 transition hover:border-pc-green/40"
        @click="openServer(s.id)"
      >
        <h3 class="font-bold">{{ s.name }}</h3>
        <p class="text-xs text-gray-500">
          {{ s.mcVersion }} · {{ serverTypeLabel(s.serverType) }} · {{ formatRamGb(s.ramMb) }}
          <span v-if="s.customFolder" class="text-amber-500/80"> · importado</span>
        </p>
        <p v-if="s.playitAddress" class="mt-1 truncate text-xs text-pc-green">Playit: {{ s.playitAddress }}</p>
      </article>
    </div>
  </div>
</template>
