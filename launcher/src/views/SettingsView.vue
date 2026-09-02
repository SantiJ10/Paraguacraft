<script setup lang="ts">
defineOptions({ name: "settings" });
import { computed, onMounted, ref } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useAccountsStore } from "@/stores/accounts";
import { useSkinsStore } from "@/stores/skins";
import { useAppStore } from "@/stores/app";
import { useInstancesStore } from "@/stores/instances";
import BaseToggle from "@/components/common/BaseToggle.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import AddAccountModal from "@/components/account/AddAccountModal.vue";
import SkinAvatar from "@/components/account/SkinAvatar.vue";
import JavaManager from "@/components/settings/JavaManager.vue";
import { formatRam } from "@/composables/useFormat";
import { applyAccentTheme } from "@/composables/useAccent";
import { api, isTauri } from "@/lib/ipc";
import { normalizeLoaderId } from "@/lib/loaders";
import { useI18n } from "@/composables/useI18n";
import type { CleanupInfo, ExtrasStatus, GcType, PvpClientStatus, AppSettings } from "@/lib/types";

const settings = useSettingsStore();
const accounts = useAccountsStore();
const skins = useSkinsStore();
const app = useAppStore();
const instancesStore = useInstancesStore();
const i18n = useI18n();

const pageReady = ref(false);
const extrasLoading = ref(false);
const javaSectionReady = ref(false);

const showAddAccount = ref(false);
const skinBusy = ref(false);
const skinMessage = ref<string | null>(null);

const copyArgsBusy = ref(false);
const copyArgsMessage = ref<string | null>(null);

async function copyLastLaunchArgs() {
  copyArgsBusy.value = true;
  copyArgsMessage.value = null;
  try {
    const { args, instanceId } = await api.getLastLaunchArgs();
    await navigator.clipboard.writeText(args);
    copyArgsMessage.value = `Args copiados (${instanceId}).`;
  } catch (e) {
    copyArgsMessage.value = String(e).replace(/^Error:\s*/i, "");
  } finally {
    copyArgsBusy.value = false;
  }
}

/** Renombrar cuenta offline (inline). */
const renamingId = ref<string | null>(null);
const renameDraft = ref("");
const renameBusy = ref(false);
const renameError = ref<string | null>(null);

function startRename(acc: { id: string; username: string; premium: boolean; type?: string }) {
  if (acc.premium) return;
  renamingId.value = acc.id;
  renameDraft.value = acc.username;
  renameError.value = null;
}

function cancelRename() {
  renamingId.value = null;
  renameDraft.value = "";
  renameError.value = null;
  renameBusy.value = false;
}

async function confirmRename() {
  const id = renamingId.value;
  if (!id) return;
  const name = renameDraft.value.trim();
  if (name.length < 3) {
    renameError.value = "Mínimo 3 caracteres";
    return;
  }
  if (name.length > 16) {
    renameError.value = "Máximo 16 caracteres";
    return;
  }
  if (!/^[A-Za-z0-9_]+$/.test(name)) {
    renameError.value = "Solo letras, números y _";
    return;
  }
  renameBusy.value = true;
  renameError.value = null;
  try {
    await accounts.renameOffline(id, name);
    cancelRename();
    void skins.refresh(true);
  } catch (e) {
    renameError.value = String(e).replace(/^Error:\s*/i, "");
  } finally {
    renameBusy.value = false;
  }
}

const extrasStatus = ref<ExtrasStatus | null>(null);
const cleanupInfo = ref<CleanupInfo | null>(null);
const extrasBusy = ref(false);
const extrasMessage = ref<string | null>(null);

const groqKey = ref("");
const groqKeyMessage = ref<string | null>(null);
const aiConfigured = ref<boolean | null>(null);
const keysManaged = ref(false);

async function loadAiStatus() {
  if (!isTauri()) return;
  try {
    const [st, managed] = await Promise.all([api.aiStatus(), api.apiKeysManaged()]);
    aiConfigured.value = st.configured;
    keysManaged.value = managed;
  } catch {
    aiConfigured.value = false;
    keysManaged.value = false;
  }
}

async function saveGroqKey() {
  if (!isTauri()) return;
  groqKeyMessage.value = null;
  try {
    await api.saveGroqApiKey(groqKey.value.trim());
    groqKeyMessage.value = "API key guardada. Reinicia Paraguabot si ya estaba abierto.";
    groqKey.value = "";
    await loadAiStatus();
  } catch (e) {
    groqKeyMessage.value = String(e);
  }
}

onMounted(async () => {
  // Si settings ya cargó en App.vue → sin skeleton ni await.
  if (!settings.settings) {
    await settings.load();
  }
  pageReady.value = true;

  if (!accounts.loaded) {
    void accounts.load();
  }
  if (!app.hardware) {
    // Nunca en hot path: hardware hace PowerShell.
    window.setTimeout(() => void app.loadHardware(), 400);
  }
  if (!skins.activeSkin) {
    window.setTimeout(() => void skins.refresh(), 800);
  }

  if (isTauri()) {
    void loadAiStatus();
    const idle = window.requestIdleCallback ?? ((cb: () => void) => window.setTimeout(cb, 800));
    idle(() => {
      javaSectionReady.value = true;
    });
  }
});

async function applyOfflineSkin() {
  if (!isTauri()) return;
  skinBusy.value = true;
  skinMessage.value = null;
  try {
    const result = await api.pickAndApplyOfflineSkin();
    skinMessage.value = result.message;
    await skins.refresh();
  } catch (e) {
    skinMessage.value = String(e);
  } finally {
    skinBusy.value = false;
  }
}

