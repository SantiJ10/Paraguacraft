<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import type { Instance } from "@/lib/types";
import InstanceIcon from "@/components/instance/InstanceIcon.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import ContextMenu, { type ContextMenuItem } from "@/components/common/ContextMenu.vue";
import { formatPlaytime, formatRelative } from "@/composables/useFormat";
import { coverKeyForMcVersion, versionCardImageUrl } from "@/lib/versionCatalog";
import { useInstancesStore } from "@/stores/instances";
import { useAppStore } from "@/stores/app";

const props = defineProps<{ instance: Instance; selected?: boolean }>();
defineEmits<{ (e: "play"): void; (e: "open"): void }>();

const router = useRouter();
const instances = useInstancesStore();
const app = useAppStore();

const menu = ref<{ x: number; y: number } | null>(null);
const busy = ref(false);
const localError = ref<string | null>(null);
const coverFailed = ref(false);
const coverUrl = computed(() => versionCardImageUrl(coverKeyForMcVersion(props.instance.mcVersion)));

watch(
  () => props.instance.mcVersion,
  () => {
    coverFailed.value = false;
  },
);

const menuItems: ContextMenuItem[] = [
  { id: "open", label: "Abrir" },
  { id: "play", label: "Jugar" },
  { id: "sep1", label: "", separator: true },
  { id: "edit", label: "Editar" },
  { id: "duplicate", label: "Duplicar" },
  { id: "sep2", label: "", separator: true },
  { id: "delete", label: "Eliminar", danger: true },
];

function onContext(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  menu.value = { x: e.clientX, y: e.clientY };
}

function openDetail() {
  instances.select(props.instance.id);
  router.push({ name: "instance-detail", params: { id: props.instance.id } });
}

async function onSelect(id: string) {
  menu.value = null;
  localError.value = null;
  if (id === "open" || id === "edit") {
    openDetail();
    return;
  }
  if (id === "play") {
    busy.value = true;
    try {
      await app.launch(props.instance.id, props.instance.name);
    } catch (e) {
      localError.value = String(e);
    } finally {
      busy.value = false;
    }
    return;
  }
  if (id === "duplicate") {
    busy.value = true;
    try {
      const copy = await instances.duplicate(props.instance.id, `${props.instance.name} (copia)`);
      instances.select(copy.id);
      router.push({ name: "instance-detail", params: { id: copy.id } });
    } catch (e) {
      localError.value = String(e);
    } finally {
      busy.value = false;
    }
    return;
  }
  if (id === "delete") {
    const ok = window.confirm(`¿Eliminar la instancia "${props.instance.name}"? Esta acción no se puede deshacer.`);
    if (!ok) return;
    busy.value = true;
    try {
      await instances.remove(props.instance.id);
    } catch (e) {
      localError.value = String(e);
    } finally {
      busy.value = false;
    }
  }
}
</script>

<template>
  <div
    class="lunar-card group relative flex h-full min-h-[220px] cursor-pointer flex-col overflow-hidden"
    :class="selected ? '!border-pc-green' : ''"
    @click="$emit('open')"
    @contextmenu="onContext"
  >
    <div class="relative min-h-[5rem] flex-1 overflow-hidden">
      <img
        v-if="!coverFailed"
        :src="coverUrl"
        alt=""
        class="absolute inset-0 h-full w-full object-cover opacity-80"
        @error="coverFailed = true"
      />
      <div
        v-else
        class="absolute inset-0 bg-gradient-to-br from-surface-4 to-surface-2"
      />
      <div class="absolute inset-0 bg-gradient-to-t from-surface-2 via-surface-2/40 to-transparent" />
    </div>
    <div class="relative bg-surface-2">
      <div class="flex items-center gap-3 p-4 pt-3">
        <InstanceIcon :icon="instance.icon" size="md" />
        <div class="min-w-0 flex-1">
          <p class="truncate text-base font-bold text-white">{{ instance.name }}</p>
          <p class="text-xs text-gray-400">
            MC {{ instance.mcVersion }}
            <span class="text-gray-600">·</span>
            <span class="capitalize">{{ instance.loader.replace(/-/g, " ") }}</span>
          </p>
          <p v-if="instance.modCount" class="mt-0.5 text-[11px] text-gray-500">
            {{ instance.modCount }} mods
          </p>
        </div>
      </div>
      <div class="flex items-center justify-between px-4 pb-2 text-xs text-gray-500">
        <span>{{ formatRelative(instance.lastPlayed) }}</span>
        <span>{{ formatPlaytime(instance.totalPlayMinutes) }}</span>
      </div>
      <BaseButton
        class="w-full rounded-none"
        :disabled="busy || app.launchPhase === 'running'"
        @click.stop="$emit('play')"
      >
        {{ busy ? "…" : "Jugar" }}
      </BaseButton>
      <p v-if="localError" class="px-3 pb-2 text-[10px] text-red-400">{{ localError }}</p>
    </div>

    <ContextMenu
      v-if="menu"
      :x="menu.x"
      :y="menu.y"
      :items="menuItems"
      @close="menu = null"
      @select="onSelect"
    />
  </div>
</template>
