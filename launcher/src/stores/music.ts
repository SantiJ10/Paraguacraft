import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import { parseMusicUrl, type ParsedMusic } from "@/lib/musicUrl";
import type { SpotifyNowPlaying } from "@/lib/types";
import { api, isTauri } from "@/lib/ipc";
import {
  destroyYoutube,
  getYoutubeNow,
  youtubeThumbnail,
  type YoutubeNow,
} from "@/lib/youtubePlayer";

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

const SMART_DEFAULTS_KEY = "paraguacraft.music.smartDefaults.v1";

/** Defaults de música según gama de hardware (solo primera vez). */
export function applyHardwareSmartDefaults(tier: string) {
  try {
    if (localStorage.getItem(SMART_DEFAULTS_KEY)) return;
    const raw = localStorage.getItem(STORAGE_KEY);
    const prev = raw ? (JSON.parse(raw) as Partial<MusicPersist>) : {};
    const next: MusicPersist = {
      inputUrl: prev.inputUrl ?? "",
      source: prev.source ?? null,
      embedUrl: prev.embedUrl ?? null,
      label: prev.label ?? "",
      playing: prev.playing ?? false,
      volume: prev.volume ?? 70,
      launcherBackground:
        tier === "baja" ? false : (prev.launcherBackground ?? true),
      launcherOverlay: tier === "baja" ? false : (prev.launcherOverlay ?? true),
      inGameOverlay: prev.inGameOverlay ?? true,
      spotifyClientId: prev.spotifyClientId ?? "",
      spotifyRedirectUri: prev.spotifyRedirectUri ?? SPOTIFY_REDIRECT_URI,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    localStorage.setItem(SMART_DEFAULTS_KEY, tier);
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
  const youtubeNow = ref<YoutubeNow | null>(null);
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
  let youtubePollTimer: number | null = null;

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

  // YouTube y Spotify son reproductores INDEPENDIENTES; el overlay sigue al que
  // realmente está sonando ahora.
  const youtubePlaying = computed(
    () => source.value === "youtube" && Boolean(embedUrl.value) && playing.value,
  );
  const activeSource = computed<"youtube" | "spotify" | null>(() => {
    if (youtubePlaying.value) return "youtube";
    if (spotifyPlaying.value) return "spotify";
    if (spotifyConnected.value && spotifyNow.value?.title) return "spotify";
    if (embedUrl.value) return "youtube";
    return null;
  });

  const isPlaying = computed(() => {
    if (activeSource.value === "youtube") return youtubePlaying.value;
    if (activeSource.value === "spotify") return spotifyPlaying.value;
    return false;
  });
  const youtubeLabel = computed(() => {
    const n = youtubeNow.value;
    if (n?.title) return n.author ? `${n.title} · ${n.author}` : n.title;
    return label.value || "YouTube";
  });
  const overlayTitle = computed(() => {
    if (activeSource.value === "youtube") {
      return youtubeNow.value?.title || label.value || "YouTube";
    }
    if (spotifyNow.value?.title) return spotifyNow.value.title;
    return label.value || "Reproduciendo";
  });
  const overlayArtist = computed(() => {
    if (activeSource.value === "youtube") {
      return youtubeNow.value?.author || "YouTube";
    }
    if (spotifyNow.value?.artist) return spotifyNow.value.artist;
    return activeSource.value === "spotify" ? "Spotify" : "YouTube";
  });
  const overlayLabel = computed(() => {
    if (activeSource.value === "youtube") return youtubeLabel.value;
    if (spotifyNow.value?.title) return spotifyLabel.value;
    return label.value;
  });
  const overlayImage = computed(() => {
    if (activeSource.value === "youtube") return youtubeNow.value?.thumbnail || null;
    return spotifyNow.value?.imageUrl ?? null;
  });
  const showOverlay = computed(() => isPlaying.value && launcherOverlay.value && hasTrack.value);

  function shouldPlayAudio(inGame: boolean) {
    // YouTube es independiente de Spotify: suena si hay un video cargado y en play,
    // sin importar si Spotify está conectado (antes esto lo bloqueaba por completo).
    if (source.value !== "youtube" || !playing.value || !embedUrl.value) return false;
    if (inGame) return inGameOverlay.value;
    return launcherBackground.value;
  }

  function syncOverlayIpc() {
    if (!isTauri()) return;
    const active = isPlaying.value && inGameOverlay.value;
    if (activeSource.value === "youtube") {
      const n = youtubeNow.value;
      void api.syncOverlayMusic(
        active,
        n?.title || label.value || "YouTube",
        n?.author || "YouTube",
        n?.thumbnail || "",
      );
    } else if (activeSource.value === "spotify") {
      const t = spotifyNow.value;
      void api.syncOverlayMusic(
        active,
        t?.title || label.value || "Spotify",
        t?.artist || "Spotify",
        t?.imageUrl ?? "",
      );
    } else {
      void api.syncOverlayMusic(false, "", "", "");
    }
  }

  function updateYoutubeNow() {
    const now = getYoutubeNow();
    const vid = now?.videoId || youtubeTrack.value?.videoId || "";
    if (now && now.title) {
      youtubeNow.value = {
        title: now.title,
        author: now.author,
        videoId: vid,
        thumbnail: youtubeThumbnail(vid),
        playing: now.playing,
      };
    } else if (vid) {
      // Antes de que la IFrame API devuelva metadata: al menos mostramos la
      // miniatura por el videoId y el label parseado.
      youtubeNow.value = {
        title: label.value || "YouTube",
        author: "",
        videoId: vid,
        thumbnail: youtubeThumbnail(vid),
        playing: playing.value,
      };
    }
    syncOverlayIpc();
  }

  function stopYoutubePoll() {
    if (youtubePollTimer !== null) {
      clearInterval(youtubePollTimer);
      youtubePollTimer = null;
    }
  }

  function startYoutubePoll() {
    stopYoutubePoll();
    updateYoutubeNow();
    youtubePollTimer = window.setInterval(updateYoutubeNow, 2500);
  }

  watch(
    [isPlaying, inGameOverlay, label, spotifyNow, playing, isSpotifyMode, youtubeNow],
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
    const vid = parsed.videoId ?? "";
    youtubeNow.value = {
      title: parsed.label,
      author: "",
      videoId: vid,
      thumbnail: youtubeThumbnail(vid),
      playing: true,
    };
    startYoutubePoll();
    return true;
  }

  function play() {
    // Controla la fuente activa; si Spotify está conectado pero quien suena es
    // YouTube, reanuda YouTube (reproductores independientes).
    if (activeSource.value === "spotify" || (isSpotifyMode.value && !embedUrl.value)) {
      void spotifyControl("play");
      return;
    }
    if (!embedUrl.value && inputUrl.value) {
      if (!loadFromInput()) return;
    }
    playing.value = Boolean(embedUrl.value);
  }

  function pause() {
    if (activeSource.value === "spotify") {
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
    youtubeNow.value = null;
    source.value = null;
    label.value = "";
    stopYoutubePoll();
    destroyYoutube();
    syncOverlayIpc();
  }

  function setVolume(v: number) {
    volume.value = Math.max(0, Math.min(100, v));
  }

  async function init() {
    if (inputUrl.value && embedUrl.value) {
      youtubeTrack.value = parseMusicUrl(inputUrl.value);
      const vid = youtubeTrack.value?.videoId ?? "";
      if (vid) {
        youtubeNow.value = {
          title: label.value || "YouTube",
          author: "",
          videoId: vid,
          thumbnail: youtubeThumbnail(vid),
          playing: playing.value,
        };
      }
      startYoutubePoll();
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
    youtubeNow,
    activeSource,
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
    overlayTitle,
    overlayArtist,
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
