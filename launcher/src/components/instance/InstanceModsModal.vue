<script setup lang="ts">
import { computed, onMounted, ref, shallowRef } from "vue";
import { useRouter } from "vue-router";
import type { Instance, InstanceContentItem, ModConflict } from "@/lib/types";
import { api } from "@/lib/ipc";
import { useInstancesStore } from "@/stores/instances";
import { useDownloadsStore } from "@/stores/downloads";
import BaseModal from "@/components/common/BaseModal.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import SearchInput from "@/components/common/SearchInput.vue";
import InstanceIcon from "@/components/instance/InstanceIcon.vue";
import InstanceContentRow from "@/components/instance/InstanceContentRow.vue";

const props = defineProps<{
  instance: Instance;
}>();

const emit = defineEmits<{
  close: [];
}>();

const router = useRouter();
const instancesStore = useInstancesStore();
const downloadsStore = useDownloadsStore();

const loading = ref(true);
const busy = ref(false);
const updating = ref(false);
const error = ref<string | null>(null);
const message = ref<string | null>(null);

const items = shallowRef<InstanceContentItem[]>([]);
const conflicts = ref<ModConflict[]>([]);

const search = ref("");
const kindFilter = ref<"mod" | "resourcepack" | "shader" | "all">("mod");
const statusFilter = ref<"all" | "active" | "disabled" | "warn">("all");

const kindTabs = [
  { id: "mod" as const, label: "Mods", folder: "mods" as const },
  { id: "resourcepack" as const, label: "Resource Packs", folder: "resourcepacks" as const },
  { id: "shader" as const, label: "Shaders", folder: "shaderpacks" as const },
  { id: "all" as const, label: "Todo", folder: "mods" as const },
];

const statusTabs = [
  { id: "all" as const, label: "Todos" },
  { id: "active" as const, label: "Activos" },
  { id: "disabled" as const, label: "Desactivados" },
  { id: "warn" as const, label: "Con avisos" },
];

const modsSummary = computed(() => {
  const scoped = kindFilter.value === "all"
    ? items.value
    : items.value.filter((i) => i.kind === kindFilter.value);
  const active = scoped.filter((i) => i.enabled).length;
  const disabled = scoped.length - active;
  const warn = scoped.filter((i) => i.compatible === false).length;
  const bytes = scoped.reduce((acc, i) => acc + (i.sizeBytes || 0), 0);
  const mb = (bytes / (1024 * 1024)).toFixed(1);
  return { total: scoped.length, active, disabled, warn, mb };
});

