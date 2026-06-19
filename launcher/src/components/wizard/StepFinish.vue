<script setup lang="ts">
import { useWizardStore } from "@/stores/wizard";
import { formatRam } from "@/composables/useFormat";
import { tierLabel } from "@/composables/useHardware";

const wizard = useWizardStore();
</script>

<template>
  <div class="mx-auto max-w-md text-center">
    <div class="mx-auto mb-5 flex h-16 w-16 items-center justify-center rounded-full bg-pc-green/15">
      <svg class="h-8 w-8 text-pc-green" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M20 6L9 17l-5-5" />
      </svg>
    </div>
    <h2 class="text-2xl font-bold">Todo listo</h2>
    <p class="mt-1 text-gray-400">Revisa tu configuracion y entra al ecosistema Paraguacraft.</p>

    <div class="mt-6 space-y-2 rounded-xl border border-surface-4 bg-surface-2 p-4 text-left text-sm">
      <div class="flex justify-between">
        <span class="text-gray-500">Perfil</span>
        <span class="font-semibold">{{ tierLabel(wizard.data.perfil) }}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-gray-500">RAM asignada</span>
        <span class="font-semibold">{{ formatRam(wizard.data.ramMb) }}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-gray-500">Cuenta</span>
        <span class="font-semibold capitalize">
          {{ wizard.data.accountType === "microsoft" ? "Microsoft Premium" : wizard.data.accountType === "offline" ? `Offline (${wizard.data.offlineName || "sin nombre"})` : "Sin elegir" }}
        </span>
      </div>
    </div>

    <div class="mt-6">
      <p class="mb-2 text-sm font-medium text-gray-300">Color de acento</p>
      <div class="flex justify-center gap-3">
        <button
          class="h-9 w-9 rounded-full ring-2 ring-offset-2 ring-offset-surface-1 transition"
          style="background: #2ecc71"
          :class="wizard.data.accent === 'green' ? 'ring-pc-green' : 'ring-transparent'"
          @click="wizard.data.accent = 'green'"
        />
        <button
          class="h-9 w-9 rounded-full ring-2 ring-offset-2 ring-offset-surface-1 transition"
          style="background: #9b59b6"
          :class="wizard.data.accent === 'ai' ? 'ring-pc-ai' : 'ring-transparent'"
          @click="wizard.data.accent = 'ai'"
        />
      </div>
    </div>
  </div>
</template>
