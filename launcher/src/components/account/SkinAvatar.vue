<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { STEVE_AVATAR_URL } from "@/lib/skins";
import { isTauri } from "@/lib/ipc";

const props = withDefaults(
  defineProps<{
    uuid?: string | null;
    username?: string | null;
    avatarUrl?: string | null;
    avatarDataUrl?: string | null;
    localAvatarPath?: string | null;
    size?: "sm" | "md" | "lg";
    loading?: boolean;
  }>(),
  { size: "md", loading: false },
);

const remoteFailed = ref(false);

const sizeClass = computed(() => {
  if (props.size === "sm") return "h-9 w-9";
  if (props.size === "lg") return "h-12 w-12";
  return "h-10 w-10";
});

const src = computed(() => {
  if (remoteFailed.value) return STEVE_AVATAR_URL;

  // Prioridad: cara recortada (data URL) o minotar — NUNCA UV crudo sin crop.
  if (props.avatarDataUrl) {
    // Si por error viejo viniera un UV completo, object-cover lo disimula mal;
    // dataURL de cara siempre es 64x64 head.
    return props.avatarDataUrl;
  }

  // localAvatarPath debe ser el cache de CARA (face.png), no la skin completa.
  if (props.localAvatarPath && isTauri()) {
    const p = props.localAvatarPath.replace(/\\/g, "/").toLowerCase();
    const looksLikeFullSkin =
      p.endsWith("paraguacraft_offline_skin.png") ||
      (p.includes("/skins/") && !p.includes("face") && !p.includes("helm") && !p.includes("avatar"));
    if (!looksLikeFullSkin) {
      try {
        return convertFileSrc(props.localAvatarPath);
      } catch {
        /* fallback */
      }
    }
  }

  const url = props.avatarUrl?.trim();
  if (url) return url;

  return STEVE_AVATAR_URL;
});

const alt = computed(() =>
  props.username ? `Skin de ${props.username}` : "Skin de Steve",
);

watch(
  () => [props.uuid, props.avatarUrl, props.avatarDataUrl, props.localAvatarPath],
  () => {
    remoteFailed.value = false;
  },
);

function onError() {
  if (!remoteFailed.value) remoteFailed.value = true;
}
</script>

<template>
  <div
    class="mc-avatar flex shrink-0 items-center justify-center overflow-hidden rounded-lg bg-surface-5 ring-1 ring-surface-4"
    :class="sizeClass"
  >
    <div v-if="loading" class="h-full w-full animate-pulse bg-surface-4" aria-hidden="true" />
    <img
      v-else
      :key="src"
      :src="src"
      :alt="alt"
      class="h-full w-full object-cover object-top"
      referrerpolicy="no-referrer"
      @error="onError"
    />
  </div>
</template>

<style scoped>
.mc-avatar img {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}
</style>
