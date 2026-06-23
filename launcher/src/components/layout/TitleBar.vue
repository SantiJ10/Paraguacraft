<script setup lang="ts">
import { isTauri } from "@/lib/ipc";

// Ventana sin marco: controlamos minimizar/maximizar/cerrar nosotros.
// `data-tauri-drag-region` permite arrastrar la ventana desde la barra.
async function win() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

async function minimize() {
  if (isTauri()) (await win()).minimize();
}
async function toggleMaximize() {
  if (isTauri()) (await win()).toggleMaximize();
}
async function close() {
  if (isTauri()) {
    const { api } = await import("@/lib/ipc");
    try {
      await api.shutdownBackgroundServices();
    } catch {
      /* best effort */
    }
    (await win()).close();
  }
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-9 shrink-0 items-center justify-between border-b border-surface-3 bg-surface-0 pl-3 pr-1 select-none"
  >
    <div data-tauri-drag-region class="flex items-center gap-2 pointer-events-none">
      <img src="/favicon.png" alt="" class="h-4 w-4 rounded-sm pointer-events-none" />
      <span class="text-xs font-bold tracking-widest text-gray-300">PARAGUACRAFT</span>
      <span class="text-[10px] font-semibold tracking-wider text-pc-green">LAUNCHER</span>
    </div>
    <div class="flex items-center">
      <button class="titlebar-btn" title="Minimizar" @click="minimize">
        <svg viewBox="0 0 12 12" class="h-3 w-3"><rect x="2" y="5.5" width="8" height="1" fill="currentColor" /></svg>
      </button>
      <button class="titlebar-btn" title="Maximizar" @click="toggleMaximize">
        <svg viewBox="0 0 12 12" class="h-3 w-3" fill="none" stroke="currentColor"><rect x="2.5" y="2.5" width="7" height="7" /></svg>
      </button>
      <button class="titlebar-btn hover:!bg-red-600 hover:!text-white" title="Cerrar" @click="close">
        <svg viewBox="0 0 12 12" class="h-3 w-3" fill="none" stroke="currentColor" stroke-width="1.4"><path d="M3 3l6 6M9 3l-6 6" /></svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar-btn {
  @apply flex h-9 w-11 items-center justify-center text-gray-400 transition-colors hover:bg-surface-3 hover:text-white;
}
</style>
