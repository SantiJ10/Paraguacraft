import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { RunningServer, ServerProfile, ServerStatus } from "@/lib/types";
import { api, isTauri } from "@/lib/ipc";

const LAST_ACTIVE_KEY = "pc.servers.lastActiveId";

function readLastActive(): string | null {
  try {
    return localStorage.getItem(LAST_ACTIVE_KEY);
  } catch {
    return null;
  }
}

export const useServersStore = defineStore("servers", () => {
  const servers = ref<ServerProfile[]>([]);
  const loaded = ref(false);
  /** Último detail abierto: Servidores del sidebar vuelve acá (con consola en KeepAlive). */
  const lastActiveId = ref<string | null>(readLastActive());
  const running = ref<RunningServer[]>([]);
  const stoppingId = ref<string | null>(null);

  const hasRunning = computed(() => running.value.length > 0);
  const primaryRunning = computed(() => running.value[0] ?? null);

  let runningWatchBound = false;

  async function refreshRunning() {
    if (!isTauri()) {
      running.value = [];
      return;
    }
    try {
      running.value = await api.listRunningServers();
    } catch {
      /* ignore */
    }
  }

  async function watchRunning() {
    if (runningWatchBound || !isTauri()) return;
    runningWatchBound = true;
    const { listen } = await import("@tauri-apps/api/event");
    await listen<{ servers?: RunningServer[] }>("server://running", (ev) => {
      running.value = ev.payload?.servers ?? [];
    });
    await refreshRunning();
  }

  async function stopRunning(id?: string) {
    const target = id ?? primaryRunning.value?.id;
    if (!target || stoppingId.value) return;
    stoppingId.value = target;
    try {
      await api.stopServer(target);
    } finally {
      stoppingId.value = null;
      await refreshRunning();
    }
  }

  async function load(force = false) {
    if (loaded.value && !force) return;
    servers.value = await api.listServers();
    loaded.value = true;
    // Si el id recordado ya no existe, limpiar
    if (lastActiveId.value && !servers.value.some((s) => s.id === lastActiveId.value)) {
      setLastActive(null);
    }
  }

  function setLastActive(id: string | null) {
    lastActiveId.value = id;
    try {
      if (id) localStorage.setItem(LAST_ACTIVE_KEY, id);
      else localStorage.removeItem(LAST_ACTIVE_KEY);
    } catch {
      /* ignore */
    }
  }

  function upsert(s: ServerProfile) {
    const idx = servers.value.findIndex((x) => x.id === s.id);
    if (idx >= 0) servers.value[idx] = s;
    else servers.value.push(s);
  }

  async function create(payload: {
    name: string;
    mcVersion: string;
    serverType: string;
    ramMb: number;
  }) {
    const s = await api.createServer(payload);
    upsert(s);
    setLastActive(s.id);
    return s;
  }

  async function remove(id: string) {
    await api.deleteServer(id);
    servers.value = servers.value.filter((s) => s.id !== id);
    if (lastActiveId.value === id) setLastActive(null);
  }

  async function status(id: string): Promise<ServerStatus> {
    return api.serverStatus(id);
  }

  async function importFolder(path: string, name?: string) {
    const s = await api.importServerFolder(path, name);
    upsert(s);
    setLastActive(s.id);
    return s;
  }

  async function update(payload: {
    id: string;
    name?: string;
    mcVersion?: string;
    ramMb?: number;
    port?: number;
  }) {
    const s = await api.updateServer(payload);
    upsert(s);
    return s;
  }

  return {
    servers,
    loaded,
    lastActiveId,
    running,
    stoppingId,
    hasRunning,
    primaryRunning,
    setLastActive,
    load,
    upsert,
    create,
    update,
    remove,
    status,
    importFolder,
    watchRunning,
    refreshRunning,
    stopRunning,
  };
});
