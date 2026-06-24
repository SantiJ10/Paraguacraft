import type { Instance, MinecraftVersion, VersionChannel } from "@/lib/types";
import { normalizeLoaderId } from "@/lib/loaders";

/** Tarjetas destacadas estilo Lunar (PARAGUA X.X). */
export const FEATURED_VERSION_KEYS = [
  "26",
  "1.21",
  "1.20",
  "1.19",
  "1.18",
  "1.17",
  "1.16",
  "1.12",
  "1.8",
  "1.7",
] as const;

export type FeaturedVersionKey = (typeof FEATURED_VERSION_KEYS)[number];

export type VersionCardKind =
  | "featured"
  | "bedrock"
  | "installed"
  | "snapshots"
  | "alpha_beta"
  | "other";

export interface VersionCardModel {
  id: string;
  kind: VersionCardKind;
  title: string;
  subtitle?: string;
  description: string;
  accent?: string;
  subs: string[];
  instances?: Instance[];
  imageKey: string;
}

const DESCRIPTIONS: Record<string, string> = {
  "26": "The 2026 Game Drops — todas las releases oficiales 26.x.",
  "1.21": "Trial Chambers y contenido reciente.",
  "1.20": "Trails & Tales.",
  "1.19": "The Wild Update.",
  "1.18": "Caves & Cliffs Parte 2.",
  "1.17": "Caves & Cliffs Parte 1.",
  "1.16": "The Nether Update.",
  "1.12": "World of Color.",
  "1.8": "PvP clásico.",
  "1.7": "The Update that Changed the World.",
  otras: "Todas las demás versiones release que no están en los packs PARAGUA.",
  snapshots: "Versiones de desarrollo Mojang. Inestables — para probar features futuras.",
  alpha_beta: "Alpha y Beta clásicas. El Minecraft de los inicios.",
  instaladas: "Versiones que ya tenés instaladas con su loader detectado.",
  bedrock: "Minecraft: Bedrock Edition · Xbox / Microsoft Store",
};

/** Nombre de archivo PNG en public/web-assets (1.21 → 1_21). */
export function versionImageFile(key: string): string {
  if (key === "bedrock") return "minecraft_bedrock";
  if (key === "26") return "26_1";
  if (key === "instaladas" || key === "otras" || key === "snapshots" || key === "alpha_beta") {
    return "Otra_Version";
  }
  return key.replace(/\./g, "_");
}

export function versionCardImageUrl(key: string): string {
  return `/web-assets/${versionImageFile(key)}.png`;
}

export function paraguaTitle(key: string): string {
  if (key === "bedrock") return "PARAGUA Bedrock";
  if (key === "instaladas") return "INSTALADAS";
  if (key === "snapshots") return "SNAPSHOTS";
  if (key === "alpha_beta") return "ALPHA / BETA";
  if (key === "otras") return "OTRAS VERSIONES";
  return `PARAGUA ${key}`;
}

function groupKey(versionId: string): string | null {
  if (/^26\.\d/.test(versionId)) return "26";
  const parts = versionId.split(".");
  if (parts.length >= 2 && /^\d+$/.test(parts[0]!) && /^\d+$/.test(parts[1]!)) {
    return `${parts[0]}.${parts[1]}`;
  }
  return null;
}

/** Releases oficiales 26.x (26.1.2, 26.2, etc.). */
export function isMc26Release(versionId: string): boolean {
  return /^26\.\d/.test(versionId);
}

function isAlphaBeta(_id: string, channel: VersionChannel): boolean {
  return channel === "old_beta" || channel === "old_alpha";
}

