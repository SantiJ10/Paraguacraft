<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useAccountsStore } from "@/stores/accounts";
import { useAiStore } from "@/stores/ai";

const route = useRoute();
const accounts = useAccountsStore();
const ai = useAiStore();

const nav = [
  { name: "home", label: "Inicio", to: "/", icon: "home" },
  { name: "instances", label: "Instancias", to: "/instances", icon: "grid" },
  { name: "store", label: "Tienda", to: "/store", icon: "store" },
  { name: "versions", label: "Versiones", to: "/versions", icon: "layers" },
];

const active = computed(() => accounts.active);

const icons: Record<string, string> = {
  home: "M3 11l9-8 9 8M5 9v11h14V9",
  grid: "M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z",
  store: "M3 9l1-5h16l1 5M4 9v11h16V9M9 13h6",
  layers: "M12 2l9 5-9 5-9-5 9-5zM3 12l9 5 9-5M3 17l9 5 9-5",
  settings:
    "M12 15a3 3 0 100-6 3 3 0 000 6zM19 12a7 7 0 00-.1-1l2-1.6-2-3.4-2.4 1a7 7 0 00-1.7-1l-.4-2.5h-4l-.4 2.5a7 7 0 00-1.7 1l-2.4-1-2 3.4 2 1.6a7 7 0 000 2l-2 1.6 2 3.4 2.4-1a7 7 0 001.7 1l.4 2.5h4l.4-2.5a7 7 0 001.7-1l2.4 1 2-3.4-2-1.6a7 7 0 00.1-1z",
};
</script>

<template>
  <aside class="flex w-60 shrink-0 flex-col border-r border-surface-3 bg-surface-0">
    <nav class="flex flex-1 flex-col gap-1 p-3">
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

      <div class="my-2 border-t border-surface-3"></div>

      <button class="nav-item text-left" :class="{ 'nav-item-active': ai.open }" @click="ai.toggle()">
        <svg class="h-5 w-5 text-pc-ai" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 3a4 4 0 014 4v1a4 4 0 010 8v1a4 4 0 01-8 0v-1a4 4 0 010-8V7a4 4 0 014-4z" />
        </svg>
        Asistente IA
      </button>
    </nav>

    <div class="p-3">
      <RouterLink
        to="/settings"
        class="nav-item mb-2"
        :class="{ 'nav-item-active': route.name === 'settings' }"
      >
        <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="icons.settings" />
        </svg>
        Ajustes
      </RouterLink>

      <div class="flex items-center gap-3 rounded-lg bg-surface-2 p-2">
        <div
          class="flex h-9 w-9 items-center justify-center rounded-md bg-surface-5 text-sm font-bold uppercase"
        >
          {{ active?.username?.[0] ?? "?" }}
        </div>
        <div class="min-w-0 flex-1">
          <p class="truncate text-sm font-semibold">{{ active?.username ?? "Sin cuenta" }}</p>
          <p class="text-xs" :class="active?.premium ? 'text-pc-green' : 'text-gray-500'">
            {{ active?.premium ? "Premium" : "Offline" }}
          </p>
        </div>
      </div>
    </div>
  </aside>
</template>
