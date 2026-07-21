<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { openUrl } from "@/lib/ipc";
import type { ContentProvider, ContentType, StoreItem } from "@/lib/types";
import SearchInput from "@/components/common/SearchInput.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import Pagination from "@/components/common/Pagination.vue";
import InstallStoreModal from "@/components/store/InstallStoreModal.vue";
import { useDownloadsStore } from "@/stores/downloads";
import { useStoreStore } from "@/stores/store";
import { formatNumber } from "@/composables/useFormat";

defineOptions({ name: "store" });

const downloads = useDownloadsStore();
const storeStore = useStoreStore();

const query = ref("");
const provider = ref<ContentProvider>("modrinth");
const typeFilter = ref<ContentType>("mod");
/** Mods CF bloqueados por distribución: id → URL del proyecto */
const distributionBlocked = ref<Record<string, { url: string; name: string }>>({});

const installOpen = ref(false);
const installItem = ref<StoreItem | null>(null);

const tabs: Array<{ id: ContentType; label: string }> = [
  { id: "mod", label: "Mods" },
  { id: "modpack", label: "Modpacks" },
  { id: "resourcepack", label: "Resource packs" },
  { id: "shader", label: "Shaders" },
  { id: "datapack", label: "Data packs" },
  { id: "plugin", label: "Plugins" },
];

const state = computed(() => storeStore.stateFor(provider.value, typeFilter.value));
const items = computed(() => state.value.items);
const loading = computed(() => storeStore.loading);
const error = computed(() => storeStore.error);

/** Carga (o recupera de la cache Pinia) la página actual para la pestaña activa. */
function refresh(offset = 0, force = false) {
  void storeStore.search({
    provider: provider.value,
    projectType: typeFilter.value,
    query: query.value,
    offset,
    force,
  });
}

refresh();

watch([provider, typeFilter], () => refresh(0));

function search() {
  refresh(0, true);
}

function onPageChange(offset: number) {
  refresh(offset);
}

function openInstall(item: StoreItem) {
  installItem.value = item;
  installOpen.value = true;
}

async function openManualDownload(item: StoreItem) {
  const blocked = distributionBlocked.value[item.id];
  const url = blocked?.url ?? item.projectUrl;
  if (url) await openUrl(url);
}

function providerLabel(p: ContentProvider): string {
  return p === "modrinth" ? "Modrinth" : "CurseForge";
}

function providerBadge(p: string) {
  return p === "modrinth"
    ? "bg-green-500/20 text-green-400"
    : "bg-orange-500/20 text-orange-400";
}

function onInstalled() {
  downloads.initEvents();
  // El contenido instalado no cambia los resultados de búsqueda: no invalidamos la cache.
}
</script>

<template>
  <div class="p-8">
    <h1 class="text-2xl font-bold">Tienda</h1>
    <p class="text-sm text-gray-500">
      Explora mods, packs y más desde Modrinth y CurseForge. Al instalar, elegirás versión, plataforma e instancia.
    </p>

    <div class="my-5 flex flex-wrap items-center gap-3">
      <div class="flex overflow-hidden rounded-lg border border-surface-3">
        <button
          v-for="p in (['modrinth', 'curseforge'] as ContentProvider[])"
          :key="p"
          class="px-3 py-2 text-sm font-semibold transition-colors"
          :class="provider === p ? 'bg-surface-4 text-white' : 'text-gray-400 hover:text-white'"
          @click="provider = p"
        >
          {{ providerLabel(p) }}
        </button>
      </div>

      <div class="min-w-[220px] flex-1">
        <SearchInput v-model="query" placeholder="Buscar en todo el catálogo..." @keyup.enter="search" />
      </div>
      <BaseButton variant="secondary" @click="search">Buscar</BaseButton>
    </div>

    <div class="mb-5 flex flex-wrap gap-2 border-b border-surface-3 pb-3">
      <button
        v-for="t in tabs"
        :key="t.id"
        class="rounded-lg px-3 py-1.5 text-sm font-semibold transition-colors"
        :class="typeFilter === t.id ? 'bg-surface-4 text-white' : 'text-gray-400 hover:text-white'"
        @click="typeFilter = t.id"
      >
        {{ t.label }}
      </button>
    </div>

    <p v-if="error" class="mb-4 rounded-lg bg-red-500/10 px-4 py-2 text-sm text-red-400">{{ error }}</p>
    <p v-if="loading && !items.length" class="text-sm text-gray-500">Buscando…</p>
    <p v-else-if="!items.length" class="text-sm text-gray-500">
      Sin resultados. Probá otra búsqueda o cambiá de categoría.
    </p>

    <div
      class="grid grid-cols-1 gap-3 lg:grid-cols-2 2xl:grid-cols-3"
      :class="{ 'opacity-60': loading && items.length }"
    >
      <article v-for="item in items" :key="item.id" class="lunar-card flex gap-4 p-4">
        <div class="flex h-16 w-16 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-surface-4 text-2xl">
          <img v-if="item.iconUrl" :src="item.iconUrl" :alt="item.title" class="h-full w-full object-cover" />
          <span v-else>{{ item.title[0] }}</span>
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <h3 class="truncate font-bold">{{ item.title }}</h3>
            <span class="rounded px-1.5 py-0.5 text-[10px] font-bold uppercase" :class="providerBadge(item.provider)">
              {{ providerLabel(item.provider) }}
            </span>
          </div>
          <p class="text-xs text-gray-500">por {{ item.author }}</p>
          <p class="mt-1 line-clamp-2 text-sm text-gray-400">{{ item.description }}</p>
          <div class="mt-2 flex flex-wrap items-center justify-between gap-2">
            <span class="text-xs text-gray-500">
              {{ formatNumber(item.downloads) }} descargas &middot; {{ formatNumber(item.follows) }} seguidores
            </span>
            <div class="flex flex-wrap gap-2">
              <BaseButton size="sm" @click="openInstall(item)">Instalar</BaseButton>
              <BaseButton
                v-if="distributionBlocked[item.id] || item.projectUrl"
                size="sm"
                variant="secondary"
                @click="openManualDownload(item)"
              >
                Descargar manualmente
              </BaseButton>
            </div>
          </div>
        </div>
      </article>
    </div>

    <Pagination
      :offset="state.offset"
      :limit="storeStore.limit"
      :total-hits="state.totalHits"
      :disabled="loading"
      @update:offset="onPageChange"
    />

    <InstallStoreModal v-model:open="installOpen" :item="installItem" @installed="onInstalled" />
  </div>
</template>
