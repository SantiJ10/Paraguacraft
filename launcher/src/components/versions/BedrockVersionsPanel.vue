<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import BaseButton from "@/components/common/BaseButton.vue";
import { api, isTauri } from "@/lib/ipc";
import type {
  BedrockCatalogVersion,
  BedrockInstalledVersion,
  BedrockPack,
  BedrockStatus,
  BedrockWorld,
} from "@/lib/types";

const DISCLAIMER_KEY = "paraguacraft.bedrock.managedDisclaimer";

const props = defineProps<{
  status: BedrockStatus | null;
  premiumLocked: boolean;
}>();

const emit = defineEmits<{
  refreshStatus: [];
}>();

const catalog = ref<BedrockCatalogVersion[]>([]);
const installed = ref<BedrockInstalledVersion[]>([]);
const worlds = ref<BedrockWorld[]>([]);
const packs = ref<BedrockPack[]>([]);
const query = ref("");
const loading = ref(false);
const error = ref<string | null>(null);
const notice = ref<string | null>(null);
const busyVersion = ref<string | null>(null);
const setupBusy = ref(false);
const contentBusy = ref(false);
const accepted = ref(false);
const showAll = ref(false);
const showSetup = ref(true);

const canManage = computed(
  () => isTauri() && !!props.status?.platformSupported && !!props.status?.premiumAllowed,
);

const needsDisclaimer = computed(
  () => !!props.status?.conflictStore || !props.status?.developerMode,
);

function iconSrc(path: string | null | undefined): string | null {
  if (!path || !isTauri()) return null;
  try {
    return convertFileSrc(path);
  } catch {
    return null;
  }
}

function versionLabel(status: BedrockStatus | null): string {
  if (!status) return "";
  if (status.managedActive && status.activeVersion) return `${status.activeVersion} (gestionada)`;
  if (status.activeVersion) return `${status.activeVersion} (Store)`;
  if (status.installed) return "Store / Xbox";
  return "No instalado";
}

const filteredCatalog = computed(() => {
  const q = query.value.trim().toLowerCase();
  let list = catalog.value.filter((v) => {
    if (q) {
      return v.version.toLowerCase().includes(q) || v.type.toLowerCase().includes(q);
    }
    return v.type === "release";
  });
  const installedSet = new Set(installed.value.map((i) => i.version));
  list = list.filter((v) => !installedSet.has(v.version));
  if (!showAll.value && !q) return list.slice(0, 24);
  return list;
});

const hiddenCount = computed(() => {
  if (showAll.value || query.value.trim()) return 0;
  const installedSet = new Set(installed.value.map((i) => i.version));
  const n = catalog.value.filter((v) => v.type === "release" && !installedSet.has(v.version)).length;
  return Math.max(0, n - 24);
});

