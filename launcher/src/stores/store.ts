import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import { api } from "@/lib/ipc";
import type { ContentProvider, ContentType, StoreItem } from "@/lib/types";

const PAGE_LIMIT = 40;

interface SearchState {
  items: StoreItem[];
  offset: number;
  totalHits: number;
  query: string;
  loaded: boolean;
}

function emptyState(): SearchState {
  return { items: [], offset: 0, totalHits: 0, query: "", loaded: false };
}

/**
 * Cache de la tienda (Pinia) para que cambiar de pestaña (Tienda ↔ Ajustes ↔ Skins)
 * no dispare de nuevo la búsqueda ni muestre pantallas de carga: al volver, la UI
 * lee esto al instante. Se combina con `<KeepAlive>` en `MainLayout` para que el
 * propio `StoreView` tampoco se destruya, pero esta cache sobrevive incluso si el
 * componente llegara a desmontarse (límite de `KeepAlive`, recarga de la app, etc).
 */
export const useStoreStore = defineStore("store", () => {
  /** Clave = provider|projectType, cada tab de tipo de contenido tiene su propia página. */
  const searches = reactive<Record<string, SearchState>>({});
  const loading = ref(false);
  const error = ref<string | null>(null);

  function keyFor(provider: ContentProvider, projectType: ContentType) {
    return `${provider}|${projectType}`;
  }

  function stateFor(provider: ContentProvider, projectType: ContentType): SearchState {
    const key = keyFor(provider, projectType);
    if (!searches[key]) searches[key] = emptyState();
    return searches[key];
  }

  async function search(opts: {
    provider: ContentProvider;
    projectType: ContentType;
    query: string;
    offset?: number;
    /** Fuerza refetch aunque ya haya datos cacheados para esta clave. */
    force?: boolean;
  }): Promise<void> {
    const key = keyFor(opts.provider, opts.projectType);
    const state = stateFor(opts.provider, opts.projectType);
    const offset = opts.offset ?? 0;
    const sameQuery = state.query === opts.query && state.offset === offset;
    if (!opts.force && state.loaded && sameQuery) return;

    loading.value = true;
    error.value = null;
    try {
      const res = await api.searchStore({
        provider: opts.provider,
        query: opts.query,
        projectType: opts.projectType,
        offset,
        limit: PAGE_LIMIT,
      });
      searches[key] = {
        items: res.items,
        offset: res.offset,
        totalHits: res.totalHits,
        query: opts.query,
        loaded: true,
      };
    } catch (e) {
      error.value = String(e);
      searches[key] = { ...state, items: [], loaded: true };
    } finally {
      loading.value = false;
    }
  }

  function invalidate(provider?: ContentProvider, projectType?: ContentType) {
    if (provider && projectType) {
      delete searches[keyFor(provider, projectType)];
      return;
    }
    for (const k of Object.keys(searches)) delete searches[k];
  }

  return { searches, loading, error, limit: PAGE_LIMIT, stateFor, search, invalidate };
});
