<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api, isTauri } from "@/lib/ipc";
import { useSettingsStore } from "@/stores/settings";
import type { JavaInstallation } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";

const settings = useSettingsStore();
const javas = ref<JavaInstallation[]>([]);
const loading = ref(false);
const downloading = ref<number | null>(null);
const error = ref<string | null>(null);

const desktop = isTauri();
/** Majors que cubren todo el rango de MC (8, 16 vía 17, 17, 21). */
const recommended = [8, 17, 21];

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

async function download(major: number) {
  downloading.value = major;
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

function use(path: string) {
  settings.update("javaPath", path);
}

function clearGlobal() {
  settings.update("javaPath", null);
}

function isActive(path: string) {
  return settings.settings?.javaPath === path;
}

function sourceLabel(source: string) {
  if (source.startsWith("mojang")) return "Mojang";
  if (source.startsWith("paraguacraft")) return "Paraguacraft";
  if (source.startsWith("java_home")) return "JAVA_HOME";
  if (source.startsWith("path")) return "PATH";
  if (source.startsWith("system")) return "Sistema";
  return source;
}

onMounted(() => refresh(false));
</script>

<template>
  <section class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
    <div class="mb-4 flex items-center justify-between">
      <h2 class="flex items-center gap-2 text-lg font-bold">
        <span class="font-emoji">&#9749;</span> Java
      </h2>
      <BaseButton size="sm" variant="secondary" :disabled="loading || !desktop" @click="refresh(true)">
        {{ loading ? "Buscando..." : "Re-detectar" }}
      </BaseButton>
    </div>

    <p v-if="!desktop" class="text-sm text-gray-500">
      La deteccion de Java solo esta disponible en la app de escritorio.
    </p>

    <template v-else>
      <p class="mb-3 text-sm text-gray-400">
        El launcher elige el Java correcto por version: runtime Mojang oficial, Temurin descargado, o
        instalaciones del sistema. Si elegis uno manual abajo, se usa para todo (salvo override por instancia).
      </p>

      <div v-if="settings.settings?.javaPath" class="mb-3 flex items-center justify-between rounded-lg border border-pc-green/40 bg-pc-green/5 px-3 py-2 text-xs">
        <span class="truncate text-gray-300">Global activo: {{ settings.settings.javaPath }}</span>
        <button type="button" class="shrink-0 text-amber-400 hover:underline" @click="clearGlobal">
          Auto por version
        </button>
      </div>

      <div v-if="javas.length" class="mb-4 space-y-2">
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
              <span class="ml-1 text-xs font-normal text-gray-500">({{ sourceLabel(j.source) }})</span>
            </p>
            <p class="truncate text-xs text-gray-500">{{ j.path }}</p>
          </div>
          <BaseButton v-if="!isActive(j.path)" size="sm" variant="ghost" @click="use(j.path)">Usar</BaseButton>
          <span v-else class="text-xs font-bold text-pc-green">Activo</span>
        </div>
      </div>
      <p v-else-if="!loading" class="mb-4 text-sm text-gray-500">No se detecto ningun Java instalado.</p>

      <p class="mb-2 text-sm text-gray-300">Descargar Eclipse Temurin (fallback si falta runtime Mojang):</p>
      <div class="flex flex-wrap gap-2">
        <BaseButton
          v-for="major in recommended"
          :key="major"
          size="sm"
          :disabled="downloading !== null"
          @click="download(major)"
        >
          {{ downloading === major ? "Descargando..." : `Java ${major}` }}
        </BaseButton>
      </div>

      <p v-if="error" class="mt-3 text-xs text-red-400">{{ error }}</p>
    </template>
  </section>
</template>
