import { defineStore } from "pinia";
import { ref } from "vue";
import type { AiMessage } from "@/lib/types";

// Asistente IA global. Provider-agnostico: en Fase 4 el backend Rust expone un
// trait `AiProvider` (heuristico local ahora, cloud/own-backend luego). La UI
// solo conoce este store.
export const useAiStore = defineStore("ai", () => {
  const open = ref(false);
  const messages = ref<AiMessage[]>([
    {
      id: "sys-1",
      role: "assistant",
      content:
        "Hola, soy el asistente de Paraguacraft. Puedo ayudarte a elegir versiones, diagnosticar crashes y optimizar tu juego. (IA en construccion - Fase 4)",
    },
  ]);
  const thinking = ref(false);

  function toggle() {
    open.value = !open.value;
  }

  function send(text: string) {
    const clean = text.trim();
    if (!clean) return;
    messages.value.push({ id: `u-${Date.now()}`, role: "user", content: clean });
    thinking.value = true;
    // Stub: respuesta heuristica simulada hasta integrar el backend.
    setTimeout(() => {
      messages.value.push({
        id: `a-${Date.now()}`,
        role: "assistant",
        content:
          "La integracion con IA llega en la Fase 4. Por ahora dejo registrada tu consulta para el diagnostico automatico.",
      });
      thinking.value = false;
    }, 700);
  }

  return { open, messages, thinking, toggle, send };
});
