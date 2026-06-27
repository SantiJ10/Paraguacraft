import { defineStore } from "pinia";
import { ref } from "vue";
import type { ServerProfile, ServerStatus } from "@/lib/types";
import { api } from "@/lib/ipc";

export const useServersStore = defineStore("servers", () => {
  const servers = ref<ServerProfile[]>([]);
  const loaded = ref(false);

  async function load(force = false) {
    if (loaded.value && !force) return;
    servers.value = await api.listServers();
    loaded.value = true;
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
    return s;
  }

  async function remove(id: string) {
    await api.deleteServer(id);
    servers.value = servers.value.filter((s) => s.id !== id);
  }

  async function status(id: string): Promise<ServerStatus> {
    return api.serverStatus(id);
  }

  async function importFolder(path: string, name?: string) {
    const s = await api.importServerFolder(path, name);
    upsert(s);
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

  return { servers, loaded, load, upsert, create, update, remove, status, importFolder };
});
