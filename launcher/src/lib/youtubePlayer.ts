import type { ParsedMusic } from "@/lib/musicUrl";

declare global {
  interface Window {
    YT?: {
      Player: new (el: HTMLElement | string, opts: Record<string, unknown>) => YtPlayer;
      PlayerState: { PLAYING: number; PAUSED: number; ENDED: number };
    };
    onYouTubeIframeAPIReady?: () => void;
  }
}

interface YtPlayer {
  playVideo(): void;
  pauseVideo(): void;
  stopVideo(): void;
  setVolume(n: number): void;
  getPlayerState(): number;
  destroy(): void;
}

let apiPromise: Promise<void> | null = null;
let active: { player: YtPlayer; trackKey: string } | null = null;

function loadApi(): Promise<void> {
  if (window.YT?.Player) return Promise.resolve();
  if (apiPromise) return apiPromise;
  apiPromise = new Promise((resolve) => {
    const prev = window.onYouTubeIframeAPIReady;
    window.onYouTubeIframeAPIReady = () => {
      prev?.();
      resolve();
    };
    if (document.querySelector('script[src*="youtube.com/iframe_api"]')) {
      const poll = setInterval(() => {
        if (window.YT?.Player) {
          clearInterval(poll);
          resolve();
        }
      }, 100);
      return;
    }
    const tag = document.createElement("script");
    tag.src = "https://www.youtube.com/iframe_api";
    tag.async = true;
    document.head.appendChild(tag);
  });
  return apiPromise;
}

function trackKey(track: ParsedMusic): string {
  return `${track.kind}:${track.videoId ?? ""}:${track.playlistId ?? track.id}`;
}

export async function mountYoutube(
  host: HTMLElement,
  track: ParsedMusic,
  volume: number,
  autoplay: boolean,
): Promise<YtPlayer> {
  await loadApi();
  const key = trackKey(track);
  if (active?.trackKey === key) {
    active.player.setVolume(volume);
    if (autoplay) active.player.playVideo();
    else active.player.pauseVideo();
    return active.player;
  }
  destroyYoutube();
  host.innerHTML = "";

  const playerVars: Record<string, string | number> = {
    autoplay: autoplay ? 1 : 0,
    controls: 0,
    disablekb: 1,
    fs: 0,
    rel: 0,
    modestbranding: 1,
    playsinline: 1,
  };

  const playerOpts: Record<string, unknown> = {
    height: "180",
    width: "320",
    playerVars,
    events: {
      onReady: (ev: { target: YtPlayer }) => {
        ev.target.setVolume(volume);
        if (autoplay) ev.target.playVideo();
      },
    },
  };

  if (track.kind === "playlist" && track.playlistId) {
    playerOpts.listType = "playlist";
    playerOpts.list = track.playlistId;
  } else if (track.videoId) {
    playerOpts.videoId = track.videoId;
    if (track.playlistId) playerVars.list = track.playlistId;
  }

  const player = new window.YT!.Player(host, playerOpts);
  active = { player, trackKey: key };
  return player;
}

export function controlYoutube(playing: boolean, volume: number) {
  if (!active) return;
  active.player.setVolume(volume);
  if (playing) active.player.playVideo();
  else active.player.pauseVideo();
}

export function destroyYoutube() {
  if (active) {
    try {
      active.player.destroy();
    } catch {
      /* ignore */
    }
    active = null;
  }
}