const filteredItems = computed(() => {
  const q = search.value.trim().toLowerCase();
  return items.value.filter((item) => {
    if (kindFilter.value !== "all" && item.kind !== kindFilter.value) return false;
    if (statusFilter.value === "active" && !item.enabled) return false;
    if (statusFilter.value === "disabled" && item.enabled) return false;
    if (statusFilter.value === "warn" && item.compatible !== false) return false;
    if (q) {
      const hay = `${item.displayName ?? ""} ${item.name} ${item.author ?? ""} ${item.description ?? ""}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });
});

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    items.value = await api.listInstanceContent(props.instance.id);
    if (!props.instance.id.startsWith("ext::")) {
      try {
        conflicts.value = await api.scanModConflicts(props.instance.id);
      } catch {
        conflicts.value = [];
      }
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void refresh();
});

async function toggleItem(item: InstanceContentItem) {
  error.value = null;
  try {
    await api.toggleInstanceContent(props.instance.id, item.path, !item.enabled);
    items.value = await api.listInstanceContent(props.instance.id);
    if (!props.instance.id.startsWith("ext::")) {
      conflicts.value = await api.scanModConflicts(props.instance.id).catch(() => []);
    }
    await instancesStore.load(true);
  } catch (e) {
    error.value = String(e);
  }
}

async function revealItem(item: InstanceContentItem) {
  try {
    await api.revealInstanceContent(props.instance.id, item.path);
  } catch (e) {
    error.value = String(e);
  }
}

async function removeItem(item: InstanceContentItem) {
  const label = item.displayName || item.name;
  if (!window.confirm(`¿Eliminar "${label}" de ${props.instance.name}?`)) return;
  error.value = null;
  try {
    await api.removeInstanceContent(props.instance.id, item.path);
    message.value = `"${label}" eliminado.`;
    items.value = await api.listInstanceContent(props.instance.id);
    await instancesStore.load(true);
  } catch (e) {
    error.value = String(e);
  }
}

async function addLocalFiles() {
  busy.value = true;
  error.value = null;
  message.value = null;
  const folder =
    kindFilter.value === "resourcepack"
      ? "resourcepacks"
      : kindFilter.value === "shader"
        ? "shaderpacks"
        : "mods";
  try {
    const added = await api.pickAndAddInstanceContent(props.instance.id, folder);
    if (added > 0) {
      message.value = `${added} archivo(s) agregados a ${folder}/.`;
      items.value = await api.listInstanceContent(props.instance.id);
      await instancesStore.load(true);
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function updateAll() {
  updating.value = true;
  error.value = null;
  message.value = null;
  try {
    await downloadsStore.initEvents();
    const n = await api.updateInstanceContent(props.instance.id);
    message.value = n > 0 ? `${n} mod(s)/pack(s) actualizados.` : "Todo el contenido está al día.";
    items.value = await api.listInstanceContent(props.instance.id);
  } catch (e) {
    error.value = String(e);
  } finally {
    updating.value = false;
  }
}

async function openFolder() {
  try {
    await api.openInstanceFolder(props.instance.id);
  } catch (e) {
    error.value = String(e);
  }
}

function goStore() {
  instancesStore.select(props.instance.id);
  emit("close");
  router.push({ name: "store" });
}
</script>

<template>
  <BaseModal :title="`Gestor de Mods · ${instance.name}`" max-width="max-w-3xl" @close="emit('close')">
    <div class="space-y-4">
      <!-- Cabecera de instancia y resumen -->
      <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-surface-4 bg-surface-3/60 p-3.5">
        <div class="flex items-center gap-3">
          <InstanceIcon :icon="instance.icon" size="md" />
          <div>
            <p class="font-bold text-white">{{ instance.name }}</p>
            <p class="text-xs text-gray-400">
              Minecraft {{ instance.mcVersion }} ·
              <span class="capitalize">{{ instance.loader.replace(/-/g, " ") }}</span>
            </p>
          </div>
        </div>
        <div class="flex flex-wrap items-center gap-2 text-xs">
          <span class="rounded-lg bg-pc-green/15 px-2.5 py-1 font-semibold text-pc-green">
            {{ modsSummary.active }} activos
          </span>
          <span
            v-if="modsSummary.disabled > 0"
            class="rounded-lg bg-surface-4 px-2.5 py-1 font-semibold text-gray-300"
          >
            {{ modsSummary.disabled }} off
          </span>
          <span
            v-if="modsSummary.warn > 0"
            class="rounded-lg bg-amber-500/20 px-2.5 py-1 font-semibold text-amber-300"
          >
            {{ modsSummary.warn }} aviso(s)
          </span>
          <span class="rounded-lg bg-surface-4 px-2.5 py-1 font-mono text-gray-400">
            {{ modsSummary.mb }} MB
          </span>
        </div>
      </div>

      <!-- Avisos de conflictos detectados -->
      <div
        v-if="conflicts.length"
        class="space-y-1.5 rounded-xl border border-amber-500/40 bg-amber-950/25 p-3 text-xs"
      >
        <p class="font-bold text-amber-200">⚠ Conflictos detectados en esta instancia</p>
        <div v-for="(c, idx) in conflicts" :key="idx" class="text-amber-100/90">
          <span class="font-semibold">{{ c.title }}:</span> {{ c.detail }}
          <span v-if="c.mods.length" class="font-mono text-amber-300"> ({{ c.mods.join(", ") }})</span>
        </div>
      </div>

      <!-- Barra de acciones rápidas -->
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="flex flex-wrap gap-1.5">
          <button
            v-for="kt in kindTabs"
            :key="kt.id"
            type="button"
            class="rounded-lg px-3 py-1.5 text-xs font-semibold transition-colors"
            :class="
              kindFilter === kt.id
                ? 'bg-pc-green text-black'
                : 'bg-surface-3 text-gray-400 hover:text-white'
            "
            @click="kindFilter = kt.id"
          >
            {{ kt.label }}
          </button>
        </div>
        <div class="flex flex-wrap gap-1.5">
          <BaseButton size="sm" variant="primary" :disabled="busy" @click="goStore">
            + Descargar de Tienda
          </BaseButton>
          <BaseButton size="sm" variant="secondary" :disabled="busy" @click="addLocalFiles">
            + Archivo local
          </BaseButton>
          <BaseButton size="sm" variant="secondary" :disabled="updating" @click="updateAll">
            {{ updating ? "Actualizando…" : "Actualizar" }}
          </BaseButton>
          <BaseButton size="sm" variant="ghost" @click="openFolder">
            Carpeta
          </BaseButton>
        </div>
      </div>

      <!-- Buscador y filtro de estado -->
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="min-w-[220px] flex-1">
          <SearchInput v-model="search" placeholder="Filtrar por nombre, autor o archivo…" />
        </div>
        <div class="flex flex-wrap gap-1">
          <button
            v-for="st in statusTabs"
            :key="st.id"
            type="button"
            class="rounded-full px-2.5 py-1 text-[11px] font-semibold transition-colors"
            :class="
              statusFilter === st.id
                ? 'bg-surface-4 text-white ring-1 ring-pc-green/50'
                : 'bg-surface-3/60 text-gray-400 hover:text-white'
            "
            @click="statusFilter = st.id"
          >
            {{ st.label }}
          </button>
        </div>
      </div>

      <p v-if="error" class="rounded-lg border border-red-500/30 bg-red-950/30 px-3 py-2 text-xs text-red-300">
        {{ error }}
      </p>
      <p v-if="message" class="rounded-lg border border-pc-green/30 bg-pc-green/10 px-3 py-2 text-xs text-pc-green">
        {{ message }}
      </p>

      <!-- Lista visual de mods -->
      <div class="max-h-[52vh] overflow-y-auto rounded-xl border border-surface-4 bg-surface-1/70">
        <div v-if="loading" class="py-14 text-center text-sm text-gray-500">
          Escaneando mods e íconos de la instancia…
        </div>
        <div
          v-else-if="!items.length"
          class="space-y-3 py-14 text-center text-sm text-gray-400"
        >
          <p>Esta instancia todavía no tiene mods ni packs instalados.</p>
          <div class="flex justify-center gap-2">
            <BaseButton size="sm" @click="goStore">Explorar Tienda de Mods</BaseButton>
            <BaseButton size="sm" variant="secondary" @click="addLocalFiles">Agregar .jar local</BaseButton>
          </div>
        </div>
        <div
          v-else-if="!filteredItems.length"
          class="py-12 text-center text-sm text-gray-500"
        >
          Ningún archivo coincide con el filtro actual.
        </div>
        <ul v-else class="divide-y divide-surface-3">
          <InstanceContentRow
            v-for="item in filteredItems"
            :key="item.path"
            :item="item"
            @toggle="toggleItem(item)"
            @reveal="revealItem(item)"
            @remove="removeItem(item)"
          />
        </ul>
      </div>
    </div>
  </BaseModal>
</template>
