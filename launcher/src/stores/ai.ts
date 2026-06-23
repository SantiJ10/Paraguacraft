import { defineStore } from "pinia";
import { ref } from "vue";
import type { AiMessage, CrashDiagnosis } from "@/lib/types";
import { api } from "@/lib/ipc";

export const useAiStore = defineStore("ai", () => {
  const open = ref(false);
  const lastDiagnosis = ref<CrashDiagnosis | null>(null);
  const lastCrashInstanceId = ref<string | null>(null);
  const messages = ref<AiMessage[]>([
    {
      id: "sys-1",
      role: "assistant",
      content:
        "Hola, soy Paraguabot. Puedo ayudarte a elegir versiones, diagnosticar crashes y optimizar tu juego.",
    },
  ]);
  const thinking = ref(false);

  function toggle() {
    open.value = !open.value;
  }

  function formatCrashMessage(d: CrashDiagnosis): string {
    const parts = [d.message, "", d.hint];
    if (d.errorLine) {
      parts.push("", "Detalle del log:", d.errorLine);
    }
    if (d.crashFile) {
      parts.push("", `Crash report: ${d.crashFile}`);
    }
    return parts.join("\n");
  }

  function pushDiagnosis(d: CrashDiagnosis, instanceId?: string | null) {
    lastDiagnosis.value = d;
    if (instanceId) lastCrashInstanceId.value = instanceId;
    open.value = true;
    messages.value.push({
      id: `crash-${Date.now()}`,
      role: "assistant",
      content: formatCrashMessage(d),
    });
    for (const s of d.suggestions.slice(0, 4)) {
      messages.value.push({ id: `s-${Date.now()}-${s.slice(0, 8)}`, role: "assistant", content: `• ${s}` });
    }
  }

  async function send(text: string) {
    const clean = text.trim();
    if (!clean) return;
    messages.value.push({ id: `u-${Date.now()}`, role: "user", content: clean });
    thinking.value = true;
    try {
      const res = await api.aiAssist(clean, lastDiagnosis.value);
      messages.value.push({ id: `a-${Date.now()}`, role: "assistant", content: res.message });
      for (const s of res.suggestions) {
        messages.value.push({ id: `as-${Date.now()}-${s.slice(0, 8)}`, role: "assistant", content: `• ${s}` });
      }
    } catch (e) {
      messages.value.push({
        id: `err-${Date.now()}`,
        role: "assistant",
        content: `No pude procesar la consulta: ${e}`,
      });
    } finally {
      thinking.value = false;
    }
  }

  return { open, messages, thinking, lastDiagnosis, lastCrashInstanceId, toggle, send, pushDiagnosis };
});
