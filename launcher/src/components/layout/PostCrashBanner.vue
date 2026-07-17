<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import { useAiStore } from "@/stores/ai";
import BaseButton from "@/components/common/BaseButton.vue";

const app = useAppStore();
const ai = useAiStore();
const router = useRouter();

const crash = computed(() => app.lastCrash);

function dismiss() {
  app.dismissCrash();
}

function openInstance() {
  const id = crash.value?.instanceId;
  if (!id) return;
  dismiss();
  router.push({ name: "instance-detail", params: { id } });
}

function askBot() {
  const c = crash.value;
  if (!c?.diagnosis) return;
  ai.pushDiagnosis(c.diagnosis, c.instanceId);
  dismiss();
}
</script>

<template>
  <div
    v-if="crash"
    class="border-b border-red-500/30 bg-red-950/40 px-4 py-3"
    role="alert"
  >
    <div class="mx-auto flex max-w-5xl flex-wrap items-start justify-between gap-3">
      <div class="min-w-0 flex-1">
        <p class="text-sm font-bold text-red-200">El juego terminó con error</p>
        <p class="mt-1 text-sm text-red-100/90">{{ crash.diagnosis.message }}</p>
        <p class="mt-1 text-xs text-gray-400">{{ crash.diagnosis.hint }}</p>
        <ul v-if="crash.diagnosis.suggestions.length" class="mt-2 space-y-0.5 text-xs text-gray-300">
          <li v-for="(s, i) in crash.diagnosis.suggestions.slice(0, 3)" :key="i">• {{ s }}</li>
        </ul>
      </div>
      <div class="flex flex-wrap gap-2">
        <BaseButton size="sm" variant="secondary" @click="openInstance">Ver instancia</BaseButton>
        <BaseButton size="sm" variant="secondary" @click="askBot">Paraguabot</BaseButton>
        <BaseButton size="sm" @click="dismiss">Cerrar</BaseButton>
      </div>
    </div>
  </div>
</template>
