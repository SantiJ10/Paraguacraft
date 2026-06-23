import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { BackupInfo, Instance } from "@/lib/types";
import { api } from "@/lib/ipc";

export const useInstancesStore = defineStore("instances", () => {
  const instances = ref<Instance[]>([]);
  const loaded = ref(false);
  const scanning = ref(false);
  const selectedId = ref<string | null>(null);

  const selected = computed(
    () => instances.value.find((i) => i.id === selectedId.value) ?? instances.value[0] ?? null,
  );

  const recent = computed(() =>
    [...instances.value]
      .filter((i) => i.lastPlayed)
      .sort((a, b) => (b.lastPlayed ?? "").localeCompare(a.lastPlayed ?? "")),
  );

  /** Instancias importables (detectadas en otros launchers). */
  const external = computed(() => instances.value.filter((i) => i.id.startsWith("ext::")));
  const local = computed(() => instances.value.filter((i) => !i.id.startsWith("ext::")));

  async function load(force = false) {
    if (loaded.value && !force) return;
    instances.value = await api.getInstances();
    selectedId.value = recent.value[0]?.id ?? instances.value[0]?.id ?? null;
    loaded.value = true;
  }

  /** Escaneo completo (incluye otros launchers). On-demand. */
  async function scan() {
    scanning.value = true;
    try {
      instances.value = await api.scanInstances();
      loaded.value = true;
      if (!selectedId.value) {
        selectedId.value = instances.value[0]?.id ?? null;
      }
    } finally {
      scanning.value = false;
    }
  }

  function select(id: string) {
    selectedId.value = id;
  }

  function upsert(inst: Instance) {
    const idx = instances.value.findIndex((i) => i.id === inst.id);
    if (idx >= 0) instances.value[idx] = inst;
    else instances.value.push(inst);
  }

  async function create(payload: {
    name: string;
    mcVersion: string;
    loader: string;
    loaderVersion: string;
    icon: string;
    ramMb: number;
  }) {
    const inst = await api.createInstance(payload);
    upsert(inst);
    selectedId.value = inst.id;
    return inst;
  }

  async function rename(id: string, name: string, icon: string) {
    upsert(await api.renameInstance(id, name, icon));
  }

  async function setRam(id: string, ramMb: number) {
    upsert(await api.setInstanceRam(id, ramMb));
  }

  async function duplicate(id: string, newName: string) {
    const inst = await api.duplicateInstance(id, newName);
    upsert(inst);
    return inst;
  }

  async function remove(id: string) {
    await api.deleteInstance(id);
    instances.value = instances.value.filter((i) => i.id !== id);
    if (selectedId.value === id) selectedId.value = instances.value[0]?.id ?? null;
  }

  async function importExternal(id: string) {
    const inst = await api.importInstance(id);
    upsert(inst);
    selectedId.value = inst.id;
    return inst;
  }

  async function backups(id: string): Promise<BackupInfo[]> {
    return api.listBackups(id);
  }

  async function createBackup(id: string): Promise<BackupInfo> {
    return api.createBackup(id);
  }

  async function restoreBackup(id: string, name: string) {
    await api.restoreBackup(id, name);
  }

  async function deleteBackup(id: string, name: string) {
    await api.deleteBackup(id, name);
  }

  return {
    instances,
    loaded,
    scanning,
    selectedId,
    selected,
    recent,
    external,
    local,
    load,
    scan,
    select,
    create,
    rename,
    setRam,
    duplicate,
    remove,
    importExternal,
    backups,
    createBackup,
    restoreBackup,
    deleteBackup,
  };
});
