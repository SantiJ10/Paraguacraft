<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useAiStore } from "@/stores/ai";
import { api, parseInvokeError } from "@/lib/ipc";
import type { AiAction } from "@/lib/types";
import BaseButton from "@/components/common/BaseButton.vue";

const ai = useAiStore();
const router = useRouter();
const draft = ref("");
const scroller = ref<HTMLElement | null>(null);
const runningAction = ref<string | null>(null);

function submit() {
  ai.send(draft.value);
  draft.value = "";
}

async function runAction(action: AiAction) {
  const key = `${action.action}:${action.param ?? ""}`;
  runningAction.value = key;
  try {
    if (action.action === "repair_instance" && action.param) {
      const report = await api.repairInstance(action.param);
      ai.pushNote(
        report.fixedCount > 0
          ? `Reparación completada: ${report.fixedCount} corrección(es).`
          : "Análisis completado: no se encontraron problemas.",
      );
    } else if (action.action === "sync_pvp_config" && action.param) {
      const msg = await api.syncPvpConfig(action.param);
      ai.pushNote(msg);
    } else if (action.action === "open_server_console" && action.param) {
      ai.toggle();
      await router.push(`/servers/${action.param}`);
    }
  } catch (e) {
    ai.pushNote(`No se pudo completar la acción: ${parseInvokeError(e).message}`);
  } finally {
    runningAction.value = null;
  }
}

watch(
  () => ai.messages.length,
  async () => {
    await nextTick();
    scroller.value?.scrollTo({ top: scroller.value.scrollHeight, behavior: "smooth" });
  },
);
</script>

<template>
  <Transition name="slide">
    <section
      v-if="ai.open"
      class="absolute bottom-0 right-0 top-0 z-50 flex w-96 flex-col border-l border-surface-4 bg-surface-1 shadow-2xl"
    >
      <header class="flex items-center justify-between border-b border-surface-3 px-4 py-3">
        <div class="flex items-center gap-2">
          <span class="flex h-7 w-7 items-center justify-center rounded-md bg-pc-ai/20">
            <svg class="h-4 w-4 text-pc-ai" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 3a4 4 0 014 4v1a4 4 0 010 8v1a4 4 0 01-8 0v-1a4 4 0 010-8V7a4 4 0 014-4z" />
            </svg>
          </span>
          <div>
            <h3 class="font-bold">Paraguabot</h3>
            <p v-if="ai.lastDiagnosis" class="text-xs text-amber-400">
              Crash: {{ ai.lastDiagnosis.category }}
            </p>
          </div>
        </div>
        <button class="text-gray-500 hover:text-white" @click="ai.toggle()">&times;</button>
      </header>

      <div ref="scroller" class="flex-1 space-y-3 overflow-y-auto p-4">
        <div v-for="m in ai.messages" :key="m.id" class="flex flex-col gap-1.5" :class="m.role === 'user' ? 'items-end' : 'items-start'">
          <div
            class="max-w-[85%] whitespace-pre-wrap rounded-2xl px-3.5 py-2 text-sm"
            :class="m.role === 'user' ? 'bg-pc-green text-black' : 'bg-surface-3 text-gray-200'"
          >
            {{ m.content }}
          </div>
          <div v-if="m.actions?.length" class="flex max-w-[85%] flex-wrap gap-1.5">
            <button
              v-for="act in m.actions"
              :key="`${act.action}:${act.param ?? ''}`"
              class="rounded-full border border-pc-ai/40 bg-pc-ai/10 px-3 py-1 text-xs font-medium text-pc-ai transition hover:bg-pc-ai/20 disabled:opacity-50"
              :disabled="runningAction === `${act.action}:${act.param ?? ''}`"
              @click="runAction(act)"
            >
              {{ runningAction === `${act.action}:${act.param ?? ''}` ? "..." : act.label }}
            </button>
          </div>
        </div>
        <div v-if="ai.thinking" class="flex gap-1 px-2">
          <span class="h-2 w-2 animate-pulse-dot rounded-full bg-pc-ai"></span>
          <span class="h-2 w-2 animate-pulse-dot rounded-full bg-pc-ai" style="animation-delay: 0.2s"></span>
          <span class="h-2 w-2 animate-pulse-dot rounded-full bg-pc-ai" style="animation-delay: 0.4s"></span>
        </div>
      </div>

      <form class="flex gap-2 border-t border-surface-3 p-3" @submit.prevent="submit">
        <input
          v-model="draft"
          placeholder="Escribi tu consulta..."
          class="flex-1 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm outline-none focus:border-pc-ai"
        />
        <BaseButton type="submit" variant="ai" size="sm">Enviar</BaseButton>
      </form>
    </section>
  </Transition>
</template>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.25s ease;
}
.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
