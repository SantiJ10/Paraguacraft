import { defineStore } from "pinia";
import { ref } from "vue";
import type { HardwareInfo, UpdateInfo, UpdateProgress, CrashDiagnosis } from "@/lib/types";
import { api, isTauri, openUrl } from "@/lib/ipc";
import { useSettingsStore } from "@/stores/settings";
import { useSkinsStore } from "@/stores/skins";

export type LaunchPhase = "idle" | "preparing" | "downloading" | "launching" | "running";

export const useAppStore = defineStore("app", () => {
  const hardware = ref<HardwareInfo | null>(null);
  const runningInTauri = ref(isTauri());
  const launchPhase = ref<LaunchPhase>("idle");
  const launchMessage = ref("Listo para jugar");
  const lastCrash = ref<{
    instanceId: string;
    exitCode: number;
    diagnosis: CrashDiagnosis;
  } | null>(null);
  const updateInfo = ref<UpdateInfo | null>(null);
  const updateProgress = ref<UpdateProgress | null>(null);
  const updating = ref(false);

  async function loadHardware(force = false) {
    if (hardware.value && !force) return;
    hardware.value = await api.getHardwareInfo();
    const { applyHardwareSmartDefaults } = await import("@/stores/music");
    applyHardwareSmartDefaults(hardware.value.perfilSugerido);
  }

  function setLaunch(phase: LaunchPhase, message: string) {
    launchPhase.value = phase;
    launchMessage.value = message;
  }

  let gameEventsBound = false;
  let updateEventsBound = false;

  async function initUpdateEvents() {
    if (updateEventsBound || !isTauri()) return;
    updateEventsBound = true;
    const { listen } = await import("@tauri-apps/api/event");
    await listen<UpdateProgress>("update://progress", (ev) => {
      updateProgress.value = ev.payload;
    });
  }

  async function initGameEvents() {
    if (gameEventsBound || !isTauri()) return;
    gameEventsBound = true;
    const { listen } = await import("@tauri-apps/api/event");
    const skins = useSkinsStore();
    await listen("game://started", () => {
      setLaunch("running", "Jugando — launcher suspendido");
    });
    await listen("game://exited", () => {
      setLaunch("idle", "Listo para jugar");
      void skins.refresh();
    });
    await listen("bedrock://exited", () => {
      void skins.refresh();
    });
    await listen<{ instanceId: string; exitCode: number; diagnosis?: CrashDiagnosis }>(
      "game://crashed",
      (ev) => {
        setLaunch("idle", "El juego terminó con error");
        void skins.refresh();
        if (ev.payload?.diagnosis) {
          lastCrash.value = {
            instanceId: ev.payload.instanceId,
            exitCode: ev.payload.exitCode,
            diagnosis: ev.payload.diagnosis,
          };
        }
      },
    );
  }

  async function checkUpdate(force = false) {
    const settings = useSettingsStore();
    if (!force && settings.loaded && settings.settings?.autoUpdateCheck === false) return;
    if (launchPhase.value === "running") return;
    try {
      await initUpdateEvents();
      updateInfo.value = await api.checkLauncherUpdate();
    } catch {
      updateInfo.value = null;
    }
  }

  async function openUpdateDownload() {
    const url = updateInfo.value?.downloadUrl;
    if (url) await openUrl(url);
  }

  function dismissUpdateBanner() {
    updateInfo.value = null;
  }

  async function installUpdate() {
    if (!updateInfo.value?.updateAvailable) return;
    updating.value = true;
    updateProgress.value = { phase: "check", progress: 0, message: "Preparando actualización…" };
    try {
      await initUpdateEvents();

      // Instalador NSIS completo (Instalar_Paraguacraft_v*.exe): descarga + abre setup.
      if (updateInfo.value.inAppInstall) {
        updateProgress.value = { phase: "download", progress: 0, message: "Descargando instalador…" };
        await api.downloadAndInstallLauncherUpdate();
        updateProgress.value = {
          phase: "install",
          progress: 1,
          message: "Instalador abierto. Seguí el asistente en pantalla.",
        };
        return;
      }

      // Parches incrementales firmados (solo si hay createUpdaterArtifacts + latest.json Tauri).
      if (isTauri()) {
        try {
          const { check } = await import("@tauri-apps/plugin-updater");
          const { relaunch } = await import("@tauri-apps/plugin-process");
          const update = await check();
          if (update) {
            updateProgress.value = { phase: "download", progress: 0, message: "Descargando actualización…" };
            await update.downloadAndInstall((event) => {
              if (event.event === "Progress" && event.data.chunkLength) {
                updateProgress.value = {
                  phase: "download",
                  progress: Math.min(0.99, (updateProgress.value?.progress ?? 0) + 0.02),
                  message: "Descargando actualización firmada…",
                };
              }
            });
            await relaunch();
            return;
          }
        } catch {
          // Sin artefactos firmados en el release.
        }
      }

      await openUpdateDownload();
    } finally {
      updating.value = false;
    }
  }

  function dismissCrash() {
    lastCrash.value = null;
  }

  async function launch(
    instanceId: string,
    name: string,
    serverAddress?: string | null,
    competeMode = false,
  ) {
    await initGameEvents();
    setLaunch(
      "preparing",
      competeMode ? `Modo Competir — ${name}…` : `Preparando ${name}…`,
    );
    try {
      await api.launchInstance(instanceId, serverAddress, competeMode);
      if (launchPhase.value === "preparing") {
        setLaunch("launching", competeMode ? `Competir — ${name}…` : `Lanzando ${name}…`);
      }
    } catch (e) {
      setLaunch("idle", "Listo para jugar");
      throw e;
    }
  }

  return {
    hardware,
    runningInTauri,
    launchPhase,
    launchMessage,
    lastCrash,
    dismissCrash,
    updateInfo,
    updateProgress,
    updating,
    loadHardware,
    setLaunch,
    initGameEvents,
    initUpdateEvents,
    launch,
    checkUpdate,
    openUpdateDownload,
    installUpdate,
    dismissUpdateBanner,
  };
});
