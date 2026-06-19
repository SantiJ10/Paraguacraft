<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api } from "@/lib/ipc";
import type { ContentType, StoreItem } from "@/lib/types";
import SearchInput from "@/components/common/SearchInput.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import { useDownloadsStore } from "@/stores/downloads";
import { formatNumber } from "@/composables/useFormat";

const downloads = useDownloadsStore();
const query = ref("");
const items = ref<StoreItem[]>([]);
const typeFilter = ref<ContentType | "all">("all");

const tabs: Array<{ id: ContentType | "all"; label: string }> = [
  { id: "all", label: "Todo" },
  { id: "mod", label: "Mods" },
  { id: "modpack", label: "Modpacks" },
  { id: "resourcepack", label: "Resource packs" },
  { id: "shader", label: "Shaders" },
  { id: "datapack", label: "Data packs" },
  { id: "plugin", label: "Plugins" },
];

async function refresh() {
  items.value = await api.searchStore(query.value);
}
onMounted(refresh);

const filtered = computed(() =>
  typeFilter.value === "all" ? items.value : items.value.filter((i) => i.type === typeFilter.value),
);

function providerBadge(p: string) {
  return p === "modrinth"
    ? "bg-green-500/20 text-green-400"
    : "bg-orange-500/20 text-orange-400";
}
</script>

<template>
  <div class="p-8">
    <h1 class="text-2xl font-bold">Tienda</h1>
    <p class="text-sm text-gray-500">Mods, modpacks, shaders y mas desde Modrinth y CurseForge</p>

    <div class="my-5 flex items-center gap-3">
      <div class="flex-1"><SearchInput v-model="query" placeholder="Buscar contenido..." @keyup.enter="refresh" /></div>
      <BaseButton variant="secondary" @click="refresh">Buscar</BaseButton>
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

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-2 2xl:grid-cols-3">
      <article
        v-for="item in filtered"
        :key="item.id"
        class="lunar-card flex gap-4 p-4"
      >
        <div class="flex h-16 w-16 shrink-0 items-center justify-center rounded-lg bg-surface-4 text-2xl">
          {{ item.title[0] }}
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <h3 class="truncate font-bold">{{ item.title }}</h3>
            <span class="rounded px-1.5 py-0.5 text-[10px] font-bold uppercase" :class="providerBadge(item.provider)">
              {{ item.provider }}
            </span>
          </div>
          <p class="text-xs text-gray-500">por {{ item.author }}</p>
          <p class="mt-1 line-clamp-2 text-sm text-gray-400">{{ item.description }}</p>
          <div class="mt-2 flex items-center justify-between">
            <span class="text-xs text-gray-500">
              {{ formatNumber(item.downloads) }} descargas &middot; {{ formatNumber(item.followers) }} seguidores
            </span>
            <BaseButton size="sm" @click="downloads.enqueueDemo(`Descargando ${item.title}`)">Instalar</BaseButton>
          </div>
        </div>
      </article>
    </div>
  </div>
</template>
