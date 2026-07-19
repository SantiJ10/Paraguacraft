<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "@/stores/app";
import { useMusicStore } from "@/stores/music";

const music = useMusicStore();
const app = useAppStore();

const visible = computed(() => {
  if (!music.hasTrack || !music.isPlaying) return false;
  if (music.inGameOverlay && app.launchPhase === "running") return true;
  if (music.showOverlay) return true;
  return false;
});

const inGame = computed(() => music.inGameOverlay && app.launchPhase === "running");
</script>

<template>
  <Transition name="float">
    <aside v-if="visible" class="music-overlay" :class="{ 'music-overlay-game': inGame }">
      <img
        v-if="music.overlayImage"
        :src="music.overlayImage"
        alt=""
        class="music-cover"
        referrerpolicy="no-referrer"
      />
      <div v-else class="music-bars music-bars-live">
        <span /><span /><span />
      </div>
      <div class="music-meta">
        <p class="music-title">{{ music.overlayTitle }}</p>
        <p class="music-artist">{{ music.overlayArtist }}</p>
        <div
          v-if="music.activeSource === 'spotify' && music.spotifyNow?.durationMs"
          class="mt-1 h-0.5 overflow-hidden rounded-full bg-surface-4"
        >
          <div class="h-full bg-[#00E5FF]" :style="{ width: `${music.spotifyProgressPct}%` }" />
        </div>
      </div>
      <div class="music-controls">
        <button type="button" class="overlay-btn" title="Pausar" @click="music.togglePlay()">
          {{ music.isPlaying ? "⏸" : "▶" }}
        </button>
        <button
          v-if="music.activeSource === 'spotify'"
          type="button"
          class="overlay-btn"
          title="Siguiente"
          @click="music.spotifyControl('next')"
        >
          ⏭
        </button>
        <button type="button" class="overlay-btn" title="Panel" @click="music.togglePanel()">♫</button>
      </div>
    </aside>
  </Transition>
</template>

<style scoped>
.music-overlay {
  @apply fixed bottom-12 right-4 z-40 flex w-auto max-w-[220px] items-center gap-2 rounded-lg border border-surface-4 bg-surface-2/95 px-2 py-1.5 shadow-xl backdrop-blur;
}
.music-overlay-game {
  @apply border-[#00E5FF]/30 bg-surface-1/95;
}

.music-cover {
  @apply h-8 w-8 shrink-0 rounded object-cover;
}

.music-meta {
  @apply min-w-0 flex-1;
}

.music-title {
  @apply truncate text-[11px] font-bold leading-tight text-[#00E5FF];
}

.music-artist {
  @apply truncate text-[10px] leading-tight text-gray-400;
}

.music-controls {
  @apply flex shrink-0 items-center gap-0.5;
}

.overlay-btn {
  @apply flex h-6 w-6 items-center justify-center rounded-md bg-surface-4 text-[10px] hover:bg-surface-5;
}

.music-bars {
  @apply flex h-8 w-8 shrink-0 items-end justify-center gap-0.5;
}
.music-bars span {
  @apply w-0.5 rounded-full bg-[#00E5FF];
  animation: eq 0.9s ease-in-out infinite alternate;
}
.music-bars span:nth-child(1) {
  animation-delay: 0s;
}
.music-bars span:nth-child(2) {
  animation-delay: 0.2s;
}
.music-bars span:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes eq {
  0% {
    height: 4px;
  }
  100% {
    height: 14px;
  }
}

.float-enter-active,
.float-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.float-enter-from,
.float-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
</style>
