<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { useWizardStore } from "@/stores/wizard";
import { useAccountsStore } from "@/stores/accounts";
import { useSettingsStore } from "@/stores/settings";
import BaseButton from "@/components/common/BaseButton.vue";
import StepWelcome from "@/components/wizard/StepWelcome.vue";
import StepHardware from "@/components/wizard/StepHardware.vue";
import StepAccount from "@/components/wizard/StepAccount.vue";
import StepFinish from "@/components/wizard/StepFinish.vue";

const router = useRouter();
const wizard = useWizardStore();
const accounts = useAccountsStore();
const settings = useSettingsStore();

const steps = [StepWelcome, StepHardware, StepAccount, StepFinish];
const current = computed(() => steps[wizard.step]);
const animClass = computed(() => (wizard.direction === "fwd" ? "animate-wiz-fwd" : "animate-wiz-back"));

const canAdvance = computed(() => {
  if (wizard.step === 2) {
    if (wizard.data.accountType === "offline") return wizard.data.offlineName.trim().length >= 3;
    return wizard.data.accountType !== null;
  }
  return true;
});

async function finish() {
  await settings.load();
  if (settings.settings) {
    settings.update("ramMb", wizard.data.ramMb);
    settings.update("accent", wizard.data.accent);
  }
  if (wizard.data.accountType === "offline" && wizard.data.offlineName.trim()) {
    await accounts.load();
    await accounts.addOffline(wizard.data.offlineName.trim());
  }
  await settings.save();
  wizard.complete();
  router.push({ name: "home" });
}

function onNext() {
  if (wizard.isLast) finish();
  else wizard.next();
}
</script>

<template>
  <div class="flex flex-1 flex-col bg-surface-1">
    <!-- Banner / progreso -->
    <div class="border-b border-surface-3 px-8 py-4">
      <div class="mx-auto flex max-w-2xl items-center gap-2">
        <div
          v-for="i in wizard.totalSteps"
          :key="i"
          class="h-1.5 flex-1 rounded-full transition-colors"
          :class="i - 1 <= wizard.step ? 'bg-pc-green' : 'bg-surface-4'"
        />
      </div>
    </div>

    <!-- Paso actual -->
    <div class="flex flex-1 items-center overflow-y-auto px-8 py-8">
      <div class="w-full" :class="animClass" :key="wizard.step">
        <component :is="current" />
      </div>
    </div>

    <!-- Navegacion -->
    <div class="flex items-center justify-between border-t border-surface-3 px-8 py-4">
      <BaseButton variant="ghost" :disabled="wizard.step === 0" @click="wizard.prev()">
        Atras
      </BaseButton>
      <span class="text-xs text-gray-500">Paso {{ wizard.step + 1 }} de {{ wizard.totalSteps }}</span>
      <BaseButton :disabled="!canAdvance" @click="onNext">
        {{ wizard.isLast ? "Entrar al launcher" : "Continuar" }}
      </BaseButton>
    </div>
  </div>
</template>
