import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Account } from "@/lib/types";
import { api } from "@/lib/ipc";

export const useAccountsStore = defineStore("accounts", () => {
  const accounts = ref<Account[]>([]);
  const loaded = ref(false);

  const active = computed(() => accounts.value.find((a) => a.active) ?? null);

  async function load() {
    if (loaded.value) return;
    accounts.value = await api.getAccounts();
    loaded.value = true;
  }

  function setActive(id: string) {
    accounts.value.forEach((a) => (a.active = a.id === id));
  }

  function addOffline(username: string) {
    const acc: Account = {
      id: `offline-${Date.now()}`,
      type: "offline",
      username,
      uuid: crypto.randomUUID(),
      avatarUrl: "",
      active: false,
      premium: false,
    };
    accounts.value.push(acc);
    setActive(acc.id);
  }

  return { accounts, loaded, active, load, setActive, addOffline };
});
