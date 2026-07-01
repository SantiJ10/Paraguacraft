import fs from "node:fs";
import path from "node:path";

export type LatestManifest = {
  version: string;
  download_url: string;
  size_bytes?: number;
  sha256?: string;
  notes?: string;
  platforms?: Record<string, { url: string; signature?: string }>;
};

const FALLBACK: LatestManifest = {
  version: "7.0.2",
  download_url:
    "https://github.com/SantiJ10/Paraguacraft/releases/download/v7.0.2/Instalar_Paraguacraft_v7.0.2.exe",
};

export function loadLatestManifest(): LatestManifest {
  const p = path.resolve("./public/latest.json");
  try {
    return { ...FALLBACK, ...JSON.parse(fs.readFileSync(p, "utf8")) };
  } catch {
    return FALLBACK;
  }
}

export function parseReleaseNotes(notes?: string): string[] {
  if (!notes) return [];
  return notes
    .split("\n")
    .map((l) => l.replace(/^[-*]\s*/, "").trim())
    .filter(Boolean);
}

export function formatSize(bytes?: number): string | null {
  if (!bytes) return null;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
