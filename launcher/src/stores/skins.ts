import { defineStore } from "pinia";
import { ref } from "vue";
import type { SkinProfile } from "@/lib/types";
import { withCacheBust } from "@/lib/skins";
import { api } from "@/lib/ipc";
import { useAccountsStore } from "@/stores/accounts";

const REFRESH_TTL_MS = 12_000;
const WATCH_MS = 30_000;

function bustProfile(p: SkinProfile): SkinProfile {
  const t = Date.now();
  return {
    ...p,
    avatarUrl: withCacheBust(p.avatarUrl, t) || p.avatarUrl,
    bodyUrl: withCacheBust(p.bodyUrl, t) || p.bodyUrl,
    skinUrl: p.skinUrl ? withCacheBust(p.skinUrl, t) || p.skinUrl : p.skinUrl,
  };
}

/** Avatar activo en sidebar/ajustes — refrescar tras aplicar skin. */
export const useSkinsStore = defineStore("skins", () => {
  const activeSkin = ref<SkinProfile | null>(null);
  const revision = ref(0);
  const loading = ref(false);

  let inflight: Promise<void> | null = null;
  let lastRefresh = 0;
  let watchTimer: number | null = null;
  let watching = false;

  async function refresh(force = false) {
    if (!force && activeSkin.value && Date.now() - lastRefresh < REFRESH_TTL_MS) {
      return;
    }
    if (inflight) return inflight;

    inflight = (async () => {
      const hadSkin = Boolean(activeSkin.value);
      if (!hadSkin) loading.value = true;
      try {
        const next = bustProfile(await api.getActiveSkin(force));
        activeSkin.value = next;
        revision.value += 1;
        lastRefresh = Date.now();
        const accounts = useAccountsStore();
        if (
          accounts.active?.premium &&
          next.username &&
          next.username !== accounts.active.username
        ) {
          void accounts.load(true);
        }
      } catch {
        if (!activeSkin.value) {
          try {
            activeSkin.value = bustProfile(await api.getActiveSkinLocal());
            revision.value += 1;
          } catch {
            activeSkin.value = null;
          }
        }
      } finally {
        loading.value = false;
      }
    })().finally(() => {
      inflight = null;
    });

    return inflight;
  }

  function startWatch() {
    if (watching || typeof window === "undefined") return;
    watching = true;
    const tick = () => {
      void refresh(true);
    };
    watchTimer = window.setInterval(tick, WATCH_MS);
    const onVis = () => {
      if (document.visibilityState === "visible") tick();
    };
    window.addEventListener("focus", tick);
    document.addEventListener("visibilitychange", onVis);
  }

  function stopWatch() {
    watching = false;
    if (watchTimer !== null) {
      window.clearInterval(watchTimer);
      watchTimer = null;
    }
  }

  return { activeSkin, revision, loading, refresh, startWatch, stopWatch };
});
