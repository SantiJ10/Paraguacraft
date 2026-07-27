<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import { useAiStore } from "@/stores/ai";
import { useSettingsStore } from "@/stores/settings";
import { api } from "@/lib/ipc";
import BaseButton from "@/components/common/BaseButton.vue";

const app = useAppStore();
const ai = useAiStore();
const settings = useSettingsStore();
const router = useRouter();

const crash = computed(() => app.lastCrash);
const busy = ref(false);
const actionMsg = ref<string | null>(null);

const category = computed(() => crash.value?.diagnosis.category ?? "");

const showRepair = computed(() =>
  ["launch_early", "corrupt_jar", "mod_crash", "mixin", "generic", "native"].includes(category.value),
);
const showLowPerf = computed(() =>
  ["oom_java", "oom_reserve", "gpu", "opengl", "launch_early"].includes(category.value) ||
  category.value.includes("memory") ||
  category.value.includes("native"),
);
const showHealth = computed(() => !!crash.value?.instanceId);

function dismiss() {
  app.dismissCrash();
  actionMsg.value = null;
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

async function repairNow() {
  const id = crash.value?.instanceId;
  if (!id || id.startsWith("ext::")) return;
  busy.value = true;
  actionMsg.value = null;
  try {
    const report = await api.repairInstance(id);
    actionMsg.value =
      report.fixedCount > 0
        ? `Reparación: ${report.fixedCount} corrección(es). Probá jugar de nuevo.`
        : "Reparación lista. Revisá la instancia si sigue fallando.";
  } catch (e) {
    actionMsg.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function applyLowProfile() {
  busy.value = true;
  actionMsg.value = null;
  try {
    await settings.load();
    settings.update("performanceTier", "baja");
    settings.update("usagePreset", "lightweight");
    settings.update("optimizeGraphics", true);
    settings.update("papaMode", true);
    const id = crash.value?.instanceId;
    if (id && !id.startsWith("ext::")) {
      await api.setInstanceConfig({
        id,
        performanceTier: "baja",
      });
    }
    actionMsg.value = "Perfil Baja + gráficos bajos aplicados. Probá jugar de nuevo.";
  } catch (e) {
    actionMsg.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function openHealth() {
  const id = crash.value?.instanceId;
  if (!id) return;
  dismiss();
  await router.push({ name: "instance-detail", params: { id }, query: { health: "1" } });
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
        <p v-if="actionMsg" class="mt-2 text-xs text-pc-green">{{ actionMsg }}</p>
      </div>
      <div class="flex flex-wrap gap-2">
        <BaseButton
          v-if="showRepair"
          size="sm"
          :disabled="busy"
          @click="repairNow"
        >
          {{ busy ? "…" : "Reparar" }}
        </BaseButton>
        <BaseButton
          v-if="showLowPerf"
          size="sm"
          variant="secondary"
          :disabled="busy"
          @click="applyLowProfile"
        >
          Perfil Baja
        </BaseButton>
        <BaseButton
          v-if="showHealth"
          size="sm"
          variant="secondary"
          @click="openHealth"
        >
          Salud
        </BaseButton>
        <BaseButton size="sm" variant="secondary" @click="openInstance">Ver instancia</BaseButton>
        <BaseButton size="sm" variant="secondary" @click="askBot">Paraguabot</BaseButton>
        <BaseButton size="sm" @click="dismiss">Cerrar</BaseButton>
      </div>
    </div>
  </div>
</template>
