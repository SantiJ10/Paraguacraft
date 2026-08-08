import { defineStore } from "pinia";
import { ref } from "vue";
import type { ServerProfile, ServerStatus } from "@/lib/types";
import { api } from "@/lib/ipc";

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
    setLastActive,
    load,
    upsert,
    create,
    update,
    remove,
    status,
    importFolder,
  };
});
