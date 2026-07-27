<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";
import { api } from "@/lib/ipc";
import type { LoaderInfo, MinecraftVersion } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";
import AppSelect, { type AppSelectOption } from "@/components/common/AppSelect.vue";
import InstanceIconPicker from "@/components/instance/InstanceIconPicker.vue";
import { iconForLoader } from "@/lib/instanceIcons";

type LoaderChannel = "stable" | "latest" | "other";

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
const loaderChannel = ref<LoaderChannel>("stable");
const icon = ref<string>(iconForLoader("vanilla"));
const ramMb = ref(app.hardware?.recommendedRamMb ?? 4096);
const busy = ref(false);
const error = ref<string | null>(null);

const visibleVersions = computed(() =>
  allVersions.value.filter((v) => showSnapshots.value || v.channel === "release"),
);

const mcOptions = computed<AppSelectOption[]>(() =>
  visibleVersions.value.map((v) => ({
    value: v.id,
    label: v.id,
    hint: v.installed ? "instalada" : undefined,
  })),
);

const loaderOptions = computed<AppSelectOption[]>(() =>
  loaders.value.map((l) => ({ value: l.id, label: l.name })),
);

const selectedLoader = computed(() => loaders.value.find((l) => l.id === loader.value) ?? null);

const loaderVersionOptions = computed<AppSelectOption[]>(() => {
  const vers = selectedLoader.value?.versions ?? [];
  return vers.map((ver) => ({
    value: ver,
    label: ver,
    hint: ver === selectedLoader.value?.recommended ? "recomendada" : undefined,
  }));
});

function isUnstableVersion(ver: string): boolean {
  const v = ver.toLowerCase();
  return /beta|alpha|rc|snapshot|pre|dev/.test(v);
}

function pickStable(l: LoaderInfo | null): string {
  if (!l || !l.versions.length) return "";
  if (l.recommended && l.versions.includes(l.recommended)) return l.recommended;
  const stable = l.versions.find((v) => !isUnstableVersion(v));
  return stable ?? l.versions[0] ?? "";
}

function pickLatest(l: LoaderInfo | null): string {
  return l?.versions[0] ?? "";
}

function applyLoaderChannel() {
  const l = selectedLoader.value;
  if (!l || !l.versions.length) {
    loaderVersion.value = "";
    return;
  }
  if (loaderChannel.value === "stable") {
    loaderVersion.value = pickStable(l);
  } else if (loaderChannel.value === "latest") {
    loaderVersion.value = pickLatest(l);
  } else if (!loaderVersion.value || !l.versions.includes(loaderVersion.value)) {
    loaderVersion.value = pickStable(l) || pickLatest(l);
  }
}

onMounted(async () => {
  allVersions.value = await api.getVersions();
  mcVersion.value =
    allVersions.value.find((v) => v.channel === "release")?.id ?? allVersions.value[0]?.id ?? "1.20.1";
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

watch(selectedLoader, () => {
  loaderChannel.value = "stable";
  applyLoaderChannel();
});

watch(loaderChannel, () => applyLoaderChannel());

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
        <h3 class="text-lg font-bold">Crear instancia</h3>
        <button class="text-gray-500 hover:text-white" @click="emit('close')">&times;</button>
      </div>

      <div class="space-y-4">
        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Nombre</span>
          <input
            v-model="name"
            type="text"
            placeholder="Mi instancia PvP"
            class="w-full rounded-xl border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          />
        </label>

        <label class="block">
          <span class="mb-1 flex items-center justify-between text-sm text-gray-300">
            Versión del juego
            <button type="button" class="text-xs text-pc-green" @click="showSnapshots = !showSnapshots">
              {{ showSnapshots ? "Solo releases" : "Mostrar todas las versiones" }}
            </button>
          </span>
          <AppSelect v-model="mcVersion" :options="mcOptions" searchable :max-panel-height="280" />
        </label>

        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Loader</span>
          <AppSelect
            v-model="loader"
            :options="loaderOptions"
            :disabled="loadingLoaders || !loaderOptions.length"
          />
        </label>

        <p v-if="loadingLoaders" class="text-xs text-gray-500">Verificando loaders compatibles…</p>

        <div v-if="selectedLoader && selectedLoader.versions.length > 0" class="space-y-2">
          <span class="block text-sm text-gray-300">Versión del loader</span>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="ch in [
                { id: 'stable' as const, label: 'Estable' },
                { id: 'latest' as const, label: 'Más reciente' },
                { id: 'other' as const, label: 'Otro' },
              ]"
              :key="ch.id"
              type="button"
              class="inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-bold transition"
              :class="
                loaderChannel === ch.id
                  ? 'bg-pc-green text-black'
                  : 'bg-surface-3 text-gray-300 hover:bg-surface-4'
              "
              @click="loaderChannel = ch.id"
            >
              <svg
                v-if="loaderChannel === ch.id"
                class="h-3.5 w-3.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="3"
              >
                <path d="M5 13l4 4L19 7" />
              </svg>
              {{ ch.label }}
            </button>
          </div>
          <AppSelect
            v-if="loaderChannel === 'other'"
            v-model="loaderVersion"
            :options="loaderVersionOptions"
            searchable
            :max-panel-height="240"
          />
          <p v-else class="text-xs text-gray-500">
            Se usará <span class="font-mono text-gray-300">{{ loaderVersion || "—" }}</span>
          </p>
        </div>

        <div>
          <span class="mb-2 block text-sm text-gray-300">Icono</span>
          <InstanceIconPicker v-model="icon" />
        </div>

        <p v-if="error" class="text-xs text-red-400">{{ error }}</p>

        <div class="flex justify-end gap-2 pt-2">
          <BaseButton variant="ghost" @click="emit('close')">Cancelar</BaseButton>
          <BaseButton :disabled="busy" @click="submit">{{ busy ? "Creando..." : "Crear instancia" }}</BaseButton>
        </div>
      </div>
    </div>
  </div>
</template>
