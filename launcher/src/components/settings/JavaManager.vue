<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { api, isTauri } from "@/lib/ipc";
import { useSettingsStore } from "@/stores/settings";
import type { JavaInstallation, MojangRuntimeInfo } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";

const props = withDefaults(
  defineProps<{ autoDetect?: boolean }>(),
  { autoDetect: false },
);

type Tab = "installed" | "mojang" | "adoptium" | "zulu";

const settings = useSettingsStore();
const javas = ref<JavaInstallation[]>([]);
const mojang = ref<MojangRuntimeInfo[]>([]);
const tab = ref<Tab>("installed");
const loading = ref(false);
const listingMojang = ref(false);
const downloading = ref<string | null>(null);
const error = ref<string | null>(null);

const desktop = isTauri();
const temurinMajors = [
  { major: 8, hint: "1.8.9 – 1.16" },
  { major: 17, hint: "1.17 – 1.20.4" },
  { major: 21, hint: "1.20.5 – 1.21" },
  { major: 25, hint: "26.x+" },
];

async function refresh(force = false) {
  loading.value = true;
  error.value = null;
  try {
    javas.value = await api.detectJavas(force);
  } catch (err) {
    error.value = String(err);
  } finally {
    loading.value = false;
  }
}

async function loadMojang() {
  if (!desktop) return;
  listingMojang.value = true;
  error.value = null;
  try {
    mojang.value = await api.listMojangRuntimes();
  } catch (err) {
    error.value = String(err);
  } finally {
    listingMojang.value = false;
  }
}

async function downloadTemurin(major: number) {
  downloading.value = `temurin-${major}`;
  error.value = null;
  try {
    const path = await api.downloadTemurin(major, false);
    settings.update("javaPath", path);
    await refresh(true);
  } catch (err) {
    error.value = String(err);
  } finally {
    downloading.value = null;
  }
}

async function downloadZulu(major: number) {
  downloading.value = `zulu-${major}`;
  error.value = null;
  try {
    const path = await api.downloadZulu(major, false);
    settings.update("javaPath", path);
    await refresh(true);
  } catch (err) {
    error.value = String(err);
  } finally {
    downloading.value = null;
  }
}

async function downloadMojang(component: string) {
  downloading.value = component;
  error.value = null;
  try {
    const path = await api.downloadMojangRuntime(component);
    settings.update("javaPath", path);
    await loadMojang();
    await refresh(true);
  } catch (err) {
    error.value = String(err);
  } finally {
    downloading.value = null;
  }
}

async function browse() {
  error.value = null;
  try {
    const j = await api.pickJavaExecutable();
    settings.update("javaPath", j.path);
    await refresh(true);
  } catch (err) {
    const msg = String(err);
    if (!/no se seleccionó/i.test(msg)) error.value = msg;
  }
}

function use(path: string) {
  settings.update("javaPath", path);
}

function clearGlobal() {
  settings.update("javaPath", null);
}

function isActive(path: string) {
  return settings.settings?.javaPath === path;
}

function sourceLabel(source: string, path: string) {
  if (path.toLowerCase().includes("zulu-jre") || source.toLowerCase().includes("zulu")) return "Zulu";
  if (source.startsWith("mojang")) return "Mojang";
  if (source.startsWith("paraguacraft")) return "Adoptium";
  if (source.startsWith("java_home")) return "JAVA_HOME";
  if (source.startsWith("path")) return "PATH";
  if (source.startsWith("system")) return "Sistema";
  if (source.startsWith("custom")) return "Manual";
  return source;
}

onMounted(() => {
  if (props.autoDetect) {
    refresh(false);
  }
});

watch(
  () => props.autoDetect,
  (on) => {
    if (on && javas.value.length === 0 && !loading.value) {
      refresh(false);
    }
  },
);

watch(tab, (t) => {
  if (t === "mojang" && mojang.value.length === 0) void loadMojang();
});
</script>

