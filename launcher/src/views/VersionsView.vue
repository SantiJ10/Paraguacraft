<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { api } from "@/lib/ipc";
import type { LoaderInfo, MinecraftVersion, VersionChannel } from "@/lib/types";
import SearchInput from "@/components/common/SearchInput.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import { useDownloadsStore } from "@/stores/downloads";

const downloads = useDownloadsStore();

const versions = ref<MinecraftVersion[]>([]);
const loaders = ref<LoaderInfo[]>([]);
const query = ref("");
const channel = ref<VersionChannel | "all">("release");

const selectedVersion = ref<MinecraftVersion | null>(null);
const selectedLoaderId = ref<string>("vanilla");
const selectedLoaderVersion = ref<string>("");

const channels: Array<{ id: VersionChannel | "all"; label: string }> = [
  { id: "release", label: "Oficiales" },
  { id: "snapshot", label: "Snapshots" },
  { id: "old_beta", label: "Beta" },
  { id: "old_alpha", label: "Alpha" },
  { id: "all", label: "Todas" },
];

onMounted(async () => {
  versions.value = await api.getVersions();
  loaders.value = await api.getLoaders("");
  selectVersion(versions.value.find((v) => v.channel === "release") ?? versions.value[0]);
});

const filteredVersions = computed(() =>
  versions.value.filter((v) => {
    const matchChannel = channel.value === "all" || v.channel === channel.value;
    const matchQuery = v.id.toLowerCase().includes(query.value.trim().toLowerCase());
    return matchChannel && matchQuery;
  }),
);

const selectedLoader = computed(() => loaders.value.find((l) => l.id === selectedLoaderId.value) ?? null);

async function selectVersion(v: MinecraftVersion | undefined) {
  if (!v) return;
  selectedVersion.value = v;
  loaders.value = await api.getLoaders(v.id);
}

watch(selectedLoader, (l) => {
  selectedLoaderVersion.value = l?.versions[0] ?? "";
});

function install() {
  if (!selectedVersion.value) return;
  const label =
    `${selectedVersion.value.id} - ${selectedLoader.value?.name}` +
    (selectedLoaderVersion.value && selectedLoaderVersion.value !== "-" ? ` ${selectedLoaderVersion.value}` : "");
  downloads.enqueueDemo(`Instalando ${label}`);
}
</script>

<template>
  <div class="flex h-full">
    <!-- Lista de versiones -->
    <div class="flex w-80 shrink-0 flex-col border-r border-surface-3">
      <div class="space-y-3 border-b border-surface-3 p-4">
        <SearchInput v-model="query" placeholder="Buscar version..." />
        <div class="flex flex-wrap gap-1.5">
          <button
            v-for="c in channels"
            :key="c.id"
            class="rounded-full px-2.5 py-1 text-xs font-semibold transition-colors"
            :class="channel === c.id ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-400 hover:bg-surface-4'"
            @click="channel = c.id"
          >
            {{ c.label }}
          </button>
        </div>
      </div>
      <div class="flex-1 overflow-y-auto p-2">
        <button
          v-for="v in filteredVersions"
          :key="v.id"
          class="flex w-full items-center justify-between rounded-lg px-3 py-2.5 text-left transition-colors"
          :class="selectedVersion?.id === v.id ? 'bg-pc-green/15 text-pc-green' : 'hover:bg-surface-3'"
          @click="selectVersion(v)"
        >
          <span class="font-semibold">{{ v.id }}</span>
          <span class="text-xs" :class="v.installed ? 'text-pc-green' : 'text-gray-600'">
            {{ v.installed ? "Instalada" : v.releaseDate }}
          </span>
        </button>
      </div>
    </div>

    <!-- Configuracion de loader -->
    <div class="flex-1 overflow-y-auto p-8">
      <template v-if="selectedVersion">
        <h1 class="text-2xl font-bold">Minecraft {{ selectedVersion.id }}</h1>
        <p class="text-sm capitalize text-gray-500">{{ selectedVersion.channel.replace("_", " ") }}</p>

        <h2 class="mb-3 mt-6 text-sm font-semibold uppercase tracking-wider text-gray-400">Loader</h2>
        <div class="grid grid-cols-2 gap-3 lg:grid-cols-3">
          <button
            v-for="l in loaders"
            :key="l.id"
            class="rounded-xl border-2 p-3 text-left transition-all"
            :class="selectedLoaderId === l.id ? 'border-pc-green bg-pc-green/10' : 'border-surface-4 bg-surface-2 hover:border-surface-6'"
            @click="selectedLoaderId = l.id"
          >
            <p class="font-semibold">{{ l.name }}</p>
            <p class="mt-0.5 text-xs text-gray-500">{{ l.description }}</p>
          </button>
        </div>

        <template v-if="selectedLoader && selectedLoader.versions[0] !== '-'">
          <h2 class="mb-3 mt-6 text-sm font-semibold uppercase tracking-wider text-gray-400">
            Version exacta del loader
          </h2>
          <select
            v-model="selectedLoaderVersion"
            class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          >
            <option v-for="ver in selectedLoader.versions" :key="ver" :value="ver">{{ ver }}</option>
          </select>
        </template>

        <div class="mt-8">
          <BaseButton size="lg" @click="install">Instalar y crear instancia</BaseButton>
        </div>
      </template>
    </div>
  </div>
</template>
