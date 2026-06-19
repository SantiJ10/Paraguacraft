import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Instance } from "@/lib/types";
import { api } from "@/lib/ipc";

export const useInstancesStore = defineStore("instances", () => {
  const instances = ref<Instance[]>([]);
  const loaded = ref(false);
  const selectedId = ref<string | null>(null);

  const selected = computed(
    () => instances.value.find((i) => i.id === selectedId.value) ?? instances.value[0] ?? null,
  );

  const recent = computed(() =>
    [...instances.value]
      .filter((i) => i.lastPlayed)
      .sort((a, b) => (b.lastPlayed ?? "").localeCompare(a.lastPlayed ?? "")),
  );

  async function load() {
    if (loaded.value) return;
    instances.value = await api.getInstances();
    selectedId.value = recent.value[0]?.id ?? instances.value[0]?.id ?? null;
    loaded.value = true;
  }

  function select(id: string) {
    selectedId.value = id;
  }

  return { instances, loaded, selectedId, selected, recent, load, select };
});
