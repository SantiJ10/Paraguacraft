<script setup lang="ts">
import { ref } from "vue";
import { api } from "@/lib/ipc";
import { useInstancesStore } from "@/stores/instances";
import { useDownloadsStore } from "@/stores/downloads";
import BaseButton from "@/components/common/BaseButton.vue";

const emit = defineEmits<{ (e: "close"): void }>();
const instances = useInstancesStore();
const downloads = useDownloadsStore();

const source = ref("");
const mc = ref("");
const busy = ref(false);
const error = ref<string | null>(null);
const message = ref<string | null>(null);

async function submit() {
  if (!source.value.trim()) {
    error.value = "Pegá el slug o la URL del modpack de Modrinth";
    return;
  }
  busy.value = true;
  error.value = null;
  message.value = "Descargando e instalando modpack…";
  try {
    await downloads.initEvents();
    const inst = await api.importMrpack(source.value.trim(), mc.value.trim());
    await instances.load(true);
    instances.select(inst.id);
    message.value = `${inst.name} importado.`;
    setTimeout(() => emit("close"), 600);
  } catch (e) {
    error.value = String(e);
    message.value = null;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" @click.self="emit('close')">
    <div class="w-full max-w-lg rounded-2xl border border-surface-4 bg-surface-2 p-6 shadow-2xl">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-lg font-bold">Importar modpack (.mrpack)</h3>
        <button class="text-gray-500 hover:text-white" @click="emit('close')">&times;</button>
      </div>

      <div class="space-y-4">
        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Modpack de Modrinth</span>
          <input
            v-model="source"
            type="text"
            placeholder="fabulously-optimized  o  https://modrinth.com/modpack/…"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          />
        </label>

        <label class="block">
          <span class="mb-1 block text-sm text-gray-300">Versión de Minecraft (opcional)</span>
          <input
            v-model="mc"
            type="text"
            placeholder="Última compatible"
            class="w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
          />
        </label>

        <p v-if="message" class="text-sm text-pc-green">{{ message }}</p>
        <p v-if="error" class="text-sm text-red-400">{{ error }}</p>

        <div class="flex justify-end gap-2 pt-2">
          <BaseButton variant="ghost" @click="emit('close')">Cancelar</BaseButton>
          <BaseButton :disabled="busy" @click="submit">{{ busy ? "Importando…" : "Importar" }}</BaseButton>
        </div>
      </div>
    </div>
  </div>
</template>
