<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";
import { api } from "@/lib/ipc";
import type { LoaderInfo, MinecraftVersion } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";
import InstanceIconPicker from "@/components/instance/InstanceIconPicker.vue";
import { iconForLoader } from "@/lib/instanceIcons";

const emit = defineEmits<{ (e: "close"): void }>();
const instances = useInstancesStore();
const app = useAppStore();

const name = ref("");
const allVersions = ref<MinecraftVersion[]>([]);
const showSnapshots = ref(false);
const mcVersion = ref("");
const loaders = ref<LoaderInfo[]>([]);
const loadingLoaders = ref(false);
const loader = ref("vanilla");
const loaderVersion = ref("");
const icon = ref<string>(iconForLoader("vanilla"));
const ramMb = ref(app.hardware?.recommendedRamMb ?? 4096);
const busy = ref(false);
const error = ref<string | null>(null);

const visibleVersions = computed(() =>
  allVersions.value.filter((v) => showSnapshots.value || v.channel === "release"),
);
const selectedLoader = computed(() => loaders.value.find((l) => l.id === loader.value) ?? null);

onMounted(async () => {
  allVersions.value = await api.getVersions();
  mcVersion.value = allVersions.value.find((v) => v.channel === "release")?.id ?? allVersions.value[0]?.id ?? "1.20.1";
});

watch(mcVersion, async (mc) => {
  if (!mc) return;
  loader.value = "vanilla";
  loadingLoaders.value = true;
  loaders.value = [];
  try {
    loaders.value = await api.getLoaders(mc);
  } catch (e) {
    error.value = String(e);
  } finally {
    loadingLoaders.value = false;
  }
});

watch(selectedLoader, (l) => {
  loaderVersion.value = l?.recommended ?? l?.versions[0] ?? "";
});

watch(loader, (id) => {
  icon.value = iconForLoader(id);
});

async function submit() {
  if (name.value.trim().length < 1) {
    error.value = "Escribi un nombre";
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    await instances.create({
      name: name.value.trim(),
      mcVersion: mcVersion.value,
      loader: loader.value,
      loaderVersion: loader.value === "vanilla" ? "" : loaderVersion.value,
      icon: icon.value,
      ramMb: ramMb.value,
    });
    emit("close");
  } catch (err) {
    error.value = String(err);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" @click.self="emit('close')">
    <div class="w-full max-w-lg rounded-2xl border border-surface-4 bg-surface-2 p-6 shadow-2xl">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-lg font-bold">Nueva instancia</h3>
        <button class="text-gray-500 hover:text-white" @click="emit('close')">&times;</button>
      </div>

      <div class="space-y-4">
        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Nombre</span>
          <input
            v-model="name"
            type="text"
            placeholder="Mi instancia PvP"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          />
        </label>

        <div class="flex gap-3">
          <label class="flex-1">
            <span class="mb-1 flex items-center justify-between text-sm text-gray-300">
              Version de Minecraft
              <button class="text-xs text-pc-green" @click="showSnapshots = !showSnapshots">
                {{ showSnapshots ? "Solo releases" : "Ver snapshots" }}
              </button>
            </span>
            <select
              v-model="mcVersion"
              class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            >
              <option v-for="v in visibleVersions" :key="v.id" :value="v.id">
                {{ v.id }}{{ v.installed ? " ✓" : "" }}
              </option>
            </select>
          </label>
          <label class="flex-1">
            <span class="mb-1 block text-sm text-gray-300">Loader</span>
            <select
              v-model="loader"
              :disabled="loadingLoaders"
              class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            >
              <option v-for="l in loaders" :key="l.id" :value="l.id">{{ l.name }}</option>
            </select>
          </label>
        </div>

        <p v-if="loadingLoaders" class="text-xs text-gray-500">Verificando loaders compatibles…</p>

        <label v-if="selectedLoader && selectedLoader.versions.length > 0" class="block">
          <span class="mb-1 block text-sm text-gray-300">Version exacta del loader</span>
          <select
            v-model="loaderVersion"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          >
            <option v-for="ver in selectedLoader.versions" :key="ver" :value="ver">{{ ver }}</option>
          </select>
        </label>

        <div>
          <span class="mb-2 block text-sm text-gray-300">Icono (segun loader)</span>
          <InstanceIconPicker v-model="icon" />
        </div>

        <p v-if="error" class="text-xs text-red-400">{{ error }}</p>

        <div class="flex justify-end gap-2 pt-2">
          <BaseButton variant="ghost" @click="emit('close')">Cancelar</BaseButton>
          <BaseButton :disabled="busy" @click="submit">{{ busy ? "Creando..." : "Crear" }}</BaseButton>
        </div>
      </div>
    </div>
  </div>
</template>
