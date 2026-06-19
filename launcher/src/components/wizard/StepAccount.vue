<script setup lang="ts">
import { useWizardStore } from "@/stores/wizard";

const wizard = useWizardStore();
</script>

<template>
  <div class="mx-auto max-w-xl">
    <h2 class="text-2xl font-bold">Inicia sesion</h2>
    <p class="mt-1 text-gray-400">Soportamos cuentas Microsoft Premium y cuentas offline (No-Premium).</p>

    <div class="mt-6 space-y-3">
      <button
        class="flex w-full items-center gap-4 rounded-xl border-2 p-4 text-left transition-all"
        :class="
          wizard.data.accountType === 'microsoft'
            ? 'border-pc-green bg-pc-green/10'
            : 'border-surface-4 bg-surface-2 hover:border-surface-6'
        "
        @click="wizard.data.accountType = 'microsoft'"
      >
        <svg class="h-7 w-7 shrink-0" viewBox="0 0 23 23">
          <rect x="1" y="1" width="10" height="10" fill="#f25022" />
          <rect x="12" y="1" width="10" height="10" fill="#7fba00" />
          <rect x="1" y="12" width="10" height="10" fill="#00a4ef" />
          <rect x="12" y="12" width="10" height="10" fill="#ffb900" />
        </svg>
        <div>
          <p class="font-semibold">Cuenta Microsoft (Premium)</p>
          <p class="text-xs text-gray-500">Skins, servidores online y sesion persistente.</p>
        </div>
      </button>

      <button
        class="w-full rounded-xl border-2 p-4 text-left transition-all"
        :class="
          wizard.data.accountType === 'offline'
            ? 'border-pc-green bg-pc-green/10'
            : 'border-surface-4 bg-surface-2 hover:border-surface-6'
        "
        @click="wizard.data.accountType = 'offline'"
      >
        <p class="font-semibold">Cuenta offline (No-Premium)</p>
        <p class="text-xs text-gray-500">Juga sin conexion o en servidores crackeados.</p>
        <input
          v-if="wizard.data.accountType === 'offline'"
          v-model="wizard.data.offlineName"
          placeholder="Nombre de usuario"
          class="mt-3 w-full rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-green"
          @click.stop
        />
      </button>
    </div>

    <p class="mt-4 text-center text-xs text-gray-600">
      El login real con Microsoft se conecta en la Fase 2.
    </p>
  </div>
</template>
