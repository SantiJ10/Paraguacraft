<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAccountsStore } from "@/stores/accounts";
import { useSkinsStore } from "@/stores/skins";
import { useAiStore } from "@/stores/ai";
import { isTauri } from "@/lib/ipc";
import launcherIcon from "@/assets/launcher-icon.png";

const route = useRoute();
const accounts = useAccountsStore();
const skins = useSkinsStore();
const ai = useAiStore();

const nav = [
  { name: "home", label: "Inicio", to: "/", icon: "home" },
  { name: "instances", label: "Instancias", to: "/instances", icon: "grid" },
  { name: "store", label: "Tienda", to: "/store", icon: "store" },
  { name: "skins", label: "Skins", to: "/skins", icon: "skins" },
  { name: "versions", label: "Versiones", to: "/versions", icon: "layers" },
  { name: "servers", label: "Servidores", to: "/servers", icon: "server" },
];

const FOCUS_REFRESH_MS = 60_000;
let unlistenFocus: (() => void) | null = null;
let lastFocusRefresh = 0;

onMounted(async () => {
  if (!isTauri()) return;
  try {
    unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused) return;
      const now = Date.now();
      if (now - lastFocusRefresh < FOCUS_REFRESH_MS) return;
      lastFocusRefresh = now;
      void skins.refresh();
    });
  } catch {
    /* ignore */
  }
});

onUnmounted(() => {
  unlistenFocus?.();
});

watch(
  () => accounts.active?.id,
  () => {
    void skins.refresh(true);
  },
);

const icons: Record<string, string> = {
  home: "M3 11l9-8 9 8M5 9v11h14V9",
  grid: "M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z",
  store: "M3 9l1-5h16l1 5M4 9v11h16V9M9 13h6",
  skins: "M12 2a4 4 0 00-4 4v2H6a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V10a2 2 0 00-2-2h-2V6a4 4 0 00-4-4zm0 2a2 2 0 012 2v2h-4V6a2 2 0 012-2z",
  layers: "M12 2l9 5-9 5-9-5 9-5zM3 12l9 5 9-5M3 17l9 5 9-5",
  server: "M4 6h16v4H4zM4 14h16v4H4zM6 8h.01M6 16h.01",
  settings:
    "M12 15a3 3 0 100-6 3 3 0 000 6zM19 12a7 7 0 00-.1-1l2-1.6-2-3.4-2.4 1a7 7 0 00-1.7-1l-.4-2.5h-4l-.4 2.5a7 7 0 00-1.7 1l-2.4-1-2 3.4 2 1.6a7 7 0 000 2l-2 1.6 2 3.4 2.4-1a7 7 0 001.7 1l.4 2.5h4l.4-2.5a7 7 0 001.7-1l2.4 1 2-3.4-2-1.6a7 7 0 00.1-1z",
  bot: "M12 3a4 4 0 014 4v1a4 4 0 010 8v1a4 4 0 01-8 0v-1a4 4 0 010-8V7a4 4 0 014-4z",
};
</script>

<template>
  <aside class="flex w-60 shrink-0 flex-col border-r border-surface-3 bg-surface-0">
    <div class="flex items-center gap-2 border-b border-surface-3 px-4 py-3">
      <img :src="launcherIcon" alt="" class="h-8 w-8 rounded-md" />
      <span class="text-sm font-bold tracking-wide text-gray-200">PARAGUACRAFT</span>
    </div>

    <nav class="flex flex-1 flex-col gap-1 overflow-y-auto p-3">
      <RouterLink
        v-for="item in nav"
        :key="item.name"
        :to="item.to"
        class="nav-item"
        :class="{ 'nav-item-active': route.name === item.name }"
      >
        <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="icons[item.icon]" />
        </svg>
        {{ item.label }}
      </RouterLink>
    </nav>

    <div class="mt-auto border-t border-surface-3 p-3">
      <button
        type="button"
        class="nav-item mb-1 w-full text-left"
        :class="{ 'nav-item-active': ai.open }"
        @click="ai.toggle()"
      >
        <svg class="h-5 w-5 text-pc-ai" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="icons.bot" />
        </svg>
        Paraguabot
      </button>

      <RouterLink
        to="/settings"
        class="nav-item"
        :class="{ 'nav-item-active': route.name === 'settings' }"
      >
        <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="icons.settings" />
        </svg>
        Ajustes
      </RouterLink>
    </div>
  </aside>
</template>
