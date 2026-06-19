<script setup lang="ts">
import { computed, ref } from "vue";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";
import InstanceCard from "@/components/common/InstanceCard.vue";
import SearchInput from "@/components/common/SearchInput.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import type { InstanceSource } from "@/lib/types";

const instances = useInstancesStore();
const app = useAppStore();

const query = ref("");
const sourceFilter = ref<InstanceSource | "all">("all");

const sources: Array<{ id: InstanceSource | "all"; label: string }> = [
  { id: "all", label: "Todas" },
  { id: "paraguacraft", label: "Paraguacraft" },
  { id: "vanilla", label: "Vanilla" },
  { id: "curseforge", label: "CurseForge" },
  { id: "modrinth", label: "Modrinth" },
  { id: "lunar", label: "Lunar" },
  { id: "prism", label: "Prism" },
];

const filtered = computed(() =>
  instances.instances.filter((i) => {
    const matchQuery = i.name.toLowerCase().includes(query.value.trim().toLowerCase());
    const matchSource = sourceFilter.value === "all" || i.source === sourceFilter.value;
    return matchQuery && matchSource;
  }),
);

function play(name: string) {
  app.setLaunch("launching", `Lanzando ${name}...`);
}
</script>

<template>
  <div class="p-8">
    <div class="mb-6 flex items-center justify-between gap-4">
      <div>
        <h1 class="text-2xl font-bold">Instancias</h1>
        <p class="text-sm text-gray-500">{{ instances.instances.length }} instancias detectadas</p>
      </div>
      <div class="flex gap-2">
        <BaseButton variant="secondary">Importar</BaseButton>
        <BaseButton>+ Nueva instancia</BaseButton>
      </div>
    </div>

    <div class="mb-5 flex items-center gap-3">
      <div class="w-72"><SearchInput v-model="query" placeholder="Buscar instancia..." /></div>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="s in sources"
          :key="s.id"
          class="rounded-full px-3 py-1.5 text-xs font-semibold transition-colors"
          :class="sourceFilter === s.id ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-400 hover:bg-surface-4'"
          @click="sourceFilter = s.id"
        >
          {{ s.label }}
        </button>
      </div>
    </div>

    <div v-if="filtered.length" class="grid grid-cols-2 gap-4 xl:grid-cols-3 2xl:grid-cols-4">
      <InstanceCard
        v-for="inst in filtered"
        :key="inst.id"
        :instance="inst"
        :selected="inst.id === instances.selectedId"
        @select="instances.select(inst.id)"
        @play="play(inst.name)"
      />
    </div>
    <p v-else class="py-16 text-center text-gray-500">No hay instancias que coincidan.</p>
  </div>
</template>
