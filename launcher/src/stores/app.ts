import { defineStore } from "pinia";
import { ref } from "vue";
import type { HardwareInfo, UpdateInfo, UpdateProgress, CrashDiagnosis } from "@/lib/types";
import { api, isTauri, openUrl } from "@/lib/ipc";
import { useAiStore } from "@/stores/ai";
import { useSettingsStore } from "@/stores/settings";
import { useSkinsStore } from "@/stores/skins";

export type LaunchPhase = "idle" | "preparing" | "downloading" | "launching" | "running";

export const useAppStore = defineStore("app", () => {
  const hardware = ref<HardwareInfo | null>(null);
  const runningInTauri = ref(isTauri());
  const launchPhase = ref<LaunchPhase>("idle");
  const launchMessage = ref("Listo para jugar");
  const updateInfo = ref<UpdateInfo | null>(null);
  const updateProgress = ref<UpdateProgress | null>(null);
  const updating = ref(false);

  async function loadHardware(force = false) {
    if (hardware.value && !force) return;
    hardware.value = await api.getHardwareInfo();
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
    const ai = useAiStore();
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
          ai.pushDiagnosis(ev.payload.diagnosis, ev.payload.instanceId);
        }
      },
    );
  }

  async function checkUpdate(force = false) {
    const settings = useSettingsStore();
    if (!force && settings.loaded && settings.settings?.autoUpdateCheck === false) return;
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
    updateProgress.value = { phase: "check", progress: 0, message: "Buscando actualización…" };
    try {
      await initUpdateEvents();

      // 1) tauri-plugin-updater (releases con latest.json firmado)
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
          // Sin pubkey/latest.json firmado: continuar con fallback GitHub.
        }
      }

      // 2) Fallback: descargar instalador del release de GitHub
      if (updateInfo.value.inAppInstall) {
        await api.downloadAndInstallLauncherUpdate();
        updateProgress.value = { phase: "install", progress: 1, message: "Instalador iniciado. Podés cerrar el launcher." };
        return;
      }

      await openUpdateDownload();
    } finally {
      updating.value = false;
    }
  }

  async function launch(instanceId: string, name: string) {
    await initGameEvents();
    setLaunch("preparing", `Preparando ${name}…`);
    try {
      await api.launchInstance(instanceId);
      if (launchPhase.value === "preparing") setLaunch("launching", `Lanzando ${name}…`);
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