const checkingUpdate = ref(false);
const pvpStatus = ref<PvpClientStatus | null>(null);
const pvpStatusLoading = ref(false);
const pvpSyncBusy = ref(false);
const pvpStatusMessage = ref<string | null>(null);
const pvpInstanceId = ref<string | null>(null);
const pvpModernStatus = ref<PvpClientStatus | null>(null);
const pvpModernStatusLoading = ref(false);
const pvpModernSyncBusy = ref(false);
const pvpModernStatusMessage = ref<string | null>(null);
const pvpModernInstanceId = ref<string | null>(null);
const perfBusy = ref(false);
const perfMessage = ref<string | null>(null);

async function applyHardwareRecommended() {
  perfBusy.value = true;
  perfMessage.value = null;
  try {
    const hw = await api.applyRecommendedPerformance();
    await settings.load(true);
    app.hardware = hw;
    perfMessage.value = `Perfil ${hw.perfilSugerido}: ${formatRam(hw.recommendedRamMb)} RAM, ${hw.recommendedGc}.`;
  } catch (e) {
    perfMessage.value = String(e);
  } finally {
    perfBusy.value = false;
  }
}

async function optimizeMinecraftOptions() {
  perfBusy.value = true;
  perfMessage.value = null;
  try {
    const r = await api.optimizeMinecraftOptions();
    const keys = Object.entries(r.applied)
      .map(([k, v]) => `${k}: ${v}`)
      .join(", ");
    perfMessage.value = `options.txt optimizado (${r.tier}): ${keys}`;
  } catch (e) {
    perfMessage.value = String(e);
  } finally {
    perfBusy.value = false;
  }
}

function setAccent(accent: "green" | "ai") {
  settings.update("accent", accent);
  applyAccentTheme(accent);
}

async function refreshUpdate() {
  checkingUpdate.value = true;
  try {
    await app.checkUpdate(true);
  } finally {
    checkingUpdate.value = false;
  }
}

async function refreshPvpClientStatus() {
  if (!isTauri()) return;
  pvpStatusLoading.value = true;
  pvpStatusMessage.value = null;
  try {
    if (!instancesStore.loaded) await instancesStore.load();
    const pvp = instancesStore.instances.find(
      (i) => normalizeLoaderId(i.loader) === "paraguacraft-pvp",
    );
    pvpInstanceId.value = pvp?.id ?? null;
    pvpStatus.value = await api.getPvpClientStatus(pvp?.id ?? null);
  } catch (e) {
    pvpStatusMessage.value = String(e);
  } finally {
    pvpStatusLoading.value = false;
  }
}

async function syncPvpClientNow() {
  if (!pvpInstanceId.value) {
    pvpStatusMessage.value = "No hay instancia Paraguacraft PvP 1.8.9 instalada.";
    return;
  }
  pvpSyncBusy.value = true;
  pvpStatusMessage.value = null;
  try {
    await api.installPvpBundle(pvpInstanceId.value);
    await refreshPvpClientStatus();
    pvpStatusMessage.value = "Cliente PvP 1.8.9 sincronizado.";
  } catch (e) {
    pvpStatusMessage.value = String(e);
  } finally {
    pvpSyncBusy.value = false;
  }
}

async function refreshPvpModernClientStatus() {
  if (!isTauri()) return;
  pvpModernStatusLoading.value = true;
  pvpModernStatusMessage.value = null;
  try {
    if (!instancesStore.loaded) await instancesStore.load();
    const pvp = instancesStore.instances.find(
      (i) => normalizeLoaderId(i.loader) === "paraguacraft-pvp-modern",
    );
    pvpModernInstanceId.value = pvp?.id ?? null;
    pvpModernStatus.value = await api.getPvpModernClientStatus(pvp?.id ?? null);
  } catch (e) {
    pvpModernStatusMessage.value = String(e);
  } finally {
    pvpModernStatusLoading.value = false;
  }
}

async function syncPvpModernClientNow() {
  if (!pvpModernInstanceId.value) {
    pvpModernStatusMessage.value = "No hay instancia Paraguacraft PvP 1.21.11 instalada.";
    return;
  }
  pvpModernSyncBusy.value = true;
  pvpModernStatusMessage.value = null;
  try {
    await api.installPvpModernBundle(pvpModernInstanceId.value);
    await refreshPvpModernClientStatus();
    pvpModernStatusMessage.value = "Cliente PvP 1.21.11 sincronizado.";
  } catch (e) {
    pvpModernStatusMessage.value = String(e);
  } finally {
    pvpModernSyncBusy.value = false;
  }
}

const maxRam = computed(() => (app.hardware ? app.hardware.ramGb * 1024 : 16384));
const gcOptions: GcType[] = ["Auto", "G1GC", "ZGC", "Shenandoah"];
const javaPriorityOptions = [
  { value: "normal", label: "Normal" },
  { value: "high", label: "Alta" },
  { value: "realtime", label: "Tiempo real (avanzado)" },
];

async function refreshExtrasStatus() {
  if (!isTauri()) return;
  extrasLoading.value = true;
  try {
    const data = await api.getExtrasPanelData();
    extrasStatus.value = data.extras;
    cleanupInfo.value = data.cleanup;
  } finally {
    extrasLoading.value = false;
  }
}

async function toggleGameMode() {
  if (!isTauri()) return;
  extrasBusy.value = true;
  extrasMessage.value = null;
  try {
    const msgs = extrasStatus.value?.gameModeActive
      ? await api.deactivateGameMode()
      : await api.activateGameMode();
    extrasMessage.value = msgs.join(" · ");
    await refreshExtrasStatus();
  } catch (e) {
    extrasMessage.value = String(e);
  } finally {
    extrasBusy.value = false;
  }
}

async function toggleTurbo() {
  if (!isTauri()) return;
  extrasBusy.value = true;
  extrasMessage.value = null;
  try {
    const msgs = extrasStatus.value?.turboActive
      ? await api.deactivateTurboMode()
      : await api.activateTurboMode();
    extrasMessage.value = msgs.join(" · ");
    await refreshExtrasStatus();
  } catch (e) {
    extrasMessage.value = String(e);
  } finally {
    extrasBusy.value = false;
  }
}

