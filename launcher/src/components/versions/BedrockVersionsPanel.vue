<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import BaseButton from "@/components/common/BaseButton.vue";
import { api, isTauri } from "@/lib/ipc";
import type {
  BedrockCatalogVersion,
  BedrockImport,
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
const packUrl = ref("");
const loading = ref(false);
const error = ref<string | null>(null);
const notice = ref<string | null>(null);
const busyVersion = ref<string | null>(null);
const setupBusy = ref(false);
const contentBusy = ref(false);
const accepted = ref(false);
const showAll = ref(false);
const showOld = ref(false);
const showSetup = ref(true);
const versionMode = ref<"latest" | "older">("latest");
const quickTargetVersion = ref("");

const canManage = computed(
  () => isTauri() && !!props.status?.platformSupported && !!props.status?.premiumAllowed,
);

const latestCatalogRelease = computed((): BedrockCatalogVersion | null => {
  const releases = catalog.value
    .filter((v) => catalogKind(v) === "release" && v.installable)
    .sort((a, b) => cmpVerDesc(a.version, b.version));
  return releases[0] ?? null;
});

const isLatestActive = computed(() => {
  if (!props.status) return false;
  if (props.status.storeInstalled && !props.status.managedActive) return true;
  if (
    props.status.managedActive &&
    props.status.activeVersion &&
    latestCatalogRelease.value &&
    props.status.activeVersion === latestCatalogRelease.value.version
  ) {
    return true;
  }
  return false;
});

const selectableOlderVersions = computed(() => {
  const releases = catalog.value
    .filter((v) => catalogKind(v) === "release" && v.installable)
    .sort((a, b) => cmpVerDesc(a.version, b.version));
  return showOld.value ? releases : releases.filter((v) => isRecentBedrock(v.version));
});

const isQuickTargetInstalled = computed(() =>
  installed.value.some((i) => i.version === quickTargetVersion.value),
);

const needsDisclaimer = computed(
  () =>
    !props.status?.developerMode
    || !!props.status?.conflictStore
    || !!props.status?.storeInstalled
    || (!!props.status?.hasSaves && !props.status?.managedActive),
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
  if (status.activeVersion) return `${status.activeVersion} (Store / Última)`;
  if (status.installed) return "Store / Xbox (Última)";
  return "No instalado";
}

function versionParts(v: string): number[] {
  return v.split(".").map((p) => Number.parseInt(p, 10) || 0);
}

function cmpVerDesc(a: string, b: string): number {
  const pa = versionParts(a);
  const pb = versionParts(b);
  const n = Math.max(pa.length, pb.length);
  for (let i = 0; i < n; i++) {
    const d = (pb[i] ?? 0) - (pa[i] ?? 0);
    if (d) return d;
  }
  return 0;
}

function isRecentBedrock(version: string): boolean {
  const p = versionParts(version);
  return (p[0] ?? 0) > 1 || ((p[0] ?? 0) === 1 && (p[1] ?? 0) >= 16);
}

function catalogKind(v: BedrockCatalogVersion): string {
  return (v.type ?? (v as { kind?: string }).kind ?? "").toLowerCase();
}

function catalogUnavailable(v: BedrockCatalogVersion): string {
  if (catalogKind(v) !== "release") return "Beta / Preview — programa Insider";
  const major = versionParts(v.version)[0] ?? 0;
  if (major === 0) return "Microsoft ya no publica esta AppX (0.x)";
  return "No disponible";
}

const filteredCatalog = computed(() => {
  const q = query.value.trim().toLowerCase();
  const installedSet = new Set(installed.value.map((i) => i.version));
  let list = catalog.value.filter((v) => !installedSet.has(v.version));
  if (q) {
    list = list.filter(
      (v) => v.version.toLowerCase().includes(q) || catalogKind(v).includes(q),
    );
  } else {
    list = list.filter((v) => catalogKind(v) === "release");
    if (!showOld.value) list = list.filter((v) => isRecentBedrock(v.version));
  }
  list = [...list].sort((a, b) => cmpVerDesc(a.version, b.version));
  if (!showAll.value && !q) return list.slice(0, 20);
  return list;
});

const hiddenCount = computed(() => {
  if (showAll.value || query.value.trim()) return 0;
  const installedSet = new Set(installed.value.map((i) => i.version));
  const n = catalog.value.filter((v) => {
    if (installedSet.has(v.version) || catalogKind(v) !== "release") return false;
    return showOld.value || isRecentBedrock(v.version);
  }).length;
  return Math.max(0, n - 20);
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
    if (!quickTargetVersion.value) {
      const activeManaged = inst.find((i) => i.active);
      if (activeManaged) {
        quickTargetVersion.value = activeManaged.version;
        versionMode.value =
          latestCatalogRelease.value && activeManaged.version === latestCatalogRelease.value.version
            ? "latest"
            : "older";
      } else {
        const secondRelease = selectableOlderVersions.value[1] ?? selectableOlderVersions.value[0];
        if (secondRelease) quickTargetVersion.value = secondRelease.version;
      }
    }
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
    emit("refreshStatus");
  } catch (e) {
    error.value = String(e);
  }
}

async function restoreStoreVersion() {
  setupBusy.value = true;
  error.value = null;
  notice.value = "Restaurando versión oficial de Microsoft Store / Xbox App (tus mundos en com.mojang se conservan)…";
  try {
    const restored = await api.restoreStoreBedrock();
    await loadAll();
    emit("refreshStatus");
    if (restored) {
      notice.value = "Última versión oficial (Store / Xbox App) restaurada y lista para jugar.";
    } else {
      notice.value = "Registro limpio (mundos intactos). Abriendo Microsoft Store para activar la última versión…";
      await api.openMicrosoftStoreBedrock();
    }
  } catch (e) {
    error.value = String(e);
    notice.value = null;
  } finally {
    setupBusy.value = false;
  }
}

async function playLatestVersion() {
  setupBusy.value = true;
  error.value = null;
  notice.value = null;
  try {
    // 1) Si ya está activa la Store o la última versión, abrimos directo.
    if (props.status?.storeInstalled && !props.status?.managedActive) {
      notice.value = "Abriendo última versión oficial (Microsoft Store / Xbox)…";
      await api.launchBedrock();
      return;
    }
    // 2) Intentar restaurar el paquete oficial de Store/XboxGames si está en disco.
    const restored = await api.restoreStoreBedrock().catch(() => false);
    if (restored) {
      emit("refreshStatus");
      await loadAll();
      notice.value = "Abriendo última versión oficial (Microsoft Store / Xbox)…";
      await api.launchBedrock();
      return;
    }
    // 3) Si no está la Store en disco, usar o descargar la última release del catálogo FE3 oficial.
    const latest = latestCatalogRelease.value;
    if (!latest) {
      await api.openMicrosoftStoreBedrock();
      return;
    }
    const already = installed.value.find((i) => i.version === latest.version);
    if (!already) {
      notice.value = `Descargando última versión oficial (${latest.version}) desde Microsoft…`;
      await api.installBedrockVersion(latest.version);
    }
    notice.value = `Lanzando Bedrock ${latest.version}…`;
    await api.launchBedrockVersion(latest.version);
    emit("refreshStatus");
    await loadAll();
  } catch (e) {
    error.value = String(e);
    notice.value = null;
  } finally {
    setupBusy.value = false;
  }
}

async function playSelectedOlderVersion() {
  const ver = quickTargetVersion.value.trim();
  if (!ver) return;
  if (!canInstallOrSwitch()) {
    error.value = "Marcá que entendés el aviso (Store + Modo desarrollador) antes de cambiar de versión.";
    showSetup.value = true;
    return;
  }
  rememberDisclaimer();
  busyVersion.value = ver;
  error.value = null;
  try {
    const inst = installed.value.find((i) => i.version === ver);
    if (!inst) {
      notice.value = `Descargando Bedrock ${ver} desde Microsoft CDN…`;
      await api.installBedrockVersion(ver);
    }
    notice.value = `Registrando y abriendo Bedrock ${ver} (backup automático de com.mojang)…`;
    await api.launchBedrockVersion(ver);
    emit("refreshStatus");
    await loadAll();
  } catch (e) {
    error.value = String(e);
    notice.value = null;
  } finally {
    busyVersion.value = null;
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

function importSummary(res: BedrockImport): string {
  if (res.kind === "world") return `Mundo importado: ${res.name}`;
  const what = res.packs > 1 ? `${res.packs} packs` : `Pack ${res.name}`;
  const extras: string[] = [];
  if (res.activated) extras.push("activado");
  if (res.downgraded) extras.push("ajustado a tu versión de Bedrock");
  return extras.length ? `${what} — ${extras.join(" y ")}.` : `${what} instalado.`;
}

async function importPack() {
  contentBusy.value = true;
  error.value = null;
  try {
    const res = await api.importBedrockPack();
    notice.value = importSummary(res);
    await refreshContent();
  } catch (e) {
    const msg = String(e);
    if (!msg.toLowerCase().includes("no se seleccion")) error.value = msg;
  } finally {
    contentBusy.value = false;
  }
}

async function installPackUrl() {
  const url = packUrl.value.trim();
  if (!url) return;
  contentBusy.value = true;
  error.value = null;
  notice.value = "Descargando pack…";
  try {
    const res = await api.installBedrockPackUrl(url);
    notice.value = importSummary(res);
    packUrl.value = "";
    await refreshContent();
  } catch (e) {
    error.value = String(e);
    notice.value = null;
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
    <div class="rounded-xl border border-pc-green/30 bg-pc-green/5 p-3">
      <div class="flex items-center justify-between gap-2">
        <span class="text-[11px] font-bold uppercase tracking-wider text-pc-green">
          100% Seguro · Sin riesgo de baneo
        </span>
        <span class="rounded bg-surface-3 px-1.5 py-0.5 text-[10px] font-semibold text-gray-300">
          {{ versionLabel(status) }}
        </span>
      </div>
      <p class="mt-1 text-[11px] leading-relaxed text-gray-400">
        Paquetes oficiales del servidor de Microsoft Update (FE3) sin inyección de DLLs ni modificaciones de memoria. Podés jugar en servidores oficiales, Realms y con amigos con total seguridad.
      </p>
    </div>

    <p v-if="status && !status.platformSupported" class="text-sm text-gray-400">
      El gestor de versiones Bedrock solo está en Windows.
    </p>

    <template v-else-if="canManage">
      <!-- Selector principal: Última versión vs Versiones anteriores -->
      <div class="rounded-xl border border-surface-4 bg-surface-2 p-3 space-y-3">
        <p class="text-[10px] font-black uppercase tracking-widest text-gray-400">
          Elegí qué versión de Paragua Bedrock jugar
        </p>
        <div class="grid grid-cols-2 gap-1.5 rounded-lg bg-surface-1 p-1">
          <button
            type="button"
            class="rounded-md px-2.5 py-1.5 text-xs font-semibold transition"
            :class="versionMode === 'latest' ? 'bg-[#0078D4] text-white' : 'text-gray-400 hover:text-white'"
            @click="versionMode = 'latest'"
          >
            Última versión
          </button>
          <button
            type="button"
            class="rounded-md px-2.5 py-1.5 text-xs font-semibold transition"
            :class="versionMode === 'older' ? 'bg-pc-green text-black' : 'text-gray-400 hover:text-white'"
            @click="versionMode = 'older'"
          >
            Versiones viejas
          </button>
        </div>

        <!-- Modo 1: Última versión -->
        <div v-if="versionMode === 'latest'" class="space-y-2 pt-1">
          <p class="text-xs text-gray-300">
            Jugá la última versión oficial de Bedrock
            <span v-if="latestCatalogRelease" class="font-semibold text-white">({{ latestCatalogRelease.version }})</span>
            para entrar a Realms y servidores actualizados.
          </p>
          <BaseButton
            size="sm"
            class="w-full !bg-[#0078D4] hover:!bg-[#106EBE]"
            :disabled="setupBusy || !!busyVersion"
            @click="playLatestVersion"
          >
            {{
              setupBusy
                ? "Preparando última versión…"
                : isLatestActive
                  ? "Jugar última versión ahora"
                  : "Activar y jugar última versión"
            }}
          </BaseButton>
          <BaseButton
            v-if="status?.managedActive"
            size="sm"
            variant="secondary"
            class="w-full"
            :disabled="setupBusy || !!busyVersion"
            @click="restoreStoreVersion"
          >
            Restaurar Microsoft Store / Xbox App original
          </BaseButton>
        </div>

        <!-- Modo 2: Versiones viejas -->
        <div v-else class="space-y-2 pt-1">
          <p class="text-xs text-gray-300">
            Elegí cualquier versión anterior (1.21, 1.20, 1.19, 1.18, 1.17, 1.16…). El launcher hace backup automático de tus mundos antes de cambiar.
          </p>
          <select
            v-model="quickTargetVersion"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-2.5 py-2 text-xs text-white outline-none focus:border-pc-green"
          >
            <option value="" disabled>Seleccioná una versión…</option>
            <option
              v-for="v in selectableOlderVersions"
              :key="v.version"
              :value="v.version"
            >
              Bedrock {{ v.version }}
              {{ installed.some((i) => i.version === v.version) ? " (Instalada)" : "" }}
              {{ status?.activeVersion === v.version && status?.managedActive ? " — Activa" : "" }}
            </option>
          </select>
          <BaseButton
            size="sm"
            class="w-full"
            :disabled="!quickTargetVersion || setupBusy || !!busyVersion"
            @click="playSelectedOlderVersion"
          >
            {{
              busyVersion === quickTargetVersion && quickTargetVersion
                ? "Cambiando de versión…"
                : isQuickTargetInstalled
                  ? `Cambiar y jugar v${quickTargetVersion}`
                  : `Descargar y jugar v${quickTargetVersion}`
            }}
          </BaseButton>
        </div>
      </div>

      <div
        v-if="needsDisclaimer && showSetup"
        class="rounded-lg border border-[#F39C12]/30 bg-[#1A1A1A] p-3 text-xs leading-relaxed text-gray-400"
      >
        <p class="font-semibold text-[#F39C12]">Antes de instalar versiones extraídas</p>
        <p v-if="status?.storeInstalled || status?.hasSaves" class="mt-1 text-gray-300">
          Se detectó
          <span v-if="status?.storeInstalled">Minecraft de la Store</span>
          <span v-if="status?.storeInstalled && status?.hasSaves"> y </span>
          <span v-if="status?.hasSaves">mundos en com.mojang</span>.
          Hacé Backup acá; no lo borres desde Configuración de Windows.
        </p>
        <ul class="mt-2 list-disc space-y-1 pl-4">
          <li>Tenés que <span class="text-gray-200">poseer</span> Minecraft / Game Pass. Las AppX salen del CDN de Microsoft.</li>
          <li>Windows solo registra <span class="text-gray-200">una</span> versión a la vez. Podés volver a la última versión con 1 clic cuando quieras.</li>
          <li>Hace falta el <span class="text-gray-200">Modo desarrollador</span> de Windows.</li>
          <li>
            <span class="text-[#F39C12]">No lo desinstales desde Configuración de Windows</span>:
            eso borra los mundos. El launcher cambia de versión preservando com.mojang.
          </li>
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
            Entiendo que se alterna el registro de la Store al jugar una versión extraída.
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
          <div class="flex gap-2">
            <button type="button" class="text-[10px] text-gray-500 hover:underline" @click="showOld = !showOld">
              {{ showOld ? "Solo 1.16+" : "Incluir antiguas" }}
            </button>
            <button type="button" class="text-[10px] text-[#3498DB] hover:underline" @click="loadAll(true)">
              Actualizar
            </button>
          </div>
        </div>
        <p v-if="loading && !catalog.length" class="text-xs text-gray-500">Cargando versiones…</p>
        <ul class="max-h-56 space-y-1 overflow-y-auto pr-0.5">
          <li
            v-for="v in filteredCatalog"
            :key="`${catalogKind(v)}:${v.version}`"
            class="flex items-center justify-between gap-2 rounded-lg border border-surface-4 px-2.5 py-1.5"
          >
            <div class="min-w-0">
              <p class="truncate text-sm">{{ v.version }}</p>
              <p v-if="!v.installable" class="text-[10px] text-gray-500">{{ catalogUnavailable(v) }}</p>
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
          <div class="flex gap-2">
            <button type="button" class="text-[10px] text-[#3498DB] hover:underline" :disabled="setupBusy" @click="backup">
              Backup
            </button>
            <button type="button" class="text-[10px] text-[#3498DB] hover:underline" @click="openContent('worlds')">
              Carpeta
            </button>
          </div>
        </div>
        <p v-if="!worlds.length" class="text-xs text-gray-500">
          Sin mundos en com.mojang. Si los tenías y desinstalaste Minecraft desde Configuración de Windows, esa carpeta se borra.
        </p>
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
        <form class="mb-2 flex gap-1.5" @submit.prevent="installPackUrl">
          <input
            v-model="packUrl"
            type="url"
            inputmode="url"
            placeholder="Pegá el link de un .mcpack o .mcaddon"
            class="min-w-0 flex-1 rounded-lg border border-surface-5 bg-surface-3 px-2.5 py-1.5 text-xs outline-none focus:border-[#3498DB]"
          />
          <BaseButton
            size="sm"
            variant="secondary"
            class="!px-2 !py-1 text-[11px]"
            :disabled="contentBusy || !packUrl.trim()"
            @click="installPackUrl"
          >
            Bajar
          </BaseButton>
        </form>
        <p class="mb-2 text-[10px] leading-relaxed text-gray-500">
          Se instala y se activa solo. Si el pack pide una Bedrock más nueva que la tuya,
          se le baja el mínimo para que igual cargue.
        </p>
        <p v-if="!packs.length" class="text-xs text-gray-500">Sin resource/behavior packs.</p>
        <ul v-else class="max-h-36 space-y-1 overflow-y-auto">
          <li v-for="p in packs" :key="`${p.kind}:${p.id}`" class="flex items-center gap-2 rounded-lg bg-surface-2 px-2 py-1.5">
            <img
              v-if="iconSrc(p.iconPath)"
              :src="iconSrc(p.iconPath)!"
              alt=""
              loading="lazy"
              decoding="async"
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
