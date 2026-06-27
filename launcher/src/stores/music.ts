import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import { parseMusicUrl, type ParsedMusic } from "@/lib/musicUrl";
import type { SpotifyNowPlaying } from "@/lib/types";
import { api, isTauri } from "@/lib/ipc";
import { destroyYoutube } from "@/lib/youtubePlayer";

const STORAGE_KEY = "paraguacraft.music.v1";

interface MusicPersist {
  inputUrl: string;
  source: "youtube" | "spotify" | null;
  embedUrl: string | null;
  label: string;
  playing: boolean;
  volume: number;
  launcherBackground: boolean;
  launcherOverlay: boolean;
  inGameOverlay: boolean;
  spotifyClientId: string;
  spotifyRedirectUri: string;
}

function loadPersist(): Partial<MusicPersist> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as Partial<MusicPersist>) : {};
  } catch {
    return {};
  }
}

function savePersist(state: MusicPersist) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    /* ignore */
  }
}

export const SPOTIFY_REDIRECT_URI = "http://127.0.0.1:8888/callback";

export const SPOTIFY_REDIRECT_OPTIONS = [SPOTIFY_REDIRECT_URI] as const;

export const useMusicStore = defineStore("music", () => {
  const saved = loadPersist();

  const panelOpen = ref(false);
  const source = ref<"youtube" | "spotify" | null>(saved.source ?? null);
  const embedUrl = ref<string | null>(saved.embedUrl ?? null);
  const youtubeTrack = ref<ParsedMusic | null>(null);
  const inputUrl = ref(saved.inputUrl ?? "");
  const label = ref(saved.label ?? "");
  const playing = ref(saved.playing ?? false);
  const volume = ref(saved.volume ?? 70);
  const launcherBackground = ref(saved.launcherBackground ?? true);
  const launcherOverlay = ref(saved.launcherOverlay ?? true);
  const inGameOverlay = ref(saved.inGameOverlay ?? true);

  const spotifyConnected = ref(false);
  const spotifyClientId = ref(saved.spotifyClientId ?? "");
  const spotifyRedirectUri = ref(
    (saved.spotifyRedirectUri ?? SPOTIFY_REDIRECT_URI).replace("localhost", "127.0.0.1"),
  );
  const spotifyClientSecret = ref("");
  const spotifyNow = ref<SpotifyNowPlaying | null>(null);
  const spotifyAuthBusy = ref(false);

  let spotifyPollTimer: number | null = null;

  const isSpotifyMode = computed(() => spotifyConnected.value);
  const spotifyPlaying = computed(() => Boolean(spotifyNow.value?.playing && spotifyNow.value?.title));
  const spotifyLabel = computed(() => {
    const t = spotifyNow.value;
    if (!t?.title) return "Spotify";
    return t.artist ? `${t.title} · ${t.artist}` : t.title;
  });
  const spotifyProgressPct = computed(() => {
    const t = spotifyNow.value;
    if (!t?.durationMs) return 0;
    return Math.min(100, Math.round((t.progressMs / t.durationMs) * 100));
  });

  const hasTrack = computed(
    () => Boolean(embedUrl.value) || Boolean(spotifyNow.value?.title),
  );
  const isPlaying = computed(() => {
    if (isSpotifyMode.value) return spotifyPlaying.value;
    return playing.value && Boolean(embedUrl.value);
  });
  const overlayLabel = computed(() =>
    isSpotifyMode.value && spotifyNow.value?.title ? spotifyLabel.value : label.value,
  );
  const overlayImage = computed(() => spotifyNow.value?.imageUrl ?? null);
  const showOverlay = computed(() => isPlaying.value && launcherOverlay.value && hasTrack.value);

  function shouldPlayAudio(inGame: boolean) {
    if (isSpotifyMode.value) return false;
    if (!playing.value || !embedUrl.value) return false;
    if (inGame) return inGameOverlay.value;
    return launcherBackground.value;
  }

  function syncOverlayIpc() {
    if (!isTauri()) return;
    const active = isPlaying.value && inGameOverlay.value;
    if (isSpotifyMode.value && spotifyNow.value?.title) {
      void api.syncOverlayMusic(
        active,
        spotifyNow.value.title ?? "",
        spotifyNow.value.artist ?? "",
      );
    } else {
      void api.syncOverlayMusic(active, active ? label.value : "", "");
    }
  }

  watch(
    [isPlaying, inGameOverlay, label, spotifyNow, playing, isSpotifyMode],
    syncOverlayIpc,
    { deep: true },
  );

  function persist() {
    savePersist({
      inputUrl: inputUrl.value,
      source: source.value,
      embedUrl: embedUrl.value,
      label: label.value,
      playing: playing.value,
      volume: volume.value,
      launcherBackground: launcherBackground.value,
      launcherOverlay: launcherOverlay.value,
      inGameOverlay: inGameOverlay.value,
      spotifyClientId: spotifyClientId.value,
      spotifyRedirectUri: spotifyRedirectUri.value,
    });
  }

  watch(
    [
      inputUrl,
      source,
      embedUrl,
      label,
      playing,
      volume,
      launcherBackground,
      launcherOverlay,
      inGameOverlay,
      spotifyClientId,
      spotifyRedirectUri,
    ],
    persist,
    { deep: true },
  );

  function stopSpotifyPoll() {
    if (spotifyPollTimer !== null) {
      clearInterval(spotifyPollTimer);
      spotifyPollTimer = null;
    }
  }

  function startSpotifyPoll() {
    stopSpotifyPoll();
    if (!isTauri() || !spotifyConnected.value) return;
    void tickSpotify();
    spotifyPollTimer = window.setInterval(() => void tickSpotify(), 5000);
  }

  async function refreshSpotifyStatus() {
    if (!isTauri()) return;
    const status = await api.spotifyStatus();
    spotifyConnected.value = status.connected;
    if (status.clientId && !spotifyClientId.value) {
      spotifyClientId.value = status.clientId;
    }
    if (status.redirectUri) {
      spotifyRedirectUri.value = status.redirectUri.replace("localhost", "127.0.0.1");
    }
  }

  async function trySpotifyAutoconnect() {
    if (!isTauri()) return false;
    await refreshSpotifyStatus();
    const r = await api.spotifyTryAutoconnect();
    if (r.ok) {
      spotifyConnected.value = true;
      source.value = "spotify";
      startSpotifyPoll();
      return true;
    }
    return false;
  }

  async function tickSpotify() {
    if (!isTauri() || !spotifyConnected.value) return;
    try {
      const data = await api.spotifyNowPlaying();
      spotifyNow.value = data;
      syncOverlayIpc();
      if (!data.ok && data.error?.includes("expirada")) {
        spotifyConnected.value = false;
        stopSpotifyPoll();
      }
    } catch {
      /* ignore */
    }
  }

  async function saveSpotifyCredentials() {
    if (!spotifyClientId.value.trim() || !spotifyClientSecret.value.trim()) {
      throw new Error("Completa Client ID y Client Secret");
    }
    await api.spotifySaveCredentials(
      spotifyClientId.value.trim(),
      spotifyClientSecret.value.trim(),
      spotifyRedirectUri.value.trim(),
    );
  }

  async function validateSpotifyCredentials() {
    await saveSpotifyCredentials();
    const result = await api.spotifyValidateApp();
    if (!result.ok) throw new Error(result.error ?? "Credenciales invalidas");
    return true;
  }

  async function startSpotifyAuth() {
    if (!isTauri()) throw new Error("Spotify solo en la app de escritorio");
    spotifyAuthBusy.value = true;
    try {
      spotifyRedirectUri.value = spotifyRedirectUri.value.replace("localhost", "127.0.0.1");
      await saveSpotifyCredentials();
      await api.spotifyDisconnect();
      await api.spotifyAuthStart();
      const deadline = Date.now() + 180_000;
      while (Date.now() < deadline) {
        await new Promise((r) => setTimeout(r, 900));
        const poll = await api.spotifyPollAuth();
        if (!poll.ready) continue;
        if (poll.code) {
          const result = await api.spotifyConnect(poll.code);
          if (!result.ok) throw new Error(result.error ?? "No se pudo conectar Spotify");
          spotifyConnected.value = true;
          source.value = "spotify";
          startSpotifyPoll();
          return;
        }
        const errCode = poll.error ?? "no_code";
        const hint = await api.spotifyOauthHint(errCode);
        const detail =
          poll.errorDescription ??
          "Spotify no devolvio codigo. Si viste redirect_uri: Not matching configuration, guarda el URI en Dashboard con SAVE.";
        throw new Error(`${detail}\n\n${hint}`);
      }
      throw new Error(
        "Tiempo agotado. Si Spotify mostro 'redirect_uri: Not matching configuration':\n" +
          "1. Abri Dashboard > Settings de TU app\n" +
          "2. Redirect URIs → pega http://127.0.0.1:8888/callback → Add\n" +
          "3. Baja al final y pulsa SAVE (sin Save no se guarda)\n" +
          "4. Espera 30 segundos e intenta de nuevo",
      );
    } finally {
      spotifyAuthBusy.value = false;
    }
  }

  async function connectSpotifyManualCode(code: string) {
    await saveSpotifyCredentials();
    const result = await api.spotifyConnect(code.trim());
    if (!result.ok) throw new Error(result.error ?? "Codigo invalido");
    spotifyConnected.value = true;
    source.value = "spotify";
    startSpotifyPoll();
  }

  async function disconnectSpotify() {
    if (isTauri()) await api.spotifyDisconnect();
    spotifyConnected.value = false;
    spotifyNow.value = null;
    stopSpotifyPoll();
  }

  async function spotifyControl(action: "play" | "pause" | "next" | "prev") {
    if (!isTauri()) return;
    const mapped = action === "play" && spotifyPlaying.value ? "pause" : action;
    await api.spotifyControl(mapped === "play" ? "play" : mapped);
    setTimeout(() => void tickSpotify(), 500);
  }

  async function spotifyToggleShuffle() {
    if (!spotifyNow.value) return;
    await api.spotifyShuffle(!spotifyNow.value.shuffle);
    setTimeout(() => void tickSpotify(), 400);
  }

  async function spotifyToggleRepeat() {
    if (!spotifyNow.value) return;
    const order: Array<"off" | "context" | "track"> = ["off", "context", "track"];
    const idx = order.indexOf(spotifyNow.value.repeatState as "off" | "context" | "track");
    const next = order[(idx + 1) % order.length];
    await api.spotifyRepeat(next);
    setTimeout(() => void tickSpotify(), 400);
  }

  function togglePanel() {
    panelOpen.value = !panelOpen.value;
    if (panelOpen.value) void trySpotifyAutoconnect();
  }

  function closePanel() {
    panelOpen.value = false;
  }

  function loadFromInput(url?: string) {
    const value = (url ?? inputUrl.value).trim();
    if (!value) return false;
    const parsed = parseMusicUrl(value);
    if (!parsed) return false;
    inputUrl.value = value;
    source.value = "youtube";
    embedUrl.value = parsed.embedUrl;
    youtubeTrack.value = parsed;
    label.value = parsed.label;
    playing.value = true;
    return true;
  }

  function play() {
    if (isSpotifyMode.value) {
      void spotifyControl("play");
      return;
    }
    if (!embedUrl.value && inputUrl.value) {
      if (!loadFromInput()) return;
    }
    playing.value = Boolean(embedUrl.value);
  }

  function pause() {
    if (isSpotifyMode.value) {
      void spotifyControl("pause");
      return;
    }
    playing.value = false;
  }

  function togglePlay() {
    if (isPlaying.value) pause();
    else play();
  }

  function stop() {
    if (isSpotifyMode.value) {
      void spotifyControl("pause");
      return;
    }
    playing.value = false;
    embedUrl.value = null;
    youtubeTrack.value = null;
    source.value = null;
    label.value = "";
    destroyYoutube();
  }

  function setVolume(v: number) {
    volume.value = Math.max(0, Math.min(100, v));
  }

  async function init() {
    if (inputUrl.value && embedUrl.value) {
      youtubeTrack.value = parseMusicUrl(inputUrl.value);
    }
    await refreshSpotifyStatus();
    if (spotifyConnected.value || (await trySpotifyAutoconnect())) {
      startSpotifyPoll();
    }
  }

  return {
    panelOpen,
    source,
    embedUrl,
    youtubeTrack,
    inputUrl,
    label,
    playing,
    volume,
    launcherBackground,
    launcherOverlay,
    inGameOverlay,
    spotifyConnected,
    spotifyClientId,
    spotifyRedirectUri,
    spotifyClientSecret,
    spotifyNow,
    spotifyAuthBusy,
    isSpotifyMode,
    spotifyPlaying,
    spotifyLabel,
    spotifyProgressPct,
    hasTrack,
    isPlaying,
    overlayLabel,
    overlayImage,
    showOverlay,
    shouldPlayAudio,
    init,
    trySpotifyAutoconnect,
    validateSpotifyCredentials,
    startSpotifyAuth,
    connectSpotifyManualCode,
    disconnectSpotify,
    spotifyControl,
    spotifyToggleShuffle,
    spotifyToggleRepeat,
    tickSpotify,
    togglePanel,
    closePanel,
    loadFromInput,
    play,
    pause,
    togglePlay,
    stop,
    setVolume,
  };
});
