import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { HardwareTier } from "@/lib/types";

export interface WizardData {
  perfil: HardwareTier;
  ramMb: number;
  accountType: "microsoft" | "offline" | null;
  offlineName: string;
  accent: "green" | "ai";
}

export const useWizardStore = defineStore("wizard", () => {
  const totalSteps = 4;
  const step = ref(0);
  const direction = ref<"fwd" | "back">("fwd");

  const data = ref<WizardData>({
    perfil: "media",
    ramMb: 4096,
    accountType: null,
    offlineName: "",
    accent: "green",
  });

  const progress = computed(() => Math.round(((step.value + 1) / totalSteps) * 100));
  const isLast = computed(() => step.value === totalSteps - 1);

  function next() {
    if (step.value < totalSteps - 1) {
      direction.value = "fwd";
      step.value++;
    }
  }
  function prev() {
    if (step.value > 0) {
      direction.value = "back";
      step.value--;
    }
  }
  function goTo(i: number) {
    direction.value = i > step.value ? "fwd" : "back";
    step.value = i;
  }
  function complete() {
    localStorage.setItem("pc.onboarding.done", "1");
  }

  return { totalSteps, step, direction, data, progress, isLast, next, prev, goTo, complete };
});
