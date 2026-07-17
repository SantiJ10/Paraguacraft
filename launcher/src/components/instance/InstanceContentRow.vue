<script setup lang="ts">
import { computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { contentFolderIcon } from "@/lib/contentIcons";
import type { InstanceContentItem } from "@/lib/types";

const props = defineProps<{
  item: InstanceContentItem;
  compact?: boolean;
}>();

const emit = defineEmits<{
  toggle: [];
  reveal: [];
  remove: [];
}>();

const title = computed(
  () => props.item.displayName?.trim() || humanizeName(props.item.name),
);

const iconSrc = computed(() => {
  if (props.item.localIconPath) {
    try {
      return convertFileSrc(props.item.localIconPath);
    } catch {
      /* ignore */
    }
  }
  if (props.item.iconUrl) return props.item.iconUrl;
  return contentFolderIcon(props.item.folder);
});

function humanizeName(raw: string) {
  return raw
    .replace(/\.(jar|zip|disabled)$/gi, "")
    .replace(/[-_]+/g, " ")
    .trim();
}
</script>

<template>
  <li
    class="flex items-start gap-3 px-4 py-3"
    :class="!item.enabled ? 'opacity-50' : ''"
  >
    <img
      :src="iconSrc"
      alt=""
      class="mt-0.5 shrink-0 rounded-lg bg-surface-3 object-cover"
      :class="compact ? 'h-10 w-10' : 'h-12 w-12'"
    />
    <div class="min-w-0 flex-1">
      <div class="flex flex-wrap items-center gap-2">
        <p class="truncate font-semibold text-white">{{ title }}</p>
        <span
          v-if="item.compatible === false"
          class="rounded bg-amber-500/15 px-1.5 py-0.5 text-[10px] font-bold uppercase text-amber-300"
          :title="item.compatMessage ?? 'Incompatible'"
        >
          Incompatible
        </span>
        <span
          v-else-if="item.enabled"
          class="rounded bg-pc-green/15 px-1.5 py-0.5 text-[10px] font-bold uppercase text-pc-green"
        >
          Activo
        </span>
      </div>
      <p v-if="item.author" class="text-xs text-gray-500">por {{ item.author }}</p>
      <p v-if="item.description && !compact" class="mt-1 line-clamp-2 text-xs text-gray-400">
        {{ item.description }}
      </p>
      <p v-if="!compact" class="mt-1 text-xs text-gray-600">
        {{ item.name }}
        <span v-if="item.sha1"> · {{ item.sha1.slice(0, 8) }}…</span>
      </p>
    </div>
    <div class="flex shrink-0 gap-1">
      <button
        v-if="!compact"
        class="rounded-lg px-2 py-1 text-xs text-gray-400 hover:bg-surface-4 hover:text-white"
        title="Mostrar en carpeta"
        @click="emit('reveal')"
      >
        📁
      </button>
      <button
        v-if="!compact"
        class="rounded-lg px-2 py-1 text-xs text-gray-400 hover:bg-red-900/40 hover:text-red-300"
        title="Eliminar"
        @click="emit('remove')"
      >
        🗑
      </button>
      <button
        class="rounded-lg px-3 py-1 text-xs font-semibold transition-colors"
        :class="item.enabled ? 'bg-surface-4 text-gray-300 hover:bg-amber-900/40 hover:text-amber-200' : 'bg-pc-green/20 text-pc-green'"
        @click="emit('toggle')"
      >
        {{ item.enabled ? "Desactivar" : "Activar" }}
      </button>
    </div>
  </li>
</template>
