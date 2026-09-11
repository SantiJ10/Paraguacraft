import { api } from "@/lib/ipc";
import pgWallpaper from "@/assets/pg_wallpaper.jpg";

const THEME_STYLE_ID = "pc-custom-theme";
const ICON_STYLE_ID = "pc-custom-icons";

export const WALLPAPER_IDS = ["none", "banner", "custom"] as const;
export type WallpaperId = (typeof WALLPAPER_IDS)[number];

export function normalizeWallpaper(id?: string): WallpaperId {
  if (id === "custom") return "custom";
  if (id === "none") return "none";
  return "banner";
}

export function applyWallpaper(id?: string) {
  const w = normalizeWallpaper(id);
  document.documentElement.dataset.wallpaper = w;
  document.documentElement.classList.toggle("has-wallpaper", w !== "none");
}

export function wallpaperCss(id?: string, customUrl?: string): string {
  const w = normalizeWallpaper(id);
  const dim = "linear-gradient(to top, rgb(10 10 10 / 0.72), rgb(10 10 10 / 0.38))";
  if (w === "none") return "";
  if (w === "custom" && customUrl) return `${dim}, url("${customUrl}")`;
  return `${dim}, url(${pgWallpaper})`;
}

export async function wallpaperCustomSrc(path?: string): Promise<string> {
  if (!path?.trim()) return "";
  try {
    const { convertFileSrc } = await import("@tauri-apps/api/core");
    return convertFileSrc(path);
  } catch {
    return "";
  }
}

export function applyTheme(theme: string) {
  const t = theme === "darker" || theme === "light" ? theme : "dark";
  document.documentElement.dataset.theme = t;
}

export function applyIconStyle(style: string) {
  document.documentElement.dataset.iconStyle = style === "simple" ? "simple" : "filled";
}

function setInjectedCss(id: string, css: string) {
  let el = document.getElementById(id) as HTMLStyleElement | null;
  if (!css.trim()) {
    el?.remove();
    return;
  }
  if (!el) {
    el = document.createElement("style");
    el.id = id;
    document.head.appendChild(el);
  }
  el.textContent = css;
}

export async function applyCustomCss(kind: "theme" | "icons", name: string | undefined) {
  const id = kind === "icons" ? ICON_STYLE_ID : THEME_STYLE_ID;
  if (!name?.trim()) {
    setInjectedCss(id, "");
    return;
  }
  try {
    const css = await api.readCustomThemeCss(kind, name);
    setInjectedCss(id, css);
  } catch {
    setInjectedCss(id, "");
  }
}
