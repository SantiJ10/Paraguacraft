<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";
import InstanceCard from "@/components/common/InstanceCard.vue";
import SearchInput from "@/components/common/SearchInput.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import NewInstanceModal from "@/components/instance/NewInstanceModal.vue";
import ImportMrpackModal from "@/components/instance/ImportMrpackModal.vue";
import { isTauri } from "@/lib/ipc";
import type { Instance, InstanceSource } from "@/lib/types";

const router = useRouter();
const instances = useInstancesStore();
const app = useAppStore();

const query = ref("");
const sourceFilter = ref<InstanceSource | "all">("all");
const showNew = ref(false);
const showImport = ref(false);
const actionError = ref<string | null>(null);
const launchingId = ref<string | null>(null);

const sources: Array<{ id: InstanceSource | "all"; label: string }> = [
  { id: "all", label: "Todas" },
  { id: "paraguacraft", label: "Paraguacraft" },
  { id: "vanilla", label: "Vanilla" },
  { id: "lunar", label: "Lunar" },
  { id: "prism", label: "Prism" },
  { id: "tlauncher", label: "TLauncher" },
  { id: "sklauncher", label: "SKLauncher" },
];

const filtered = computed(() =>
  instances.instances.filter((i) => {
    const matchQuery = i.name.toLowerCase().includes(query.value.trim().toLowerCase());
    const matchSource = sourceFilter.value === "all" || i.source === sourceFilter.value;
    return matchQuery && matchSource;
  }),
);

function openInstance(inst: Instance) {
  instances.select(inst.id);
  router.push({ name: "instance-detail", params: { id: inst.id } });
}

async function play(inst: Instance) {
  actionError.value = null;
  launchingId.value = inst.id;
  try {
    await app.launch(inst.id, inst.name);
  } catch (e) {
    actionError.value = String(e);
  } finally {
    launchingId.value = null;
  }
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
        <BaseButton variant="secondary" :disabled="instances.scanning || !isTauri()" @click="instances.scan()">
          {{ instances.scanning ? "Escaneando..." : "Detectar otros launchers" }}
        </BaseButton>
        <BaseButton variant="secondary" @click="showImport = true">Importar .mrpack</BaseButton>
        <BaseButton @click="showNew = true">+ Nueva instancia</BaseButton>
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

    <p v-if="actionError" class="mb-4 text-sm text-red-400">{{ actionError }}</p>

    <div v-if="filtered.length" class="grid grid-cols-2 gap-4 xl:grid-cols-3 2xl:grid-cols-4">
      <InstanceCard
        v-for="inst in filtered"
        :key="inst.id"
        :instance="inst"
        :selected="inst.id === instances.selectedId"
        @open="openInstance(inst)"
        @play="play(inst)"
      />
    </div>
    <p v-else class="py-16 text-center text-gray-500">No hay instancias que coincidan.</p>

    <NewInstanceModal v-if="showNew" @close="showNew = false" />
    <ImportMrpackModal v-if="showImport" @close="showImport = false" />
  </div>
</template>
