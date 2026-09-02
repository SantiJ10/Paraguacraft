<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { useServersStore } from "@/stores/servers";
import launcherIcon from "@/assets/launcher-icon.png";

const servers = useServersStore();
const router = useRouter();

const visible = computed(() => servers.hasRunning);
const extra = computed(() => Math.max(0, servers.running.length - 1));
const stopping = computed(() => !!servers.stoppingId);

function openServer() {
  const id = servers.primaryRunning?.id;
  if (!id) return;
  void router.push({ name: "server-detail", params: { id } });
}

function stopServer() {
  void servers.stopRunning();
}
</script>

<template>
  <Transition name="float">
    <aside v-if="visible" class="server-overlay" title="Hay un servidor local encendido. Gasta RAM hasta que lo detengas.">
      <img :src="launcherIcon" alt="" class="server-icon" />
      <div class="server-meta">
        <p class="server-title">{{ servers.primaryRunning?.name ?? "Servidor" }}</p>
        <p class="server-sub">
          En ejecución{{ extra ? ` · +${extra}` : "" }}
        </p>
      </div>
      <div class="server-controls">
        <button
          type="button"
          class="overlay-btn"
          :disabled="stopping"
          title="Detener servidor"
          @click="stopServer"
        >
          <svg class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <rect x="6" y="6" width="12" height="12" rx="1" />
          </svg>
        </button>
        <button type="button" class="overlay-btn" title="Abrir servidor" @click="openServer">
          <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M9 18l6-6-6-6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </div>
    </aside>
  </Transition>
</template>

<style scoped>
.server-overlay {
  @apply fixed bottom-12 z-40 flex max-w-[240px] items-center gap-2 rounded-lg border border-pc-green/35 bg-surface-2/95 px-2 py-1.5 shadow-xl backdrop-blur;
  left: 15.75rem;
}
.server-icon {
  @apply h-8 w-8 shrink-0 rounded object-cover;
}
.server-meta {
  @apply min-w-0 flex-1;
}
.server-title {
  @apply truncate text-[11px] font-bold leading-tight text-pc-green;
}
.server-sub {
  @apply truncate text-[10px] leading-tight text-gray-400;
}
.server-controls {
  @apply flex shrink-0 items-center gap-0.5;
}
.overlay-btn {
  @apply flex h-6 w-6 items-center justify-center rounded-md bg-surface-4 text-gray-200 hover:bg-surface-5 disabled:opacity-50;
}
.float-enter-active,
.float-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.float-enter-from,
.float-leave-to {
  opacity: 0;
  transform: translateY(6px);
}
</style>
