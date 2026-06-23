<script setup lang="ts">
import { ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import InstanceIcon from "@/components/instance/InstanceIcon.vue";
import {
  DEFAULT_INSTANCE_ICON,
  INSTANCE_ICONS,
  INSTANCE_ICON_MIN_SIZE,
  INSTANCE_ICON_SIZE,
  isCustomInstanceIcon,
  type McInstanceIconId,
} from "@/lib/instanceIcons";
import { LOADER_ICONS } from "@/lib/loaderIcons";
import { api, isTauri } from "@/lib/ipc";

const model = defineModel<string>({ default: DEFAULT_INSTANCE_ICON });

const importBusy = ref(false);
const importError = ref<string | null>(null);
const customPreview = ref<string | null>(null);

async function refreshCustomPreview(icon: string) {
  customPreview.value = null;
  if (!isTauri() || !isCustomInstanceIcon(icon)) return;
  try {
    const path = await api.getInstanceIconPath(icon);
    if (path) customPreview.value = convertFileSrc(path);
  } catch {
    customPreview.value = null;
  }
}

async function selectBuiltIn(id: McInstanceIconId | string) {
  model.value = id;
  importError.value = null;
  customPreview.value = null;
}

async function importCustom() {
  if (!isTauri()) return;
  importBusy.value = true;
  importError.value = null;
  try {
    const result = await api.pickAndImportInstanceIcon();
    model.value = result.iconId;
    customPreview.value = convertFileSrc(result.path);
  } catch (e) {
    importError.value = String(e);
  } finally {
    importBusy.value = false;
  }
}

defineExpose({ refreshCustomPreview });

watch(model, (v) => {
  if (isCustomInstanceIcon(v)) void refreshCustomPreview(v);
}, { immediate: true });
</script>

<template>
  <div>
    <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-gray-500">Loaders</p>
    <div class="mb-3 grid grid-cols-4 gap-2 sm:grid-cols-8">
      <button
        v-for="ic in LOADER_ICONS"
        :key="ic.id"
        type="button"
        class="flex items-center justify-center rounded-lg border p-1.5 transition"
        :class="model === `loader:${ic.id}` ? 'border-pc-green bg-pc-green/10' : 'border-surface-4 hover:border-surface-5'"
        :title="ic.name"
        @click="selectBuiltIn(`loader:${ic.id}`)"
      >
        <img :src="ic.src" :alt="ic.name" class="h-10 w-10 object-contain" draggable="false" />
      </button>
    </div>

    <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-gray-500">Minecraft</p>
    <div class="grid grid-cols-5 gap-2 sm:grid-cols-6">
      <button
        v-for="ic in INSTANCE_ICONS"
        :key="ic.id"
        type="button"
        class="flex items-center justify-center rounded-lg border p-1.5 transition"
        :class="model === ic.id ? 'border-pc-green bg-pc-green/10' : 'border-surface-4 hover:border-surface-5'"
        :title="ic.label"
        @click="selectBuiltIn(ic.id)"
      >
        <img :src="ic.src" :alt="ic.label" class="h-10 w-10 object-contain" draggable="false" />
      </button>

      <button
        v-if="isTauri()"
        type="button"
        class="flex flex-col items-center justify-center rounded-lg border border-dashed p-1.5 transition"
        :class="isCustomInstanceIcon(model) ? 'border-pc-green bg-pc-green/10' : 'border-surface-4 hover:border-surface-5'"
        :disabled="importBusy"
        title="Importar imagen"
        @click="importCustom"
      >
        <img
          v-if="customPreview && isCustomInstanceIcon(model)"
          :src="customPreview"
          alt="Icono personalizado"
          class="h-10 w-10 object-contain"
        />
        <span v-else class="text-2xl leading-none text-gray-400">+</span>
      </button>
    </div>

    <p class="mt-2 text-xs text-gray-500">
      PNG/JPG/WebP, mínimo {{ INSTANCE_ICON_MIN_SIZE }}×{{ INSTANCE_ICON_MIN_SIZE }} px.
      Se ajusta a {{ INSTANCE_ICON_SIZE }}×{{ INSTANCE_ICON_SIZE }}.
      <span v-if="importBusy" class="text-pc-green"> Importando…</span>
    </p>
    <p v-if="importError" class="mt-1 text-xs text-red-400">{{ importError }}</p>
    <div class="mt-2 flex items-center gap-2 text-xs text-gray-500">
      <span>Vista previa:</span>
      <InstanceIcon :icon="model" size="sm" />
    </div>
  </div>
</template>
