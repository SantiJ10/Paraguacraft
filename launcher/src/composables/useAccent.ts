import type { AppSettings } from "@/lib/types";

const ACCENTS = {
  green: {
    primary: "#2ECC71",
    hover: "#27AE60",
    dark: "#1A7A40",
    glow: "0 10px 25px -5px rgba(46, 204, 113, 0.25)",
  },
  ai: {
    primary: "#9B59B6",
    hover: "#8E44AD",
    dark: "#7D3C98",
    glow: "0 10px 25px -5px rgba(155, 89, 182, 0.25)",
  },
  blue: {
    primary: "#3B82F6",
    hover: "#2563EB",
    dark: "#1D4ED8",
    glow: "0 10px 25px -5px rgba(59, 130, 246, 0.28)",
  },
  orange: {
    primary: "#F59E0B",
    hover: "#D97706",
    dark: "#B45309",
    glow: "0 10px 25px -5px rgba(245, 158, 11, 0.28)",
  },
} as const;

export type AccentId = keyof typeof ACCENTS;

export const ACCENT_SWATCHES: Array<{ id: AccentId; color: string }> = [
  { id: "green", color: ACCENTS.green.primary },
  { id: "ai", color: ACCENTS.ai.primary },
  { id: "blue", color: ACCENTS.blue.primary },
  { id: "orange", color: ACCENTS.orange.primary },
];

export function applyAccentTheme(accent: AppSettings["accent"]) {
  const palette = ACCENTS[accent] ?? ACCENTS.green;
  const root = document.documentElement;
  root.style.setProperty("--pc-accent", palette.primary);
  root.style.setProperty("--pc-accent-hover", palette.hover);
  root.style.setProperty("--pc-accent-dark", palette.dark);
  root.style.setProperty("--pc-accent-glow", palette.glow);
  root.dataset.accent = accent;
}
