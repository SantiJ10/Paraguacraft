import { defineStore } from "pinia";
import { ref } from "vue";
import type { SkinProfile } from "@/lib/types";
import { api } from "@/lib/ipc";

const REFRESH_TTL_MS = 60_000;

/** Avatar activo en sidebar/ajustes — refrescar tras aplicar skin. */
export const useSkinsStore = defineStore("skins", () => {
  const activeSkin = ref<SkinProfile | null>(null);
  const revision = ref(0);
  const loading = ref(false);

  let inflight: Promise<void> | null = null;
  let lastRefresh = 0;

  async function refresh(force = false) {
    if (!force && activeSkin.value && Date.now() - lastRefresh < REFRESH_TTL_MS) {
      return;
    }
    if (inflight) return inflight;

    inflight = (async () => {
      loading.value = true;
      try {
        if (!activeSkin.value) {
          try {
            activeSkin.value = await api.getActiveSkinLocal();
            revision.value += 1;
          } catch {
            /* fallback al enrich completo */
          }
        }
        activeSkin.value = await api.getActiveSkin(force);
        revision.value += 1;
        lastRefresh = Date.now();
      } catch {
        if (!activeSkin.value) activeSkin.value = null;
      } finally {
        loading.value = false;
      }
    })().finally(() => {
      inflight = null;
    });

    return inflight;
  }

  return { activeSkin, revision, loading, refresh };
});
