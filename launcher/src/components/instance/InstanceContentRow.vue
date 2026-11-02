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
  // Preferir URL remota o data URL (Modrinth / logo PNG extraído del JAR / pack.png).
  if (props.item.iconUrl) return props.item.iconUrl;
  if (props.item.localIconPath) {
    try {
      return convertFileSrc(props.item.localIconPath);
    } catch {
      /* ignore */
    }
  }
  return contentFolderIcon(props.item.folder);
});

const formattedSize = computed(() => {
  const b = props.item.sizeBytes || 0;
  if (b <= 0) return "";
  if (b >= 1024 * 1024) return `${(b / (1024 * 1024)).toFixed(1)} MB`;
  return `${Math.max(1, Math.round(b / 1024))} KB`;
});

const versionBadge = computed(() => {
  const v = props.item.version?.trim();
  if (!v) return null;
  return v.startsWith("v") || v.startsWith("V") ? v : `v${v}`;
});

function humanizeName(raw: string) {
  return raw
    .replace(/\.(jar|zip)\.disabled$/gi, "")
    .replace(/\.(jar|zip|disabled)$/gi, "")
    .replace(/[-_]+/g, " ")
    .trim();
}
</script>

<template>
  <li
    class="flex items-start gap-3 px-4 py-3 transition-colors hover:bg-surface-3/40"
    :class="!item.enabled ? 'opacity-60' : ''"
  >
    <img
      :src="iconSrc"
      alt=""
      loading="lazy"
      decoding="async"
      class="mt-0.5 shrink-0 rounded-lg border border-surface-4 bg-surface-3 object-cover"
      :class="compact ? 'h-10 w-10' : 'h-12 w-12'"
    />
    <div class="min-w-0 flex-1">
      <div class="flex flex-wrap items-center gap-1.5">
        <p class="truncate font-semibold text-white">{{ title }}</p>
        <span
          v-if="versionBadge"
          class="rounded bg-surface-4 px-1.5 py-0.5 font-mono text-[10px] font-semibold text-gray-300"
        >
          {{ versionBadge }}
        </span>
        <span
          v-if="item.modLoader"
          class="rounded bg-sky-500/15 px-1.5 py-0.5 text-[10px] font-bold uppercase text-sky-300"
        >
          {{ item.modLoader }}
        </span>
        <span
          v-if="item.compatible === false"
          class="rounded bg-amber-500/20 px-1.5 py-0.5 text-[10px] font-bold uppercase text-amber-300"
          :title="item.compatMessage ?? 'Posible incompatibilidad'"
        >
          ⚠ Incompatible
        </span>
        <span
          v-else-if="item.enabled"
          class="rounded bg-pc-green/15 px-1.5 py-0.5 text-[10px] font-bold uppercase text-pc-green"
        >
          Activo
        </span>
        <span
          v-else
          class="rounded bg-surface-4 px-1.5 py-0.5 text-[10px] font-bold uppercase text-gray-400"
        >
          Desactivado
        </span>
      </div>
      <p v-if="item.author" class="mt-0.5 text-xs text-gray-400">por {{ item.author }}</p>
      <p
        v-if="item.compatible === false && item.compatMessage"
        class="mt-1 text-xs font-medium text-amber-300/90"
      >
        {{ item.compatMessage }}
      </p>
      <p v-if="item.description && !compact" class="mt-1 line-clamp-2 text-xs text-gray-400">
        {{ item.description }}
      </p>
      <p v-if="!compact" class="mt-1 flex flex-wrap items-center gap-1.5 text-[11px] text-gray-500">
        <span class="truncate font-mono">{{ item.name }}</span>
        <span v-if="formattedSize">· {{ formattedSize }}</span>
        <span v-if="item.sha1">· {{ item.sha1.slice(0, 8) }}</span>
      </p>
    </div>
    <div class="flex shrink-0 items-center gap-1.5 self-center">
      <button
        v-if="!compact"
        type="button"
        class="rounded-lg px-2 py-1.5 text-xs text-gray-400 transition hover:bg-surface-4 hover:text-white"
        title="Mostrar archivo en carpeta"
        @click="emit('reveal')"
      >
        📁
      </button>
      <button
        v-if="!compact"
        type="button"
        class="rounded-lg px-2 py-1.5 text-xs text-gray-400 transition hover:bg-red-900/40 hover:text-red-300"
        title="Eliminar archivo"
        @click="emit('remove')"
      >
        🗑
      </button>
      <button
        type="button"
        role="switch"
        :aria-checked="item.enabled"
        :title="item.enabled ? 'Desactivar (renombra a .disabled)' : 'Activar'"
        class="inline-flex items-center gap-2 rounded-lg border px-2.5 py-1.5 text-xs font-semibold transition-colors"
        :class="
          item.enabled
            ? 'border-pc-green/40 bg-pc-green/15 text-pc-green hover:border-amber-500/40 hover:bg-amber-900/30 hover:text-amber-200'
            : 'border-surface-4 bg-surface-3 text-gray-400 hover:border-pc-green/40 hover:text-white'
        "
        @click="emit('toggle')"
      >
        <span
          class="inline-block h-2 w-2 rounded-full"
          :class="item.enabled ? 'bg-pc-green' : 'bg-gray-500'"
        />
        {{ item.enabled ? "Activo" : "Off" }}
      </button>
    </div>
  </li>
</template>