<template>
  <section class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
    <div class="mb-4 flex items-center justify-between">
      <h2 class="flex items-center gap-2 text-lg font-bold">
        <span class="font-emoji">&#9749;</span> Java
      </h2>
      <div class="flex gap-2">
        <BaseButton size="sm" variant="secondary" :disabled="!desktop" @click="browse">
          Explorar
        </BaseButton>
        <BaseButton size="sm" variant="secondary" :disabled="loading || !desktop" @click="refresh(true)">
          {{ loading ? "Buscando..." : "Re-detectar" }}
        </BaseButton>
      </div>
    </div>

    <p v-if="!desktop" class="text-sm text-gray-500">
      La deteccion de Java solo esta disponible en la app de escritorio.
    </p>

    <template v-else>
      <p class="mb-3 text-sm text-gray-400">
        Descargá Java oficial de Mojang, Eclipse Temurin (Adoptium) o Azul Zulu.
        Si no elegís uno, el launcher usa el Java correcto según la versión de Minecraft.
      </p>

      <div v-if="settings.settings?.javaPath" class="mb-3 flex items-center justify-between rounded-lg border border-pc-green/40 bg-pc-green/5 px-3 py-2 text-xs">
        <span class="truncate text-gray-300">Global activo: {{ settings.settings.javaPath }}</span>
        <button type="button" class="shrink-0 text-amber-400 hover:underline" @click="clearGlobal">
          Auto por version
        </button>
      </div>

      <div class="mb-4 flex flex-wrap gap-1 rounded-lg bg-surface-3 p-1 text-xs sm:text-sm">
        <button
          type="button"
          class="flex-1 rounded-md px-2 py-1.5 font-medium"
          :class="tab === 'installed' ? 'bg-surface-2 text-pc-green' : 'text-gray-400 hover:text-white'"
          @click="tab = 'installed'"
        >
          Instalado
        </button>
        <button
          type="button"
          class="flex-1 rounded-md px-2 py-1.5 font-medium"
          :class="tab === 'mojang' ? 'bg-surface-2 text-pc-green' : 'text-gray-400 hover:text-white'"
          @click="tab = 'mojang'"
        >
          Mojang
        </button>
        <button
          type="button"
          class="flex-1 rounded-md px-2 py-1.5 font-medium"
          :class="tab === 'adoptium' ? 'bg-surface-2 text-pc-green' : 'text-gray-400 hover:text-white'"
          @click="tab = 'adoptium'"
        >
          Adoptium
        </button>
        <button
          type="button"
          class="flex-1 rounded-md px-2 py-1.5 font-medium"
          :class="tab === 'zulu' ? 'bg-surface-2 text-pc-green' : 'text-gray-400 hover:text-white'"
          @click="tab = 'zulu'"
        >
          Zulu
        </button>
      </div>

      <div v-if="tab === 'installed'">
        <div v-if="javas.length" class="mb-2 space-y-2">
          <div
            v-for="j in javas"
            :key="j.path"
            class="flex items-center gap-3 rounded-lg border p-3"
            :class="isActive(j.path) ? 'border-pc-green bg-pc-green/5' : 'border-surface-4'"
          >
            <span class="flex h-8 w-8 items-center justify-center rounded-md bg-surface-5 text-sm font-bold">
              {{ j.versionMajor }}
            </span>
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-semibold">
                Java {{ j.versionFull }}
                <span class="ml-1 text-xs font-normal text-gray-500">({{ sourceLabel(j.source, j.path) }})</span>
              </p>
              <p class="truncate text-xs text-gray-500">{{ j.path }}</p>
            </div>
            <BaseButton v-if="!isActive(j.path)" size="sm" variant="ghost" @click="use(j.path)">Usar</BaseButton>
            <span v-else class="text-xs font-bold text-pc-green">Activo</span>
          </div>
        </div>
        <p v-else-if="!loading" class="text-sm text-gray-500">No se detecto ningun Java instalado.</p>
      </div>

      <div v-else-if="tab === 'mojang'">
        <p class="mb-3 text-xs text-gray-500">
          El mismo Java que usa el launcher oficial. Recomendado si querés máxima compatibilidad.
        </p>
        <p v-if="listingMojang" class="text-sm text-gray-500">Cargando catálogo Mojang…</p>
        <div v-else class="space-y-2">
          <div
            v-for="r in mojang"
            :key="r.component"
            class="flex items-center gap-3 rounded-lg border border-surface-4 p-3"
          >
            <div class="min-w-0 flex-1">
              <p class="text-sm font-semibold">
                {{ r.name }}
                <span v-if="r.installed" class="ml-2 text-xs font-normal text-pc-green">instalado</span>
              </p>
              <p class="truncate text-xs text-gray-500">{{ r.version || r.component }}</p>
            </div>
            <BaseButton
              size="sm"
              :disabled="downloading !== null"
              @click="downloadMojang(r.component)"
            >
              {{ downloading === r.component ? "Descargando..." : r.installed ? "Reusar" : "Descargar" }}
            </BaseButton>
          </div>
          <p v-if="!mojang.length" class="text-sm text-gray-500">
            No se pudo listar runtimes Mojang. Probá de nuevo con internet.
          </p>
        </div>
      </div>

      <div v-else-if="tab === 'adoptium'">
        <p class="mb-3 text-xs text-gray-500">
          Eclipse Temurin (Adoptium): JRE HotSpot, suele ir un poco más liviano que el runtime de Mojang.
        </p>
        <div class="grid gap-2 sm:grid-cols-2">
          <button
            v-for="item in temurinMajors"
            :key="item.major"
            type="button"
            class="rounded-lg border border-surface-4 bg-surface-3 px-3 py-3 text-left hover:border-pc-green disabled:opacity-50"
            :disabled="downloading !== null"
            @click="downloadTemurin(item.major)"
          >
            <span class="block text-sm font-bold">
              {{ downloading === `temurin-${item.major}` ? "Descargando..." : `Java ${item.major}` }}
            </span>
            <span class="text-xs text-gray-500">{{ item.hint }}</span>
          </button>
        </div>
      </div>

      <div v-else>
        <p class="mb-3 text-xs text-gray-500">
          Azul Zulu (Community): otro OpenJDK bien optimizado para Minecraft. Convivé con Temurin, no lo pisa.
        </p>
        <div class="grid gap-2 sm:grid-cols-2">
          <button
            v-for="item in temurinMajors"
            :key="item.major"
            type="button"
            class="rounded-lg border border-surface-4 bg-surface-3 px-3 py-3 text-left hover:border-pc-green disabled:opacity-50"
            :disabled="downloading !== null"
            @click="downloadZulu(item.major)"
          >
            <span class="block text-sm font-bold">
              {{ downloading === `zulu-${item.major}` ? "Descargando..." : `Java ${item.major}` }}
            </span>
            <span class="text-xs text-gray-500">{{ item.hint }}</span>
          </button>
        </div>
      </div>

      <p v-if="error" class="mt-3 text-xs text-red-400">{{ error }}</p>
    </template>
  </section>
</template>
