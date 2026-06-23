<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getInstanceIconSrc, isCustomInstanceIcon, resolveInstanceIcon } from "@/lib/instanceIcons";
import { api, isTauri } from "@/lib/ipc";

const props = withDefaults(
  defineProps<{
    icon: string;
    size?: "sm" | "md" | "lg";
  }>(),
  { size: "md" },
);

const resolved = computed(() => resolveInstanceIcon(props.icon));
const builtInSrc = computed(() => getInstanceIconSrc(props.icon));
const customSrc = ref<string | null>(null);

watch(
  () => props.icon,
  async (icon) => {
    customSrc.value = null;
    if (!isTauri() || !isCustomInstanceIcon(icon)) return;
    try {
      const path = await api.getInstanceIconPath(icon);
      if (path) customSrc.value = convertFileSrc(path);
    } catch {
      customSrc.value = null;
    }
  },
  { immediate: true },
);

const src = computed(() => builtInSrc.value ?? customSrc.value);

const boxClass = computed(() => {
  const sizes = {
    sm: "h-9 w-9 rounded-lg",
    md: "h-12 w-12 rounded-lg",
    lg: "h-16 w-16 rounded-xl",
  };
  return `flex shrink-0 items-center justify-center overflow-hidden bg-surface-4 ${sizes[props.size]}`;
});

const imgClass = computed(() => {
  const sizes = {
    sm: "h-7 w-7",
    md: "h-10 w-10",
    lg: "h-14 w-14",
  };
  return `${sizes[props.size]} object-contain`;
});
</script>

<template>
  <div :class="boxClass">
    <img
      v-if="src"
      :src="src"
      :alt="`Icono ${resolved}`"
      :class="imgClass"
      draggable="false"
    />
    <span v-else class="font-emoji text-2xl leading-none">{{ icon }}</span>
  </div>
</template>