async function applyJavaPriority(level: string) {
  if (!isTauri()) return;
  settings.update("javaPriority", level);
  extrasBusy.value = true;
  try {
    const n = await api.setJavaPriority(level);
    extrasMessage.value = `Prioridad Java → ${level} (${n} proceso(s))`;
    await refreshExtrasStatus();
  } catch (e) {
    extrasMessage.value = String(e);
  } finally {
    extrasBusy.value = false;
  }
}

async function runCleanup(kind: "logs" | "crash" | "both") {
  if (!isTauri()) return;
  extrasBusy.value = true;
  extrasMessage.value = null;
  try {
    const n = await api.runCleanup(kind);
    extrasMessage.value = `Limpieza: ${n} archivo(s) eliminado(s)`;
    cleanupInfo.value = await api.getCleanupInfo();
  } catch (e) {
    extrasMessage.value = String(e);
  } finally {
    extrasBusy.value = false;
  }
}
</script>

<template>
  <div class="mx-auto max-w-3xl p-8">
    <h1 class="mb-6 text-2xl font-bold">Ajustes</h1>

    <div v-if="!pageReady" class="space-y-4">
      <div v-for="n in 4" :key="n" class="h-28 animate-pulse rounded-xl bg-surface-3" />
    </div>

    <template v-else-if="settings.settings">
      <!-- Rendimiento -->
      <section class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#9889;</span> Rendimiento
        </h2>

        <div class="mb-5">
          <div class="mb-1 flex justify-between text-sm">
            <span class="text-gray-300">Memoria RAM asignada</span>
            <span class="font-bold text-pc-green">{{ formatRam(settings.settings.ramMb) }}</span>
          </div>
          <input
            :value="settings.settings.ramMb"
            type="range"
            :min="2048"
            :max="maxRam"
            :step="512"
            class="w-full accent-pc-green"
            @input="settings.update('ramMb', Number(($event.target as HTMLInputElement).value))"
          />
          <p class="mt-1 text-xs text-gray-500">
            Recomendado para tu hardware: {{ formatRam(app.hardware?.recommendedRamMb ?? 4096) }}
          </p>
        </div>

        <label class="mb-5 block">
          <span class="mb-1 block text-sm text-gray-300">Perfil de rendimiento</span>
          <select
            :value="settings.settings.performanceTier ?? 'auto'"
            class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            @change="settings.update('performanceTier', ($event.target as HTMLSelectElement).value)"
          >
            <option value="auto">Auto (detectar PC)</option>
            <option value="baja">Baja — máximo FPS</option>
            <option value="media">Media — equilibrado</option>
            <option value="alta">Alta — calidad / shaders</option>
            <option value="custom">Personalizado — no tocar options.txt</option>
          </select>
          <p class="mt-1 text-xs text-gray-500">
            Detectado: {{ app.hardware?.perfilSugerido ?? "…" }}. Auto usa el hardware y el preset de uso.
          </p>
        </label>

        <label class="mb-5 block">
          <span class="mb-1 block text-sm text-gray-300">Estilo de juego PvP</span>
          <select
            :value="settings.settings.pvpPlayStyle ?? 'competitive'"
            class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            @change="settings.update('pvpPlayStyle', ($event.target as HTMLSelectElement).value)"
          >
            <option value="competitive">Full rendimiento — fluidez 1.8-like</option>
            <option value="casual">Casual — un poco más visual</option>
          </select>
          <p class="mt-1 text-xs text-gray-500">
            Se aplica a PvP 1.21.11 (y configs del launcher) al jugar: bobView off, RD/sim justos, Sodium Extra, shaders off.
          </p>
        </label>

        <label class="mb-5 block">
          <span class="mb-1 block text-sm text-gray-300">Preset por uso</span>
          <select
            :value="settings.settings.usagePreset ?? 'balanced'"
            class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            :disabled="(settings.settings.performanceTier ?? 'auto') !== 'auto'"
            @change="settings.update('usagePreset', ($event.target as HTMLSelectElement).value)"
          >
            <option value="balanced">Equilibrado</option>
            <option value="gameplay">Jugabilidad (supervivencia / builds)</option>
            <option value="shaders">Shaders</option>
            <option value="pvp">PvP / competitivo</option>
            <option value="lightweight">PC papa / liviano</option>
          </select>
          <p class="mt-1 text-xs text-gray-500">
            Solo aplica con perfil Auto. PvP fuerza Baja; Shaders empuja a Media/Alta.
          </p>
        </label>

        <label class="mb-5 block">
          <span class="mb-1 block text-sm text-gray-300">Garbage Collector</span>
          <select
            :value="settings.settings.gcType"
            class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            @change="settings.update('gcType', ($event.target as HTMLSelectElement).value as GcType)"
          >
            <option v-for="gc in gcOptions" :key="gc" :value="gc">{{ gc }}</option>
          </select>
        </label>

        <div class="space-y-4">
          <BaseToggle
            :model-value="settings.settings.optimizeGraphics"
            label="Optimizar graficos"
            hint="Aplica ajustes de bajo consumo en options.txt al jugar."
            @update:model-value="settings.update('optimizeGraphics', $event)"
          />
          <BaseToggle
            :model-value="settings.settings.closeOnLaunch"
            label="Ocultar launcher al jugar"
            hint="Queda en la bandeja (no se cierra). Discord RPC sigue. Al salir de Minecraft vuelve. Un server local prendido también sigue: usá la pestaña verde o Detener."
            @update:model-value="settings.update('closeOnLaunch', $event)"
          />
          <BaseToggle
            :model-value="settings.settings.showGameConsole ?? false"
            label="Mostrar consola del juego"
            hint="Abre la ventana de Java al jugar (útil para ver crashes de mods). Si también cerrás el launcher, queda en bandeja en vez de salir."
            @update:model-value="settings.update('showGameConsole', $event)"
          />
          <div class="flex flex-wrap items-center gap-2">
            <BaseButton
              size="sm"
              variant="secondary"
              :disabled="copyArgsBusy || !isTauri()"
              @click="copyLastLaunchArgs"
            >
              {{ copyArgsBusy ? "Copiando…" : "Copiar args del último launch" }}
            </BaseButton>
            <p
              v-if="copyArgsMessage"
              class="text-xs"
              :class="copyArgsMessage.startsWith('Args copiados') ? 'text-pc-green' : 'text-red-400'"
            >
              {{ copyArgsMessage }}
            </p>
          </div>
          <BaseToggle
            :model-value="settings.settings.competeTurbo ?? false"
            label="Turbo en Modo Competir"
            hint="DNS Cloudflare y cierre de apps de red al usar Competir."
            @update:model-value="settings.update('competeTurbo', $event)"
          />
          <BaseToggle
            :model-value="settings.settings.trayLite ?? true"
            label="Bandeja ultra-lite"
            hint="Icono en la bandeja mientras jugás o hay un server activo. «Salir» desde ahí sí apaga los servers locales."
            @update:model-value="settings.update('trayLite', $event)"
          />
        </div>

        <div v-if="app.hardware" class="mt-5 rounded-lg border border-surface-4 bg-surface-3/50 p-4 text-sm">
          <p class="font-semibold text-white">{{ app.hardware.cpuName }}</p>
          <p class="text-gray-400">{{ app.hardware.gpuName }} · {{ app.hardware.ramGb }} GB RAM · perfil {{ app.hardware.perfilSugerido }}</p>
        </div>

        <div class="mt-4 flex flex-wrap gap-2">
          <BaseButton size="sm" variant="secondary" :disabled="perfBusy" @click="applyHardwareRecommended">
            Aplicar recomendado del hardware
          </BaseButton>
          <BaseButton size="sm" variant="secondary" :disabled="perfBusy" @click="optimizeMinecraftOptions">
            Optimizar options.txt
          </BaseButton>
        </div>
        <p v-if="perfMessage" class="mt-3 text-xs" :class="perfMessage.startsWith('options') || perfMessage.startsWith('Perfil') ? 'text-pc-green' : 'text-red-400'">
          {{ perfMessage }}
        </p>
      </section>

      <!-- Extras / rendimiento avanzado -->
      <section class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#128640;</span> Extras y rendimiento
        </h2>

        <div v-if="extrasLoading" class="mb-4 h-16 animate-pulse rounded-lg bg-surface-3" />
        <div v-else-if="!extrasStatus && isTauri()" class="mb-4">
          <BaseButton size="sm" variant="secondary" @click="refreshExtrasStatus">
            Cargar datos de limpieza y RAM
          </BaseButton>
        </div>

        <div class="mb-5 space-y-4">
          <label class="block">
            <span class="mb-1 block text-sm text-gray-300">Descargas simultáneas</span>
            <select
              :value="String(settings.settings.downloadConcurrency ?? 0)"
              class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
              @change="settings.update('downloadConcurrency', Number(($event.target as HTMLSelectElement).value))"
            >
              <option value="0">Auto (según PC)</option>
              <option value="2">2 — red lenta / PC papa</option>
              <option value="4">4 — equilibrado</option>
              <option value="8">8 — rápido</option>
              <option value="12">12 — máximo</option>
            </select>
            <p class="mt-1 text-xs text-gray-500">Cuántos archivos se bajan a la vez (mods, libs, Java).</p>
          </label>

          <label class="block">
            <span class="mb-1 block text-sm text-gray-300">Compatibilidad GPU</span>
            <select
              :value="settings.settings.gpuCompatMode"
              class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
              @change="settings.update('gpuCompatMode', ($event.target as HTMLSelectElement).value as AppSettings['gpuCompatMode'])"
            >
              <option value="off">Normal (GPU del sistema)</option>
              <option value="mesa-d3d12">Mesa D3D12 (si tenés Mesa)</option>
              <option value="mesa-llvmpipe">Mesa software (muy lento, último recurso)</option>
              <option value="mesa-zink">Mesa Zink</option>
            </select>
            <p class="mt-1 text-xs text-gray-500">
              Solo si Minecraft no abre por la GPU. Requiere Mesa en el PATH.
            </p>
          </label>

          <BaseToggle
            :model-value="settings.settings.papaMode ?? false"
            label="Modo Papa inteligente"
            hint="PCs débiles: 800×600, 2 GB RAM, G1GC, chunks 4, gráficos Fast y mods de rendimiento (Sodium/Lithium/Iris en Fabric)."
            @update:model-value="settings.update('papaMode', $event)"
          />
          <div class="flex flex-wrap gap-3">
            <label class="block">
              <span class="mb-1 block text-sm text-gray-300">Ancho personalizado</span>
              <input
                type="number"
                min="0"
                max="7680"
                :value="settings.settings.gameWidth ?? 0"
                :disabled="settings.settings.papaMode"
                class="w-28 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green disabled:opacity-50"
                @input="settings.update('gameWidth', Number(($event.target as HTMLInputElement).value) || 0)"
              />
            </label>
            <label class="block">
              <span class="mb-1 block text-sm text-gray-300">Alto personalizado</span>
              <input
                type="number"
                min="0"
                max="4320"
                :value="settings.settings.gameHeight ?? 0"
                :disabled="settings.settings.papaMode"
                class="w-28 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green disabled:opacity-50"
                @input="settings.update('gameHeight', Number(($event.target as HTMLInputElement).value) || 0)"
              />
            </label>
          </div>
          <p class="text-xs text-gray-500">
            0 × 0 = no forzar (salvo Modo PC Papa). Ej: 1920 × 1080.
          </p>
          <label class="block">
            <span class="mb-1 block text-sm text-gray-300">Argumentos JVM globales</span>
            <input
              type="text"
              :value="settings.settings.globalJvmArgs ?? ''"
              placeholder="-XX:+UseG1GC …"
              class="w-full max-w-xl rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
              @input="settings.update('globalJvmArgs', ($event.target as HTMLInputElement).value)"
            />
            <p class="mt-1 text-xs text-gray-500">
              Se aplican a todas las instancias, antes de los args de cada una.
            </p>
          </label>
          <BaseToggle
            :model-value="settings.settings.deepCleanOnLaunch ?? false"
            label="Limpieza profunda al jugar"
            hint="Borra logs viejos y crash-reports antes de cada partida."
            @update:model-value="settings.update('deepCleanOnLaunch', $event)"
          />
          <label class="block">
            <span class="mb-1 block text-sm text-gray-300">Auto-backup al jugar (horas, 0 = desactivado)</span>
            <input
              type="number"
              min="0"
              max="168"
              :value="settings.settings.backupAutoHours ?? 0"
              class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
              @input="settings.update('backupAutoHours', Number(($event.target as HTMLInputElement).value))"
            />
          </label>
          <label class="block">
            <span class="mb-1 block text-sm text-gray-300">Prioridad Java al lanzar</span>
            <select
              :value="settings.settings.javaPriority ?? 'high'"
              class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
              @change="applyJavaPriority(($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in javaPriorityOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </label>
        </div>

        <div v-if="isTauri() && !extrasLoading" class="space-y-3 border-t border-surface-4 pt-4">
          <div class="flex flex-wrap gap-2">
            <BaseButton
              size="sm"
              :variant="extrasStatus?.gameModeActive ? 'primary' : 'secondary'"
              :disabled="extrasBusy"
              @click="toggleGameMode"
            >
              {{ extrasStatus?.gameModeActive ? "Desactivar Game Mode" : "Activar Game Mode" }}
            </BaseButton>
            <BaseButton
              size="sm"
              :variant="extrasStatus?.turboActive ? 'primary' : 'secondary'"
              :disabled="extrasBusy"
              @click="toggleTurbo"
            >
              {{ extrasStatus?.turboActive ? "Desactivar Turbo" : "Modo Turbo Anti-Lag" }}
            </BaseButton>
          </div>
          <p class="text-xs text-gray-500">
            Game Mode cierra Teams/OneDrive y sube la prioridad de Java.
            Turbo cambia DNS a Cloudflare y cierra apps que consumen red.
          </p>

          <div v-if="cleanupInfo" class="rounded-lg border border-surface-4 bg-surface-3/50 p-3 text-xs text-gray-400">
            Logs: {{ cleanupInfo.logsMb }} MB · Crash reports: {{ cleanupInfo.crashMb }} MB · RAM Java: {{ cleanupInfo.mcRamMb }} MB
          </div>
          <div class="flex flex-wrap gap-2">
            <BaseButton size="sm" variant="secondary" :disabled="extrasBusy" @click="runCleanup('logs')">
              Limpiar logs
            </BaseButton>
            <BaseButton size="sm" variant="secondary" :disabled="extrasBusy" @click="runCleanup('crash')">
              Limpiar crashes
            </BaseButton>
            <BaseButton size="sm" variant="secondary" :disabled="extrasBusy" @click="runCleanup('both')">
              Limpieza completa
            </BaseButton>
          </div>
        </div>

        <div class="mt-5 space-y-4 border-t border-surface-4 pt-4">
          <h3 class="text-sm font-bold text-gray-200">Discord Rich Presence</h3>
          <BaseToggle
            :model-value="settings.settings.discordRpc !== false"
            label="Mostrar estado en Discord"
            @update:model-value="settings.update('discordRpc', $event)"
          />
          <BaseToggle
            :model-value="settings.settings.discordRpcVersion !== false"
            label="Mostrar versión de Minecraft"
            :disabled="settings.settings.discordRpc === false"
            @update:model-value="settings.update('discordRpcVersion', $event)"
          />
          <BaseToggle
            :model-value="settings.settings.discordRpcTime !== false"
            label="Mostrar tiempo en el launcher"
            :disabled="settings.settings.discordRpc === false"
            @update:model-value="settings.update('discordRpcTime', $event)"
          />
        </div>

        <p v-if="extrasMessage" class="mt-3 text-xs text-pc-green">{{ extrasMessage }}</p>
      </section>

      <!-- Cuentas -->
      <section id="accounts" class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <div class="mb-4 flex items-center justify-between">
          <h2 class="flex items-center gap-2 text-lg font-bold"><span class="font-emoji">&#128100;</span> Cuentas</h2>
          <BaseButton size="sm" variant="secondary" @click="showAddAccount = true">+ Agregar cuenta</BaseButton>
        </div>
        <div class="space-y-2">
          <div
            v-for="acc in accounts.accounts"
            :key="acc.id"
            class="flex items-center gap-3 rounded-lg border p-3"
            :class="acc.active ? 'border-pc-green bg-pc-green/5' : 'border-surface-4'"
          >
            <SkinAvatar
              :key="acc.active ? `${skins.revision}:${skins.activeSkin?.skinUrl ?? ''}` : acc.id"
              :uuid="acc.uuid"
              :username="acc.username"
              :avatar-url="acc.active ? skins.activeSkin?.avatarUrl : acc.avatarUrl"
              :avatar-data-url="acc.active ? skins.activeSkin?.avatarDataUrl : null"
              :local-avatar-path="acc.active ? skins.activeSkin?.localAvatarPath : null"
              size="sm"
            />
            <div class="min-w-0 flex-1">
              <template v-if="renamingId === acc.id">
                <div class="flex flex-wrap items-center gap-2">
                  <input
                    v-model="renameDraft"
                    type="text"
                    maxlength="16"
                    class="min-w-0 flex-1 rounded-md border border-surface-4 bg-surface-1 px-2 py-1 text-sm font-semibold text-white outline-none focus:border-pc-green"
                    placeholder="Nuevo nick (Ely.by)"
                    @keydown.enter.prevent="confirmRename"
                    @keydown.escape.prevent="cancelRename"
                  />
                  <BaseButton size="sm" :disabled="renameBusy" @click="confirmRename">
                    {{ renameBusy ? "…" : "Guardar" }}
                  </BaseButton>
                  <BaseButton size="sm" variant="ghost" :disabled="renameBusy" @click="cancelRename">
                    Cancelar
                  </BaseButton>
                </div>
                <p v-if="renameError" class="mt-1 text-xs text-red-400">{{ renameError }}</p>
                <p class="mt-1 text-[11px] text-gray-500">
                  Usá el mismo nick que en Ely.by. Solo letras, números y _.
                </p>
              </template>
              <template v-else>
                <p class="font-semibold">{{ acc.username }}</p>
                <p class="text-xs" :class="acc.premium ? 'text-pc-green' : 'text-gray-500'">
                  {{ acc.premium ? "Microsoft Premium" : "Offline / no-premium" }}
                </p>
              </template>
            </div>
            <template v-if="renamingId !== acc.id">
              <BaseButton
                v-if="!acc.premium"
                size="sm"
                variant="ghost"
                title="Cambiar nick (p. ej. para Ely.by)"
                @click="startRename(acc)"
              >
                Renombrar
              </BaseButton>
              <BaseButton v-if="!acc.active" size="sm" variant="ghost" @click="accounts.setActive(acc.id)">
                Usar
              </BaseButton>
              <span v-else class="text-xs font-bold text-pc-green">Activa</span>
              <button
                class="text-gray-600 transition hover:text-red-400"
                title="Quitar cuenta"
                @click="accounts.remove(acc.id)"
              >
                &times;
              </button>
            </template>
          </div>
          <p v-if="!accounts.accounts.length" class="text-sm text-gray-500">
            No hay cuentas. Agrega una para jugar.
          </p>
          <p class="text-xs text-gray-500">
            ¿Querés sincronizar con Ely.by? Renombrá tu cuenta offline al mismo nick que registraste ahí.
          </p>
        </div>
      </section>

      <!-- Skin No-Premium -->
      <section
        v-if="accounts.active && !accounts.active.premium"
        class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6"
      >
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#129504;</span> Skin No-Premium
        </h2>
        <p class="mb-4 text-sm text-gray-400">
          Subi un PNG 64x64 (o 64x32) para ver tu skin en el juego y en servidores locales con SkinsRestorer.
        </p>
        <div class="mb-4 flex items-center gap-3">
          <SkinAvatar
            :key="`${skins.revision}:${skins.activeSkin?.skinUrl ?? ''}`"
            :uuid="accounts.active.uuid"
            :username="accounts.active.username"
            :avatar-url="skins.activeSkin?.avatarUrl"
            :avatar-data-url="skins.activeSkin?.avatarDataUrl"
            :local-avatar-path="skins.activeSkin?.localAvatarPath"
          />
          <RouterLink to="/skins" class="text-xs text-pc-green hover:underline">Abrir editor de skins →</RouterLink>
          <BaseButton size="sm" :disabled="skinBusy" @click="applyOfflineSkin">
            {{ skinBusy ? "Aplicando…" : "Elegir skin (.png)" }}
          </BaseButton>
        </div>
        <p v-if="skinMessage" class="text-sm" :class="skinMessage.startsWith('No se') ? 'text-red-400' : 'text-pc-green'">
          {{ skinMessage }}
        </p>
      </section>

      <!-- Cliente PvP 1.8.9 (actualización independiente del launcher) -->
      <section v-if="isTauri()" class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#9876;</span> Cliente PvP 1.8.9
        </h2>
        <p class="text-sm text-gray-400">
          El mod del cliente se actualiza solo al lanzar el juego (lee el manifest en GitHub).
          <strong class="text-gray-300">No necesitás reinstalar el launcher</strong> para cada versión nueva del cliente.
          Sin internet podés jugar con los mods ya instalados; al reconectar se sincroniza solo.
        </p>
        <div v-if="pvpStatusLoading" class="mt-3 text-sm text-gray-500">Consultando versión…</div>
        <template v-else-if="pvpStatus">
          <dl class="mt-4 grid gap-2 text-sm">
            <div class="flex flex-wrap gap-x-2">
              <dt class="text-gray-500">Publicada:</dt>
              <dd class="font-semibold text-white">{{ pvpStatus.remoteVersion }}</dd>
              <dd class="text-gray-500">({{ pvpStatus.remoteFilename }})</dd>
            </div>
            <div class="flex flex-wrap gap-x-2">
              <dt class="text-gray-500">Instalada:</dt>
              <dd v-if="pvpStatus.installedVersion" class="font-semibold text-white">
                {{ pvpStatus.installedVersion }}
              </dd>
              <dd v-else class="text-amber-400">sin instancia / no detectada</dd>
              <dd v-if="pvpStatus.installedFilename" class="text-gray-500">
                ({{ pvpStatus.installedFilename }})
              </dd>
            </div>
            <div class="flex flex-wrap gap-x-2">
              <dt class="text-gray-500">Estado:</dt>
              <dd :class="pvpStatus.upToDate ? 'text-pc-green' : 'text-amber-400'">
                {{ pvpStatus.upToDate ? "Al día" : "Pendiente de sincronizar" }}
              </dd>
            </div>
            <div class="flex flex-wrap gap-x-2">
              <dt class="text-gray-500">Fuente:</dt>
              <dd class="text-gray-400">
                {{ pvpStatus.manifestSource === "remote" ? "manifest en línea" : pvpStatus.manifestSource === "bundled" ? "manifest embebido" : "respaldo offline" }}
              </dd>
            </div>
          </dl>
        </template>
        <p v-if="pvpStatusMessage" class="mt-3 text-sm" :class="pvpStatusMessage.startsWith('No') || pvpStatusMessage.includes('Error') ? 'text-red-400' : 'text-pc-green'">
          {{ pvpStatusMessage }}
        </p>
        <div class="mt-4 flex flex-wrap gap-2">
          <BaseButton size="sm" variant="secondary" :disabled="pvpStatusLoading" @click="refreshPvpClientStatus">
            {{ pvpStatusLoading ? "Consultando…" : "Verificar cliente" }}
          </BaseButton>
          <BaseButton
            v-if="pvpInstanceId"
            size="sm"
            variant="secondary"
            :disabled="pvpSyncBusy || pvpStatusLoading"
            @click="syncPvpClientNow"
          >
            {{ pvpSyncBusy ? "Sincronizando…" : "Sincronizar ahora" }}
          </BaseButton>
        </div>
      </section>

      <!-- Cliente PvP 1.21.11 (actualización independiente del launcher) -->
      <section v-if="isTauri()" class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#9889;</span> Cliente PvP 1.21.11
        </h2>
        <p class="text-sm text-gray-400">
          El mod Fabric se sincroniza al lanzar (manifest en GitHub), igual que el cliente 1.8.9.
          <strong class="text-gray-300">No necesitás actualizar el launcher</strong> para cada versión del mod.
        </p>
        <div v-if="pvpModernStatusLoading" class="mt-3 text-sm text-gray-500">Consultando versión…</div>
        <template v-else-if="pvpModernStatus">
          <dl class="mt-4 grid gap-2 text-sm">
            <div class="flex flex-wrap gap-x-2">
              <dt class="text-gray-500">Publicada:</dt>
              <dd class="font-semibold text-white">{{ pvpModernStatus.remoteVersion }}</dd>
              <dd class="text-gray-500">({{ pvpModernStatus.remoteFilename }})</dd>
            </div>
            <div class="flex flex-wrap gap-x-2">
              <dt class="text-gray-500">Instalada:</dt>
              <dd v-if="pvpModernStatus.installedVersion" class="font-semibold text-white">
                {{ pvpModernStatus.installedVersion }}
              </dd>
              <dd v-else class="text-amber-400">sin instancia / no detectada</dd>
              <dd v-if="pvpModernStatus.installedFilename" class="text-gray-500">
                ({{ pvpModernStatus.installedFilename }})
              </dd>
            </div>
            <div class="flex flex-wrap gap-x-2">
              <dt class="text-gray-500">Estado:</dt>
              <dd :class="pvpModernStatus.upToDate ? 'text-pc-green' : 'text-amber-400'">
                {{ pvpModernStatus.upToDate ? "Al día" : "Pendiente de sincronizar" }}
              </dd>
            </div>
            <div class="flex flex-wrap gap-x-2">
              <dt class="text-gray-500">Fuente:</dt>
              <dd class="text-gray-400">
                {{ pvpModernStatus.manifestSource === "remote" ? "manifest en línea" : pvpModernStatus.manifestSource === "bundled" ? "manifest embebido" : "respaldo offline" }}
              </dd>
            </div>
          </dl>
        </template>
        <p v-if="pvpModernStatusMessage" class="mt-3 text-sm" :class="pvpModernStatusMessage.startsWith('No') || pvpModernStatusMessage.includes('Error') ? 'text-red-400' : 'text-pc-green'">
          {{ pvpModernStatusMessage }}
        </p>
        <div class="mt-4 flex flex-wrap gap-2">
          <BaseButton size="sm" variant="secondary" :disabled="pvpModernStatusLoading" @click="refreshPvpModernClientStatus">
            {{ pvpModernStatusLoading ? "Consultando…" : "Verificar cliente" }}
          </BaseButton>
          <BaseButton
            v-if="pvpModernInstanceId"
            size="sm"
            variant="secondary"
            :disabled="pvpModernSyncBusy || pvpModernStatusLoading"
            @click="syncPvpModernClientNow"
          >
            {{ pvpModernSyncBusy ? "Sincronizando…" : "Sincronizar ahora" }}
          </BaseButton>
        </div>
      </section>

      <!-- Launcher / actualizaciones -->
      <section class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#128640;</span> Launcher
        </h2>
        <p class="text-sm text-gray-400">
          Versión actual:
          <span class="font-semibold text-white">{{ app.updateInfo?.currentVersion ?? "…" }}</span>
        </p>
        <p v-if="app.updateInfo?.updateAvailable" class="mt-2 text-sm text-pc-green">
          Nueva versión {{ app.updateInfo.latestVersion }} disponible.
        </p>
        <p v-else-if="app.updateInfo" class="mt-2 text-sm text-gray-500">Estás al día.</p>
        <p v-if="app.updateInfo?.releaseNotes" class="mt-2 max-h-24 overflow-y-auto whitespace-pre-wrap text-xs text-gray-500">
          {{ app.updateInfo.releaseNotes }}
        </p>
        <div class="mt-4">
          <BaseToggle
            :model-value="settings.settings?.autoUpdateCheck !== false"
            label="Buscar actualizaciones al iniciar"
            @update:model-value="settings.update('autoUpdateCheck', $event)"
          />
        </div>
        <div v-if="app.updating && app.updateProgress" class="mt-4">
          <div class="mb-1 flex justify-between text-xs text-gray-400">
            <span>{{ app.updateProgress.message }}</span>
            <span>{{ Math.round((app.updateProgress.progress ?? 0) * 100) }}%</span>
          </div>
          <div class="h-1.5 overflow-hidden rounded-full bg-surface-4">
            <div
              class="h-full bg-pc-green transition-all"
              :style="{ width: `${Math.round((app.updateProgress.progress ?? 0) * 100)}%` }"
            />
          </div>
        </div>
        <div class="mt-4 flex flex-wrap items-center gap-2">
          <BaseButton size="sm" variant="secondary" :disabled="checkingUpdate" @click="refreshUpdate">
            {{ checkingUpdate ? "Buscando…" : "Buscar actualizaciones" }}
          </BaseButton>
          <BaseButton
            v-if="app.updateInfo?.updateAvailable && app.updateInfo.inAppInstall"
            size="sm"
            variant="secondary"
            :disabled="app.updating"
            @click="app.installUpdate()"
          >
            {{ app.updating ? "Actualizando…" : "Actualizar ahora" }}
          </BaseButton>
          <BaseButton
            v-else-if="app.updateInfo?.updateAvailable"
            size="sm"
            variant="secondary"
            @click="app.openUpdateDownload()"
          >
            Descargar en GitHub
          </BaseButton>
        </div>
      </section>

      <!-- Java -->
      <JavaManager v-if="javaSectionReady" :auto-detect="true" />
      <div v-else class="mb-6 h-40 animate-pulse rounded-xl bg-surface-3" />

      <!-- Paraguabot -->
      <section class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#129302;</span> Paraguabot
        </h2>
        <p class="mb-4 text-sm text-gray-400">
          Asistente con IA para settings PvP, crashes y el launcher.
        </p>
        <p v-if="keysManaged" class="mb-3 text-xs text-pc-green">
          Paraguabot y CurseForge están habilitados por el launcher (no necesitás API keys).
        </p>
        <p v-else-if="aiConfigured === true" class="mb-3 text-xs text-pc-green">Paraguabot configurado (Groq/OpenAI detectado).</p>
        <p v-else-if="aiConfigured === false" class="mb-3 text-xs text-amber-400">Sin API key — Paraguabot no puede responder con IA.</p>
        <template v-if="!keysManaged">
        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Groq API Key</span>
          <input
            v-model="groqKey"
            type="password"
            placeholder="gsk_..."
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          />
        </label>
        <div class="mt-3">
          <BaseButton size="sm" variant="secondary" @click="saveGroqKey">Guardar key</BaseButton>
        </div>
        <p v-if="groqKeyMessage" class="mt-2 text-xs" :class="groqKeyMessage.includes('guardada') ? 'text-pc-green' : 'text-red-400'">
          {{ groqKeyMessage }}
        </p>
        </template>
      </section>

      <!-- Tienda -->
      <section class="rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold"><span class="font-emoji">&#128722;</span> Tienda</h2>
        <p v-if="keysManaged" class="mb-3 text-xs text-pc-green">CurseForge habilitado por el launcher.</p>
        <label v-if="!keysManaged" class="block">
          <span class="mb-1 block text-sm text-gray-300">CurseForge API Key (opcional)</span>
          <input
            type="password"
            :value="settings.settings.curseforgeApiKey ?? ''"
            placeholder="Necesaria solo para la tienda de CurseForge"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            @input="settings.update('curseforgeApiKey', ($event.target as HTMLInputElement).value)"
          />
          <span class="mt-1 block text-xs text-gray-500">
            Modrinth funciona sin key. CurseForge requiere una key gratuita de
            <a class="text-pc-green underline" href="https://console.curseforge.com/" target="_blank" rel="noopener">console.curseforge.com</a>.
            Si la key empieza con <code class="text-gray-400">$2a$</code>, en <code class="text-gray-400">launcher/.env</code> ponela entre comillas simples para evitar error 403.
          </span>
        </label>
      </section>

      <!-- Apariencia -->
      <section class="rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#127912;</span> {{ i18n.t("settings_appearance") }}
        </h2>

        <label class="mb-5 block">
          <span class="mb-1 block text-sm text-gray-300">{{ i18n.t("settings_theme") }}</span>
          <select
            :value="settings.settings.theme"
            class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            @change="settings.update('theme', ($event.target as HTMLSelectElement).value as 'dark' | 'darker')"
          >
            <option value="dark">{{ i18n.t("settings_theme_dark") }}</option>
            <option value="darker">{{ i18n.t("settings_theme_darker") }}</option>
          </select>
        </label>

        <label class="mb-5 block">
          <span class="mb-1 block text-sm text-gray-300">{{ i18n.t("settings_language") }}</span>
          <select
            :value="settings.settings.language"
            class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            @change="i18n.setLanguage(($event.target as HTMLSelectElement).value as 'es' | 'en' | 'pt')"
          >
            <option value="es">Español</option>
            <option value="en">English</option>
            <option value="pt">Português</option>
          </select>
          <p class="mt-1 text-xs text-gray-500">Menú y textos principales. Se irá traduciendo el resto.</p>
        </label>

        <p class="mb-2 text-sm text-gray-300">{{ i18n.t("settings_accent") }}</p>
        <div class="flex gap-3">
          <button
            class="h-9 w-9 rounded-full ring-2 ring-offset-2 ring-offset-surface-2"
            style="background: #2ecc71"
            :class="settings.settings.accent === 'green' ? 'ring-pc-green' : 'ring-transparent'"
            @click="setAccent('green')"
          />
          <button
            class="h-9 w-9 rounded-full ring-2 ring-offset-2 ring-offset-surface-2"
            style="background: #9b59b6"
            :class="settings.settings.accent === 'ai' ? 'ring-pc-ai' : 'ring-transparent'"
            @click="setAccent('ai')"
          />
        </div>
      </section>
    </template>

    <AddAccountModal v-if="showAddAccount" @close="showAddAccount = false" />
  </div>
</template>
