import { defineStore } from "pinia";
import { ref } from "vue";
import type { HardwareInfo } from "@/lib/types";
import { api, isTauri } from "@/lib/ipc";

export type LaunchPhase = "idle" | "preparing" | "downloading" | "launching" | "running";

export const useAppStore = defineStore("app", () => {
  const hardware = ref<HardwareInfo | null>(null);
  const runningInTauri = ref(isTauri());
  const launchPhase = ref<LaunchPhase>("idle");
  const launchMessage = ref("Listo para jugar");

  async function loadHardware() {
    if (hardware.value) return;
    hardware.value = await api.getHardwareInfo();
  }

  function setLaunch(phase: LaunchPhase, message: string) {
    launchPhase.value = phase;
    launchMessage.value = message;
  }

  return { hardware, runningInTauri, launchPhase, launchMessage, loadHardware, setLaunch };
});