export function buildVersionCards(
  allVersions: MinecraftVersion[],
  instances: Instance[],
): VersionCardModel[] {
  const releases = allVersions.filter((v) => v.channel === "release");
  const snapshots = allVersions.filter((v) => v.channel === "snapshot");
  const legacy = allVersions.filter((v) => isAlphaBeta(v.id, v.channel));

  const grouped = new Map<string, string[]>();
  for (const v of releases) {
    const g = groupKey(v.id);
    if (!g) continue;
    const list = grouped.get(g) ?? [];
    list.push(v.id);
    grouped.set(g, list);
  }

  const featuredSet = new Set<string>(FEATURED_VERSION_KEYS);
  const otherReleases: string[] = [];
  for (const v of releases) {
    const g = groupKey(v.id);
    if (!g || !featuredSet.has(g as FeaturedVersionKey)) {
      otherReleases.push(v.id);
    }
  }

  const cards: VersionCardModel[] = [];

  for (const key of FEATURED_VERSION_KEYS) {
    const subs =
      key === "26"
        ? (grouped.get("26") ?? releases.filter((v) => isMc26Release(v.id)).map((v) => v.id))
        : (grouped.get(key) ?? [key]);
    cards.push({
      id: key,
      kind: "featured",
      title: paraguaTitle(key),
      description: DESCRIPTIONS[key] ?? "",
      subs: subs.sort((a, b) => b.localeCompare(a, undefined, { numeric: true })),
      imageKey: key,
    });
  }

  if (otherReleases.length) {
    cards.push({
      id: "otras",
      kind: "other",
      title: paraguaTitle("otras"),
      description: DESCRIPTIONS.otras!,
      subs: otherReleases.sort((a, b) => b.localeCompare(a, undefined, { numeric: true })),
      imageKey: "otras",
    });
  }

  if (snapshots.length) {
    cards.push({
      id: "snapshots",
      kind: "snapshots",
      title: paraguaTitle("snapshots"),
      description: DESCRIPTIONS.snapshots!,
      accent: "#9B59B6",
      subs: snapshots.map((v) => v.id).sort((a, b) => b.localeCompare(a, undefined, { numeric: true })),
      imageKey: "snapshots",
    });
  }

  if (legacy.length) {
    cards.push({
      id: "alpha_beta",
      kind: "alpha_beta",
      title: paraguaTitle("alpha_beta"),
      description: DESCRIPTIONS.alpha_beta!,
      accent: "#F39C12",
      subs: legacy.map((v) => v.id).sort((a, b) => b.localeCompare(a, undefined, { numeric: true })),
      imageKey: "alpha_beta",
    });
  }

  const pcInstances = instances.filter((i) => i.source === "paraguacraft");
  if (pcInstances.length) {
    cards.push({
      id: "instaladas",
      kind: "installed",
      title: paraguaTitle("instaladas"),
      description: DESCRIPTIONS.instaladas!,
      subs: pcInstances.map((i) => i.mcVersion),
      instances: pcInstances,
      imageKey: "instaladas",
    });
  }

  cards.push({
    id: "bedrock",
    kind: "bedrock",
    title: paraguaTitle("bedrock"),
    description: DESCRIPTIONS.bedrock!,
    accent: "#3498DB",
    subs: [],
    imageKey: "bedrock",
  });

  return cards;
}

export function matchMcGroup(mcVersion: string, cardId: string): boolean {
  if (cardId === "otras" || cardId === "instaladas" || cardId === "bedrock") return false;
  if (cardId === "snapshots" || cardId === "alpha_beta") return false;
  if (cardId === "26") return isMc26Release(mcVersion);
  return mcVersion === cardId || mcVersion.startsWith(`${cardId}.`) || groupKey(mcVersion) === cardId;
}

export function findInstanceForVersion(
  instances: Instance[],
  mcVersion: string,
  loader?: string,
): Instance | undefined {
  const want = loader ? normalizeLoaderId(loader) : undefined;
  return instances.find((i) => {
    if (i.source !== "paraguacraft") return false;
    if (i.mcVersion !== mcVersion) return false;
    if (want && normalizeLoaderId(i.loader) !== want) return false;
    return true;
  });
}
