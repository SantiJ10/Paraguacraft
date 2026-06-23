import { defineStore } from "pinia";
import { ref } from "vue";
import type { FavoriteServer } from "@/lib/types";
import { api } from "@/lib/ipc";

export const useFavoritesStore = defineStore("favorites", () => {
  const servers = ref<FavoriteServer[]>([]);
  const loaded = ref(false);

  async function load(force = false) {
    if (loaded.value && !force) return;
    servers.value = await api.listFavoriteServers();
    loaded.value = true;
  }

  async function add(name: string, address: string, notes?: string) {
    const fav = await api.addFavoriteServer(name, address, notes ?? null);
    servers.value = [...servers.value, fav];
    return fav;
  }

  async function remove(id: string) {
    await api.removeFavoriteServer(id);
    servers.value = servers.value.filter((s) => s.id !== id);
  }

  return { servers, loaded, load, add, remove };
});
