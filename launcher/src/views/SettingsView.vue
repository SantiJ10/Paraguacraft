<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useAccountsStore } from "@/stores/accounts";
import { useAppStore } from "@/stores/app";
import BaseToggle from "@/components/common/BaseToggle.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import { formatRam } from "@/composables/useFormat";
import type { GcType } from "@/lib/types";

const settings = useSettingsStore();
const accounts = useAccountsStore();
const app = useAppStore();

onMounted(() => {
  settings.load();
  accounts.load();
  app.loadHardware();
});

const maxRam = computed(() => (app.hardware ? app.hardware.ramGb * 1024 : 16384));
const gcOptions: GcType[] = ["Auto", "G1GC", "ZGC", "Shenandoah"];
</script>

<template>
  <div class="mx-auto max-w-3xl p-8">
    <h1 class="mb-6 text-2xl font-bold">Ajustes</h1>

    <template v-if="settings.settings">
      <!-- Rendimiento -->
      <section class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold">
          <span class="font-emoji">&#9889;</span> Rendimiento
        </h2>

        <div class="mb-5">
          <div class="mb-1 flex justify-between text-sm">
            <span class="text-gray-300">Memoria RAM asignada</span>
            <span class="font-bold text-pc-green">{{ formatRam(settings.settings.ramMb) }}</span>
          </div>
          <input
            :value="settings.settings.ramMb"
            type="range"
            :min="2048"
            :max="maxRam"
            :step="512"
            class="w-full accent-pc-green"
            @input="settings.update('ramMb', Number(($event.target as HTMLInputElement).value))"
          />
          <p class="mt-1 text-xs text-gray-500">
            Recomendado para tu hardware: {{ formatRam(app.hardware?.recommendedRamMb ?? 4096) }}
          </p>
        </div>

        <label class="mb-5 block">
          <span class="mb-1 block text-sm text-gray-300">Garbage Collector</span>
          <select
            :value="settings.settings.gcType"
            class="w-full max-w-xs rounded-lg border border-surface-5 bg-surface-3 px-3 py-2.5 text-sm outline-none focus:border-pc-green"
            @change="settings.update('gcType', ($event.target as HTMLSelectElement).value as GcType)"
          >
            <option v-for="gc in gcOptions" :key="gc" :value="gc">{{ gc }}</option>
          </select>
        </label>

        <div class="space-y-4">
          <BaseToggle
            :model-value="settings.settings.optimizeGraphics"
            label="Optimizar graficos"
            hint="Aplica ajustes de bajo consumo automaticamente."
            @update:model-value="settings.update('optimizeGraphics', $event)"
          />
          <BaseToggle
            :model-value="settings.settings.closeOnLaunch"
            label="Cerrar launcher al jugar"
            hint="Libera RAM y procesos en segundo plano."
            @update:model-value="settings.update('closeOnLaunch', $event)"
          />
        </div>
      </section>

      <!-- Cuentas -->
      <section class="mb-6 rounded-xl border border-surface-4 bg-surface-2 p-6">
        <div class="mb-4 flex items-center justify-between">
          <h2 class="flex items-center gap-2 text-lg font-bold"><span class="font-emoji">&#128100;</span> Cuentas</h2>
          <BaseButton size="sm" variant="secondary">+ Agregar cuenta</BaseButton>
        </div>
        <div class="space-y-2">
          <div
            v-for="acc in accounts.accounts"
            :key="acc.id"
            class="flex items-center gap-3 rounded-lg border p-3"
            :class="acc.active ? 'border-pc-green bg-pc-green/5' : 'border-surface-4'"
          >
            <div class="flex h-9 w-9 items-center justify-center rounded-md bg-surface-5 font-bold uppercase">
              {{ acc.username[0] }}
            </div>
            <div class="flex-1">
              <p class="font-semibold">{{ acc.username }}</p>
              <p class="text-xs" :class="acc.premium ? 'text-pc-green' : 'text-gray-500'">
                {{ acc.premium ? "Microsoft Premium" : "Offline" }}
              </p>
            </div>
            <BaseButton v-if="!acc.active" size="sm" variant="ghost" @click="accounts.setActive(acc.id)">
              Usar
            </BaseButton>
            <span v-else class="text-xs font-bold text-pc-green">Activa</span>
          </div>
        </div>
      </section>

      <!-- Apariencia -->
      <section class="rounded-xl border border-surface-4 bg-surface-2 p-6">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold"><span class="font-emoji">&#127912;</span> Apariencia</h2>
        <p class="mb-2 text-sm text-gray-300">Color de acento</p>
        <div class="flex gap-3">
          <button
            class="h-9 w-9 rounded-full ring-2 ring-offset-2 ring-offset-surface-2"
            style="background: #2ecc71"
            :class="settings.settings.accent === 'green' ? 'ring-pc-green' : 'ring-transparent'"
            @click="settings.update('accent', 'green')"
          />
          <button
            class="h-9 w-9 rounded-full ring-2 ring-offset-2 ring-offset-surface-2"
            style="background: #9b59b6"
            :class="settings.settings.accent === 'ai' ? 'ring-pc-ai' : 'ring-transparent'"
            @click="settings.update('accent', 'ai')"
          />
        </div>
      </section>
    </template>
  </div>
</template>
