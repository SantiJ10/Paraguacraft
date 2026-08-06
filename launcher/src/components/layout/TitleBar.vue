<script setup lang="ts">
import { isTauri, openUrl } from "@/lib/ipc";

const CAFECITO_URL = "https://cafecito.app/amin1001";

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

async function openCafecito() {
  await openUrl(CAFECITO_URL);
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
      <button
        class="titlebar-btn titlebar-cafecito"
        title="Invitame un cafecito"
        aria-label="Donar en Cafecito"
        @click="openCafecito"
      >
        <svg viewBox="0 0 24 24" class="h-4 w-4" aria-hidden="true">
          <rect x="2" y="2" width="20" height="20" rx="5" fill="#7BA3D4" />
          <path
            fill="#fff"
            d="M8.2 9.1c0-2.1 1.7-3.8 3.8-3.8s3.8 1.7 3.8 3.8c0 .7-.2 1.3-.5 1.8l1.1.9c.7-.9 1.1-2 1.1-3.2 0-2.9-2.4-5.3-5.5-5.3S6.5 5.9 6.5 8.8c0 1.5.6 2.8 1.6 3.7l1.1-.9A3.7 3.7 0 0 1 8.2 9.1zm3.8 1.6c-1 0-1.8.7-2 1.6h1.5c.1-.2.4-.4.7-.4s.6.2.7.4h1.5c-.2-.9-1-1.6-2-1.6zm-3.4 3.5 1.2 1.1c.6.5 1.4.8 2.2.8s1.6-.3 2.2-.8l1.2-1.1c.5.5.9 1.1 1.1 1.8H7.5c.2-.7.6-1.3 1.1-1.8z"
          />
          <circle cx="12" cy="13.2" r="2.1" fill="#7BA3D4" />
          <path fill="#fff" d="M12.9 14.1c0 .3-.4.5-.9.5s-.9-.2-.9-.5.4-.6.9-.6.9.3.9.6z" />
        </svg>
      </button>
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
.titlebar-cafecito {
  @apply w-10 text-[#9eb8da] hover:text-white;
}
</style>
