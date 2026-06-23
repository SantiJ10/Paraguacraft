import { defineStore } from "pinia";
import { ref } from "vue";
import type { AppSettings } from "@/lib/types";
import { api } from "@/lib/ipc";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings | null>(null);
  const loaded = ref(false);
  const saving = ref(false);

  async function load(force = false) {
    if (loaded.value && !force) return;
    settings.value = await api.getSettings();
    loaded.value = true;
  }

  let saveTimer: number | null = null;

  /** Persiste con un pequeno debounce para no escribir en cada tecla. */
  function scheduleSave() {
    if (!settings.value) return;
    if (saveTimer !== null) clearTimeout(saveTimer);
    saveTimer = window.setTimeout(async () => {
      if (!settings.value) return;
      saving.value = true;
      try {
        await api.saveSettings(settings.value);
      } finally {
        saving.value = false;
      }
    }, 350);
  }

  function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    if (settings.value) {
      settings.value[key] = value;
      scheduleSave();
    }
  }

  async function save() {
    if (!settings.value) return;
    saving.value = true;
    try {
      await api.saveSettings(settings.value);
    } finally {
      saving.value = false;
    }
  }

  return { settings, loaded, saving, load, update, save };
});
