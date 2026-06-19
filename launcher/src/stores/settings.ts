import { defineStore } from "pinia";
import { ref } from "vue";
import type { AppSettings } from "@/lib/types";
import { api } from "@/lib/ipc";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings | null>(null);
  const loaded = ref(false);

  async function load() {
    if (loaded.value) return;
    settings.value = await api.getSettings();
    loaded.value = true;
  }

  function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    if (settings.value) settings.value[key] = value;
  }

  return { settings, loaded, load, update };
});
