<script setup lang="ts">
import { computed } from "vue";
import { useMusicStore } from "@/stores/music";

const music = useMusicStore();

const active = computed(() => music.isPlaying && music.hasTrack);
</script>

<template>
  <button
    type="button"
    class="music-btn"
    :class="{ 'music-btn-active': active || music.panelOpen }"
    title="Musica de fondo"
    @click="music.togglePanel()"
  >
    <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 3v10.55A4 4 0 1014 17V7h4V3h-6z" />
    </svg>
    <span class="music-bars" :class="{ 'music-bars-live': active }">
      <span /><span /><span /><span />
    </span>
  </button>
</template>

<style scoped>
.music-btn {
  @apply flex h-9 items-center gap-2 rounded-lg border border-surface-4 bg-surface-2/80 px-3 text-gray-400 transition-all hover:border-surface-5 hover:bg-surface-3 hover:text-white;
}
.music-btn-active {
  @apply border-pc-green/40 bg-pc-green/10 text-pc-green;
}

.music-bars {
  @apply flex h-4 items-end gap-0.5;
}
.music-bars span {
  @apply w-0.5 rounded-full bg-current opacity-40;
  height: 6px;
}
.music-bars-live span {
  @apply opacity-100;
  animation: eq 0.9s ease-in-out infinite alternate;
}
.music-bars-live span:nth-child(1) {
  animation-delay: 0s;
}
.music-bars-live span:nth-child(2) {
  animation-delay: 0.15s;
}
.music-bars-live span:nth-child(3) {
  animation-delay: 0.3s;
}
.music-bars-live span:nth-child(4) {
  animation-delay: 0.45s;
}

@keyframes eq {
  0% {
    height: 4px;
  }
  100% {
    height: 14px;
  }
}
</style>
