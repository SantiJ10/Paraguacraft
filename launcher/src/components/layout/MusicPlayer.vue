<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { parseMusicUrl } from "@/lib/musicUrl";
import { controlYoutube, destroyYoutube, mountYoutube } from "@/lib/youtubePlayer";
import { useAppStore } from "@/stores/app";
import { useMusicStore } from "@/stores/music";

const music = useMusicStore();
const app = useAppStore();
const host = ref<HTMLDivElement | null>(null);

const inGame = computed(() => app.launchPhase === "running");
const audioActive = computed(() => music.shouldPlayAudio(inGame.value));

const parsed = computed(() => music.youtubeTrack ?? (music.inputUrl ? parseMusicUrl(music.inputUrl) : null));

async function syncPlayer() {
  if (!host.value || !parsed.value || !audioActive.value) {
    if (!audioActive.value) controlYoutube(false, music.volume);
    return;
  }
  try {
    await mountYoutube(host.value, parsed.value, music.volume, true);
  } catch {
    /* API aun cargando */
  }
}

watch(
  () => [audioActive.value, music.volume, music.embedUrl, music.inputUrl] as const,
  () => {
    if (audioActive.value && parsed.value) void syncPlayer();
    else controlYoutube(false, music.volume);
  },
);

watch(audioActive, (active) => {
  if (!active) controlYoutube(false, music.volume);
});

onUnmounted(() => destroyYoutube());
</script>

<template>
  <div v-show="music.embedUrl && audioActive" class="music-player-host" aria-hidden="true">
    <div ref="host" class="music-host" />
  </div>
</template>

<style scoped>
.music-player-host {
  position: fixed;
  bottom: 0;
  right: 0;
  width: 320px;
  height: 180px;
  overflow: hidden;
  opacity: 0.001;
  pointer-events: none;
  z-index: -1;
}
.music-host {
  width: 320px;
  height: 180px;
}
</style>
