<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import { useAppStore } from "@/stores/app";
import { useWizardStore } from "@/stores/wizard";
import { recommendForHardware, tierLabel } from "@/composables/useHardware";
import { formatRam } from "@/composables/useFormat";

const app = useAppStore();
const wizard = useWizardStore();

const rec = computed(() => (app.hardware ? recommendForHardware(app.hardware) : null));

onMounted(async () => {
  await app.loadHardware();
});

watch(
  rec,
  (r) => {
    if (r) {
      wizard.data.perfil = r.tier;
      wizard.data.ramMb = r.ramMb;
    }
  },
  { immediate: true },
);

const maxRam = computed(() => (app.hardware ? app.hardware.ramGb * 1024 : 16384));
</script>

<template>
  <div class="mx-auto max-w-xl">
    <h2 class="text-2xl font-bold">Detectamos tu hardware</h2>
    <p class="mt-1 text-gray-400">Autoconfiguramos RAM y Garbage Collector segun tu equipo.</p>

    <div v-if="app.hardware" class="mt-6 grid grid-cols-2 gap-3">
      <div class="rounded-xl border border-surface-4 bg-surface-2 p-4">
        <p class="text-xs uppercase text-gray-500">Procesador</p>
        <p class="mt-1 font-semibold">{{ app.hardware.cpuName }}</p>
        <p class="text-xs text-gray-500">{{ app.hardware.cpuCores }} nucleos / {{ app.hardware.cpuThreads }} hilos</p>
      </div>
      <div class="rounded-xl border border-surface-4 bg-surface-2 p-4">
        <p class="text-xs uppercase text-gray-500">Grafica</p>
        <p class="mt-1 font-semibold">{{ app.hardware.gpuName }}</p>
        <p class="text-xs text-gray-500">{{ app.hardware.ramGb }} GB RAM total</p>
      </div>
    </div>

    <div v-if="rec" class="mt-4 rounded-xl border border-pc-green/40 bg-pc-green/10 p-4">
      <div class="flex items-center justify-between">
        <p class="font-semibold text-pc-green">Perfil sugerido: {{ tierLabel(rec.tier) }}</p>
        <span class="rounded bg-pc-green/20 px-2 py-0.5 text-xs font-bold text-pc-green">{{ rec.gc }}</span>
      </div>

      <div class="mt-4">
        <div class="mb-1 flex justify-between text-sm">
          <span class="text-gray-300">RAM asignada</span>
          <span class="font-bold text-white">{{ formatRam(wizard.data.ramMb) }}</span>
        </div>
        <input
          v-model.number="wizard.data.ramMb"
          type="range"
          :min="2048"
          :max="maxRam"
          :step="512"
          class="w-full accent-pc-green"
        />
        <div class="flex justify-between text-xs text-gray-600">
          <span>2 GB</span>
          <span>{{ formatRam(maxRam) }}</span>
        </div>
      </div>
    </div>

    <p v-else class="mt-6 text-sm text-gray-500">Analizando hardware...</p>
  </div>
</template>
