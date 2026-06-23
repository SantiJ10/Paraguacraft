import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Account, DeviceCodeStart } from "@/lib/types";
import { api, isTauri } from "@/lib/ipc";

export const useAccountsStore = defineStore("accounts", () => {
  const accounts = ref<Account[]>([]);
  const loaded = ref(false);

  // Estado del login Microsoft por device-code.
  const msDevice = ref<DeviceCodeStart | null>(null);
  const msPolling = ref(false);
  const msError = ref<string | null>(null);

  const active = computed(() => accounts.value.find((a) => a.active) ?? null);

  async function load(force = false) {
    if (loaded.value && !force) return;
    accounts.value = await api.getAccounts();
    loaded.value = true;
  }

  async function setActive(id: string) {
    accounts.value = await api.setActiveAccount(id);
  }

  async function addOffline(username: string) {
    accounts.value = await api.addOfflineAccount(username);
  }

  async function remove(id: string) {
    accounts.value = await api.removeAccount(id);
  }

  let pollTimer: number | null = null;

  function stopMicrosoftLogin() {
    if (pollTimer !== null) {
      clearTimeout(pollTimer);
      pollTimer = null;
    }
    msPolling.value = false;
    msDevice.value = null;
  }

  /**
   * Inicia el flujo device-code de Microsoft y arranca el poll on-demand.
   * No usa hilos de fondo en Rust: cada poll es una llamada puntual.
   */
  async function startMicrosoftLogin(): Promise<DeviceCodeStart> {
    if (!isTauri()) throw new Error("Login Microsoft solo disponible en la app de escritorio");
    msError.value = null;
    const start = await api.msLoginStart();
    msDevice.value = start;
    msPolling.value = true;

    const tick = async () => {
      if (!msPolling.value) return;
      try {
        const result = await api.msLoginPoll();
        if (result) {
          accounts.value = result;
          stopMicrosoftLogin();
          return;
        }
      } catch (err) {
        msError.value = String(err);
        stopMicrosoftLogin();
        return;
      }
      pollTimer = window.setTimeout(tick, Math.max(start.interval, 3) * 1000);
    };
    pollTimer = window.setTimeout(tick, Math.max(start.interval, 3) * 1000);
    return start;
  }

  return {
    accounts,
    loaded,
    active,
    msDevice,
    msPolling,
    msError,
    load,
    setActive,
    addOffline,
    remove,
    startMicrosoftLogin,
    stopMicrosoftLogin,
  };
});
