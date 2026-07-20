import { defineStore } from "pinia";
import { ref } from "vue";
import type { FavoriteServer } from "@/lib/types";
import { api } from "@/lib/ipc";
import { useAppStore } from "@/stores/app";

export const useFavoritesStore = defineStore("favorites", () => {
  const servers = ref<FavoriteServer[]>([]);
  const loaded = ref(false);
  const joinBusyId = ref<string | null>(null);

  async function load(force = false) {
    if (loaded.value && !force) return;
    servers.value = await api.listFavoriteServers();
    loaded.value = true;
  }

  async function add(
    name: string,
    address: string,
    notes?: string,
    loaderHint?: "modern" | "189",
    fromPlayit?: boolean,
    bedrockPort?: number,
  ) {
    const fav = await api.addFavoriteServer(name, address, notes ?? null, loaderHint ?? null, fromPlayit, bedrockPort);
    servers.value = [...servers.value, fav];
    return fav;
  }

  async function addFromServer(serverId: string, bedrockPort?: number) {
    const fav = await api.addFavoriteFromServer(serverId, bedrockPort ?? null);
    servers.value = [...servers.value, fav];
    return fav;
  }

  async function remove(id: string) {
    await api.removeFavoriteServer(id);
    servers.value = servers.value.filter((s) => s.id !== id);
  }

  /** Join inteligente: infiere perfil (1.8.9 vs 1.21.11) en el backend y lanza. */
  async function join(fav: FavoriteServer) {
    const app = useAppStore();
    joinBusyId.value = fav.id;
    try {
      await app.initGameEvents();
      app.setLaunch("preparing", `Unirse — ${fav.name}…`);
      await api.joinFavoriteServer(fav.id);
    } catch (e) {
      app.setLaunch("idle", "Listo para jugar");
      throw e;
    } finally {
      joinBusyId.value = null;
    }
  }

  /** Abre Minecraft Bedrock (UWP) para un favorito con Geyser; devuelve host:puerto a copiar. */
  async function joinBedrock(fav: FavoriteServer) {
    joinBusyId.value = fav.id;
    try {
      return await api.joinFavoriteBedrock(fav.id);
    } finally {
      joinBusyId.value = null;
    }
  }

  return { servers, loaded, joinBusyId, load, add, addFromServer, remove, join, joinBedrock };
});
