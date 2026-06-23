<script setup lang="ts">
import type { Instance } from "@/lib/types";
import InstanceIcon from "@/components/instance/InstanceIcon.vue";
import { formatPlaytime, formatRelative } from "@/composables/useFormat";

defineProps<{ instance: Instance; selected?: boolean }>();
defineEmits<{ (e: "play"): void; (e: "open"): void }>();
</script>

<template>
  <div
    class="lunar-card group cursor-pointer overflow-hidden"
    :class="selected ? '!border-pc-green' : ''"
    @click="$emit('open')"
  >
    <div class="flex items-center gap-3 p-4">
      <InstanceIcon :icon="instance.icon" size="md" />
      <div class="min-w-0 flex-1">
        <p class="truncate font-semibold">{{ instance.name }}</p>
        <p class="text-xs text-gray-500">
          {{ instance.mcVersion }} &middot; <span class="capitalize">{{ instance.loader.replace("-", " ") }}</span>
          <span v-if="instance.modCount"> &middot; {{ instance.modCount }} mods</span>
        </p>
      </div>
    </div>
    <div class="flex items-center justify-between border-t border-surface-3 px-4 py-2 text-xs text-gray-500">
      <span>{{ formatRelative(instance.lastPlayed) }}</span>
      <span>{{ formatPlaytime(instance.totalPlayMinutes) }}</span>
    </div>
    <button
      class="flex w-full items-center justify-center gap-2 bg-pc-green/0 py-2 text-sm font-bold text-pc-green opacity-0 transition-all group-hover:bg-pc-green/10 group-hover:opacity-100"
      @click.stop="$emit('play')"
    >
      <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
      Jugar
    </button>
  </div>
</template>
