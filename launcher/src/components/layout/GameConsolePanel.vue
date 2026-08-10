<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import { api, isTauri } from "@/lib/ipc";
import BaseButton from "@/components/common/BaseButton.vue";

const app = useAppStore();
const router = useRouter();

const open = ref(false);
const lines = ref<string[]>([]);
const busy = ref(false);
const msg = ref<string | null>(null);
const consoleEl = ref<HTMLElement | null>(null);

let poll: ReturnType<typeof setInterval> | null = null;

const instanceId = computed(() => app.activeGameInstanceId);
const playing = computed(() => app.launchPhase === "running" && !!instanceId.value);
const showPanel = computed(() => playing.value && isTauri());

async function refresh() {
  const id = instanceId.value;
  if (!id) return;
  try {
    lines.value = await api.getClientConsole(id, 400);
    scrollBottom();
  } catch {
    /* ignore transient */
  }
}

function scrollBottom() {
  requestAnimationFrame(() => {
    const el = consoleEl.value;
    if (el) el.scrollTop = el.scrollHeight;
  });
}

function startPoll() {
  stopPoll();
  void refresh();
  poll = setInterval(() => void refresh(), 800);
}

function stopPoll() {
  if (poll) {
    clearInterval(poll);
    poll = null;
  }
}

watch(
  [playing, open],
  ([isPlaying, isOpen]) => {
    if (isPlaying && isOpen) startPoll();
    else stopPoll();
  },
  { immediate: true },
);

onMounted(() => {
  void app.initGameEvents();
});

onUnmounted(() => stopPoll());

async function copy() {
  const text = lines.value.join("\n");
  if (!text.trim()) {
    msg.value = "Sin líneas aún.";
    return;
  }
  try {
    await navigator.clipboard.writeText(text);
    msg.value = "Log copiado.";
  } catch {
    msg.value = "No se pudo copiar.";
  }
}

async function exportLog() {
  const id = instanceId.value;
  if (!id) return;
  busy.value = true;
  msg.value = null;
  try {
    const path = await api.exportClientConsole(id);
    msg.value = `Exportado: ${path}`;
  } catch (e) {
    msg.value = String(e);
  } finally {
    busy.value = false;
  }
}

function goInstanceLogs() {
  const id = instanceId.value;
  if (!id) return;
  open.value = false;
  router.push({ name: "instance-detail", params: { id }, query: { tab: "logs" } });
}
</script>

<template>
  <div
    v-if="showPanel"
    class="fixed bottom-3 right-3 z-40 flex max-w-[min(36rem,calc(100vw-1.5rem))] flex-col items-end gap-2"
  >
    <div
      v-if="open"
      class="w-[min(36rem,calc(100vw-1.5rem))] overflow-hidden rounded-xl border border-surface-4 bg-surface-2/95 shadow-xl backdrop-blur"
    >
      <div class="flex items-center justify-between gap-2 border-b border-surface-4 px-3 py-2">
        <div class="min-w-0">
          <p class="text-sm font-semibold text-white">Consola del juego</p>
          <p class="truncate font-mono text-[10px] text-gray-500">{{ instanceId }} · en vivo</p>
        </div>
        <div class="flex shrink-0 flex-wrap gap-1">
          <BaseButton size="sm" variant="secondary" @click="copy">Copiar</BaseButton>
          <BaseButton size="sm" variant="secondary" :disabled="busy" @click="exportLog">Exportar</BaseButton>
          <BaseButton size="sm" variant="secondary" @click="goInstanceLogs">Ampliar</BaseButton>
          <BaseButton size="sm" variant="secondary" @click="open = false">Cerrar</BaseButton>
        </div>
      </div>
      <pre
        ref="consoleEl"
        class="max-h-64 overflow-auto bg-black/50 p-3 font-mono text-[11px] leading-relaxed text-gray-300"
      >{{ lines.length ? lines.join("\n") : "Esperando líneas de logs/latest.log…" }}</pre>
      <p v-if="msg" class="border-t border-surface-4 px-3 py-1.5 text-xs text-pc-green">{{ msg }}</p>
    </div>
    <button
      v-else
      type="button"
      class="rounded-lg border border-pc-green/40 bg-surface-2/95 px-3 py-2 text-sm font-semibold text-pc-green shadow-lg backdrop-blur hover:bg-surface-3"
      @click="open = true"
    >
      Consola del juego
    </button>
  </div>
</template>