async function loadAll(forceCatalog = false) {
  if (!canManage.value) return;
  loading.value = true;
  error.value = null;
  try {
    const [cat, inst, w, p] = await Promise.all([
      api.listBedrockVersions(forceCatalog),
      api.listInstalledBedrockVersions(),
      api.listBedrockWorlds().catch(() => [] as BedrockWorld[]),
      api.listBedrockPacks().catch(() => [] as BedrockPack[]),
    ]);
    catalog.value = cat;
    installed.value = inst;
    worlds.value = w;
    packs.value = p;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function refreshContent() {
  try {
    worlds.value = await api.listBedrockWorlds();
    packs.value = await api.listBedrockPacks();
  } catch {
    /* carpeta todavía no existe */
  }
}

onMounted(async () => {
  accepted.value = localStorage.getItem(DISCLAIMER_KEY) === "1";
  if (canManage.value) await loadAll();
});

watch(canManage, (ok) => {
  if (ok && !catalog.value.length) void loadAll();
});

function rememberDisclaimer() {
  if (accepted.value) localStorage.setItem(DISCLAIMER_KEY, "1");
}

async function backup() {
  setupBusy.value = true;
  error.value = null;
  notice.value = null;
  try {
    const dest = await api.backupBedrockSaves();
    notice.value = `Backup de mundos en ${dest}`;
  } catch (e) {
    error.value = String(e);
  } finally {
    setupBusy.value = false;
  }
}

async function enableDev() {
  setupBusy.value = true;
  error.value = null;
  try {
    await api.enableBedrockDeveloperMode();
    emit("refreshStatus");
    notice.value = "Modo desarrollador activado.";
  } catch (e) {
    error.value = String(e);
  } finally {
    setupBusy.value = false;
  }
}

async function openStore() {
  try {
    await api.openMicrosoftStoreBedrock();
  } catch (e) {
    error.value = String(e);
  }
}

function canInstallOrSwitch(): boolean {
  if (!needsDisclaimer.value) return true;
  return accepted.value;
}

async function install(v: BedrockCatalogVersion) {
  if (!v.installable) return;
  if (!canInstallOrSwitch()) {
    error.value = "Marcá que entendés el aviso (Store + Modo desarrollador) antes de instalar.";
    showSetup.value = true;
    return;
  }
  rememberDisclaimer();
  busyVersion.value = v.version;
  error.value = null;
  notice.value = `Descargando Bedrock ${v.version}…`;
  try {
    await api.installBedrockVersion(v.version);
    notice.value = `Bedrock ${v.version} extraída. Tocá Jugar para registrarla.`;
    await loadAll();
    emit("refreshStatus");
  } catch (e) {
    error.value = String(e);
    notice.value = null;
  } finally {
    busyVersion.value = null;
  }
}

async function playInstalled(v: BedrockInstalledVersion) {
  if (!canInstallOrSwitch()) {
    error.value = "Marcá que entendés el aviso (Store + Modo desarrollador) antes de cambiar de versión.";
    showSetup.value = true;
    return;
  }
  rememberDisclaimer();
  busyVersion.value = v.version;
  error.value = null;
  notice.value = `Registrando Bedrock ${v.version}…`;
  try {
    await api.launchBedrockVersion(v.version);
    emit("refreshStatus");
    await loadAll();
  } catch (e) {
    error.value = String(e);
    notice.value = null;
  } finally {
    busyVersion.value = null;
  }
}

async function removeInstalled(v: BedrockInstalledVersion) {
  if (!confirm(`¿Borrar la versión extraída ${v.version}? Los mundos en com.mojang no se tocan.`)) {
    return;
  }
  busyVersion.value = v.version;
  try {
    await api.removeBedrockVersion(v.version);
    await loadAll();
    emit("refreshStatus");
  } catch (e) {
    error.value = String(e);
  } finally {
    busyVersion.value = null;
  }
}

async function openContent(kind: string) {
  try {
    await api.openBedrockFolder(kind);
  } catch (e) {
    error.value = String(e);
  }
}

async function importPack() {
  contentBusy.value = true;
  error.value = null;
  try {
    const name = await api.importBedrockPack();
    notice.value = `Importado: ${name}`;
    await refreshContent();
  } catch (e) {
    const msg = String(e);
    if (!msg.toLowerCase().includes("no se seleccion")) error.value = msg;
  } finally {
    contentBusy.value = false;
  }
}

async function removeWorld(w: BedrockWorld) {
  if (!confirm(`¿Borrar el mundo «${w.name}»?`)) return;
  try {
    await api.deleteBedrockWorld(w.id);
    await refreshContent();
  } catch (e) {
    error.value = String(e);
  }
}

async function removePack(p: BedrockPack) {
  if (!confirm(`¿Borrar el pack «${p.name}»?`)) return;
  try {
    await api.deleteBedrockPack(p.kind, p.id);
    await refreshContent();
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <div v-if="!premiumLocked" class="mt-4 space-y-4">
    <p class="text-xs text-gray-500">
      Versión activa:
      <span class="font-semibold text-gray-300">{{ versionLabel(status) }}</span>
    </p>

    <p v-if="status && !status.platformSupported" class="text-sm text-gray-400">
      El gestor de versiones Bedrock solo está en Windows.
    </p>

    <template v-else-if="canManage">
      <div
        v-if="needsDisclaimer && showSetup"
        class="rounded-lg border border-[#F39C12]/30 bg-[#1A1A1A] p-3 text-xs leading-relaxed text-gray-400"
      >
        <p class="font-semibold text-[#F39C12]">Antes de instalar versiones extraídas</p>
        <ul class="mt-2 list-disc space-y-1 pl-4">
          <li>Tenés que <span class="text-gray-200">poseer</span> Minecraft / Game Pass. Las AppX salen del CDN de Microsoft.</li>
          <li>Windows solo registra <span class="text-gray-200">una</span> versión a la vez. Se desregistra el Minecraft de la Store.</li>
          <li>Hace falta el <span class="text-gray-200">Modo desarrollador</span> de Windows.</li>
        </ul>
        <div class="mt-3 flex flex-col gap-2">
          <BaseButton size="sm" variant="secondary" :disabled="setupBusy" @click="backup">
            Backup de mundos
          </BaseButton>
          <BaseButton
            v-if="!status?.developerMode"
            size="sm"
            variant="secondary"
            :disabled="setupBusy"
            @click="enableDev"
          >
            Activar Modo desarrollador (UAC)
          </BaseButton>
          <label class="flex items-start gap-2 text-[11px] text-gray-400">
            <input v-model="accepted" type="checkbox" class="mt-0.5" @change="rememberDisclaimer" />
            Entiendo que se desregistra el Minecraft de la Store al jugar una versión extraída.
          </label>
        </div>
      </div>

      <div class="flex gap-2">
        <input
          v-model="query"
          type="search"
          placeholder="Buscar versión…"
          class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-[#3498DB]"
        />
      </div>

      <div>
        <p class="mb-1.5 text-[10px] font-black uppercase tracking-widest text-gray-500">Instaladas</p>
        <p v-if="!installed.length" class="text-xs text-gray-500">Ninguna versión extraída todavía.</p>
        <ul v-else class="space-y-1.5">
          <li
            v-for="v in installed"
            :key="v.version"
            class="rounded-lg border border-surface-4 bg-surface-2 px-2.5 py-2"
          >
            <div class="flex items-center justify-between gap-2">
              <div class="min-w-0">
                <p class="truncate text-sm font-semibold">{{ v.version }}</p>
                <p class="text-[10px] uppercase text-gray-500">
                  {{ v.active ? "Activa" : "En disco" }}
                </p>
              </div>
              <div class="flex shrink-0 gap-1">
                <BaseButton
                  size="sm"
                  class="!px-2 !py-1 text-[11px]"
                  :disabled="busyVersion === v.version"
                  @click="playInstalled(v)"
                >
                  Jugar
                </BaseButton>
                <BaseButton
                  size="sm"
                  variant="ghost"
                  class="!px-2 !py-1 text-[11px]"
                  :disabled="busyVersion === v.version"
                  @click="removeInstalled(v)"
                >
                  Borrar
                </BaseButton>
              </div>
            </div>
          </li>
        </ul>
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <p class="text-[10px] font-black uppercase tracking-widest text-gray-500">Catálogo</p>
          <button type="button" class="text-[10px] text-[#3498DB] hover:underline" @click="loadAll(true)">
            Actualizar
          </button>
        </div>
        <p v-if="loading && !catalog.length" class="text-xs text-gray-500">Cargando versiones…</p>
        <ul class="max-h-56 space-y-1 overflow-y-auto pr-0.5">
          <li
            v-for="v in filteredCatalog"
            :key="`${v.type}:${v.version}`"
            class="flex items-center justify-between gap-2 rounded-lg border border-surface-4 px-2.5 py-1.5"
          >
            <div class="min-w-0">
              <p class="truncate text-sm">{{ v.version }}</p>
              <p v-if="!v.installable" class="text-[10px] text-gray-500">Beta / Preview — no disponible</p>
            </div>
            <BaseButton
              v-if="v.installable"
              size="sm"
              variant="secondary"
              class="!px-2 !py-1 text-[11px]"
              :disabled="busyVersion === v.version"
              @click="install(v)"
            >
              {{ busyVersion === v.version ? "…" : "Instalar" }}
            </BaseButton>
          </li>
        </ul>
        <button
          v-if="hiddenCount > 0"
          type="button"
          class="mt-1.5 text-[11px] text-[#3498DB] hover:underline"
          @click="showAll = true"
        >
          Ver {{ hiddenCount }} más
        </button>
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <p class="text-[10px] font-black uppercase tracking-widest text-gray-500">Mundos</p>
          <button type="button" class="text-[10px] text-[#3498DB] hover:underline" @click="openContent('worlds')">
            Carpeta
          </button>
        </div>
        <p v-if="!worlds.length" class="text-xs text-gray-500">Sin mundos en com.mojang.</p>
        <ul v-else class="max-h-36 space-y-1 overflow-y-auto">
          <li v-for="w in worlds" :key="w.id" class="flex items-center gap-2 rounded-lg bg-surface-2 px-2 py-1.5">
            <img
              v-if="iconSrc(w.iconPath)"
              :src="iconSrc(w.iconPath)!"
              alt=""
              class="h-8 w-8 shrink-0 rounded object-cover"
            />
            <div v-else class="h-8 w-8 shrink-0 rounded bg-surface-4" />
            <p class="min-w-0 flex-1 truncate text-xs font-medium">{{ w.name }}</p>
            <button type="button" class="text-[10px] text-red-400 hover:underline" @click="removeWorld(w)">
              Borrar
            </button>
          </li>
        </ul>
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <p class="text-[10px] font-black uppercase tracking-widest text-gray-500">Packs</p>
          <div class="flex gap-2">
            <button type="button" class="text-[10px] text-[#3498DB] hover:underline" :disabled="contentBusy" @click="importPack">
              Importar
            </button>
            <button type="button" class="text-[10px] text-[#3498DB] hover:underline" @click="openContent('resource_packs')">
              Carpeta
            </button>
          </div>
        </div>
        <p v-if="!packs.length" class="text-xs text-gray-500">Sin resource/behavior packs.</p>
        <ul v-else class="max-h-36 space-y-1 overflow-y-auto">
          <li v-for="p in packs" :key="`${p.kind}:${p.id}`" class="flex items-center gap-2 rounded-lg bg-surface-2 px-2 py-1.5">
            <img
              v-if="iconSrc(p.iconPath)"
              :src="iconSrc(p.iconPath)!"
              alt=""
              class="h-8 w-8 shrink-0 rounded object-cover"
            />
            <div class="min-w-0 flex-1">
              <p class="truncate text-xs font-medium">{{ p.name }}</p>
              <p class="text-[10px] uppercase text-gray-500">{{ p.kind === "behavior" ? "Behavior" : "Resource" }}</p>
            </div>
            <button type="button" class="text-[10px] text-red-400 hover:underline" @click="removePack(p)">
              Borrar
            </button>
          </li>
        </ul>
      </div>

      <button type="button" class="text-[11px] text-gray-500 hover:text-[#3498DB] hover:underline" @click="openStore">
        Instalar último desde Microsoft Store
      </button>
    </template>

    <p v-if="notice" class="text-xs text-pc-green">{{ notice }}</p>
    <p v-if="error" class="text-xs text-red-400">{{ error }}</p>
    <p v-if="contentBusy" class="text-[11px] text-gray-500">Importando…</p>
  </div>
</template>
