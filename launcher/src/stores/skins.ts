import { defineStore } from "pinia";
import { ref } from "vue";
import type { SkinProfile } from "@/lib/types";
import { api } from "@/lib/ipc";

/** Avatar activo en sidebar/ajustes — refrescar tras aplicar skin. */
export const useSkinsStore = defineStore("skins", () => {
  const activeSkin = ref<SkinProfile | null>(null);
  const revision = ref(0);
  const loading = ref(false);

  async function refresh() {
    loading.value = true;
    try {
      activeSkin.value = await api.getActiveSkin();
      revision.value += 1;
    } catch {
      activeSkin.value = null;
    } finally {
      loading.value = false;
    }
  }

  return { activeSkin, revision, loading, refresh };
});
