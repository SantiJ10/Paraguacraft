import type { LoaderId } from "@/lib/types";
import vanilla from "@/assets/loader-icons/vanilla.png";
import fabric from "@/assets/loader-icons/fabric.png";
import iris from "@/assets/loader-icons/iris.png";
import forge from "@/assets/loader-icons/forge.png";
import neoforge from "@/assets/loader-icons/neoforge.png";
import quilt from "@/assets/loader-icons/quilt.png";
import optifine from "@/assets/loader-icons/optifine.png";
import tnt from "@/assets/instance-icons/tnt.svg";

import { normalizeLoaderId } from "@/lib/loaders";

export interface LoaderIconOption {
  id: LoaderId | string;
  name: string;
  src: string;
  description?: string;
}

export const LOADER_ICONS: LoaderIconOption[] = [
  { id: "vanilla", name: "Vanilla", src: vanilla, description: "Minecraft sin mods" },
  { id: "fabric", name: "Fabric", src: fabric, description: "Loader liviano" },
  { id: "fabric-iris", name: "Fabric + Iris", src: iris, description: "Sodium + Iris" },
  { id: "forge", name: "Forge", src: forge, description: "Mods clásicos" },
  { id: "neoforge", name: "NeoForge", src: neoforge, description: "Forge moderno" },
  { id: "quilt", name: "Quilt", src: quilt, description: "Fork de Fabric" },
  { id: "optifine", name: "OptiFine", src: optifine, description: "Optimización + shaders" },
  { id: "paraguacraft-pvp", name: "Paraguacraft PvP 1.8.9", src: tnt, description: "Cliente PvP Forge" },
  { id: "paraguacraft-pvp-modern", name: "Paraguacraft PvP 1.21.11", src: tnt, description: "Cliente PvP Fabric" },
];

const ICON_BY_ID: Record<string, string> = Object.fromEntries(
  LOADER_ICONS.map((l) => [l.id, l.src]),
);

export function loaderIconSrc(loaderId: string): string {
  const id = normalizeLoaderId(loaderId);
  if (id in ICON_BY_ID) return ICON_BY_ID[id]!;
  return vanilla;
}

export function loaderDisplayName(loaderId: string): string {
  const id = normalizeLoaderId(loaderId);
  const hit = LOADER_ICONS.find((l) => l.id === id);
  return hit?.name ?? loaderId.replace(/-/g, " ");
}
