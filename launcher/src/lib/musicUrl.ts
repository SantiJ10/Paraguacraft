export type MusicSource = "youtube";

export interface ParsedMusic {
  source: MusicSource;
  embedUrl: string;
  id: string;
  label: string;
  kind: "video" | "playlist";
  videoId?: string;
  playlistId?: string;
}

function originParam(): string {
  try {
    return encodeURIComponent(window.location.origin);
  } catch {
    return "";
  }
}

function buildEmbed(extra: Record<string, string>): string {
  const q = new URLSearchParams({
    enablejsapi: "1",
    autoplay: "1",
    rel: "0",
    modestbranding: "1",
    playsinline: "1",
    ...extra,
  });
  const origin = originParam();
  if (origin) q.set("origin", decodeURIComponent(origin));
  return q.toString();
}

function videoResult(id: string, label?: string): ParsedMusic {
  const qs = buildEmbed({});
  return {
    source: "youtube",
    id,
    kind: "video",
    videoId: id,
    embedUrl: `https://www.youtube.com/embed/${id}?${qs}`,
    label: label ?? `YouTube · ${id}`,
  };
}

function playlistResult(listId: string): ParsedMusic {
  const qs = buildEmbed({ list: listId, listType: "playlist" });
  return {
    source: "youtube",
    id: listId,
    kind: "playlist",
    playlistId: listId,
    embedUrl: `https://www.youtube.com/embed/videoseries?${qs}`,
    label: "YouTube · playlist",
  };
}

function mixResult(videoId: string, listId: string): ParsedMusic {
  const qs = buildEmbed({ list: listId });
  return {
    source: "youtube",
    id: videoId,
    kind: "video",
    videoId,
    playlistId: listId,
    embedUrl: `https://www.youtube.com/embed/${videoId}?${qs}`,
    label: `YouTube · ${videoId}`,
  };
}

const VIDEO_ID = /^[a-zA-Z0-9_-]{11}$/;

export function parseMusicUrl(raw: string): ParsedMusic | null {
  const input = raw.trim();
  if (!input) return null;

  if (VIDEO_ID.test(input)) return videoResult(input);

  let url: URL;
  try {
    const withProto = /^https?:\/\//i.test(input) ? input : `https://${input}`;
    url = new URL(withProto);
  } catch {
    return null;
  }

  const host = url.hostname.replace(/^www\./i, "").toLowerCase();

  if (host === "youtu.be") {
    const id = url.pathname.split("/").filter(Boolean)[0]?.split("?")[0];
    if (id && VIDEO_ID.test(id)) return videoResult(id);
    return null;
  }

  if (!host.includes("youtube.com") && !host.includes("youtube-nocookie.com")) {
    return null;
  }

  const list = url.searchParams.get("list");
  const v = url.searchParams.get("v");

  const pathMatch =
    url.pathname.match(/\/(?:shorts|embed|live|v)\/([a-zA-Z0-9_-]{11})/i) ??
    url.pathname.match(/\/watch\/([a-zA-Z0-9_-]{11})/i);
  const pathId = pathMatch?.[1];

  if (url.pathname.includes("/playlist") && list) return playlistResult(list);

  const videoId =
    (v && VIDEO_ID.test(v) ? v : null) ?? (pathId && VIDEO_ID.test(pathId) ? pathId : null);

  if (videoId && list) return mixResult(videoId, list);
  if (videoId) return videoResult(videoId);
  if (list) return playlistResult(list);

  return null;
}

export function youtubeCommand(iframe: HTMLIFrameElement, func: string, args: unknown[] = []) {
  iframe.contentWindow?.postMessage(JSON.stringify({ event: "command", func, args }), "*");
}
