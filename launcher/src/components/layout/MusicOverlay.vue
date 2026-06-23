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
        class="h-10 w-10 shrink-0 rounded-md object-cover"
      />
      <div v-else class="music-bars music-bars-live">
        <span /><span /><span />
      </div>
      <div class="min-w-0 flex-1">
        <p class="truncate text-xs font-bold">{{ music.overlayLabel }}</p>
        <p class="text-[10px] text-gray-500">
          {{ music.isSpotifyMode ? "Spotify" : inGame ? "Overlay en juego" : "Reproduciendo" }}
        </p>
        <div
          v-if="music.isSpotifyMode && music.spotifyNow?.durationMs"
          class="mt-1 h-0.5 overflow-hidden rounded-full bg-surface-4"
        >
          <div class="h-full bg-emerald-500" :style="{ width: `${music.spotifyProgressPct}%` }" />
        </div>
      </div>
      <div class="flex shrink-0 items-center gap-1">
        <button type="button" class="overlay-btn" title="Pausar" @click="music.togglePlay()">
          {{ music.isPlaying ? "⏸" : "▶" }}
        </button>
        <button
          v-if="music.isSpotifyMode"
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
  @apply fixed bottom-12 right-4 z-40 flex max-w-sm items-center gap-3 rounded-xl border border-surface-4 bg-surface-2/95 px-3 py-2 shadow-xl backdrop-blur;
}
.music-overlay-game {
  @apply border-pc-green/30 bg-surface-1/95;
}

.overlay-btn {
  @apply flex h-7 w-7 items-center justify-center rounded-md bg-surface-4 text-xs hover:bg-surface-5;
}

.music-bars {
  @apply flex h-10 w-10 shrink-0 items-end justify-center gap-0.5;
}
.music-bars span {
  @apply w-0.5 rounded-full bg-pc-green;
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
