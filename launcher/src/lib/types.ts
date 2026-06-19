// Tipos compartidos del dominio del launcher.
// Estos tipos espejan (a futuro) los structs `serde` del backend Rust.

export type LoaderId =
  | "vanilla"
  | "forge"
  | "fabric"
  | "neoforge"
  | "optifine"
  | "quilt"
  | "fabric-iris"
  | "paraguacraft-pvp";

export type GcType = "Auto" | "G1GC" | "ZGC" | "Shenandoah";

export type HardwareTier = "baja" | "media" | "alta";

export interface HardwareInfo {
  ramGb: number;
  cpuCores: number;
  cpuThreads: number;
  cpuName: string;
  gpuName: string;
  os: string;
  arch: string;
  perfilSugerido: HardwareTier;
  // Autoconfig sugerida derivada del hardware (la real la calcula Rust en Fase 2)
  recommendedRamMb: number;
  recommendedGc: GcType;
}

export type VersionChannel = "release" | "snapshot" | "old_beta" | "old_alpha";

export interface MinecraftVersion {
  id: string;
  channel: VersionChannel;
  releaseDate: string;
  installed: boolean;
}

export interface LoaderInfo {
  id: LoaderId;
  name: string;
  description: string;
  // Versiones exactas del loader disponibles para la versión de MC seleccionada
  versions: string[];
}

export type AccountType = "microsoft" | "offline";

export interface Account {
  id: string;
  type: AccountType;
  username: string;
  uuid: string;
  avatarUrl: string;
  active: boolean;
  premium: boolean;
}

export type InstanceSource =
  | "paraguacraft"
  | "vanilla"
  | "lunar"
  | "prism"
  | "tlauncher"
  | "sklauncher"
  | "curseforge"
  | "modrinth";

export interface Instance {
  id: string;
  name: string;
  icon: string; // emoji o data-url
  mcVersion: string;
  loader: LoaderId;
  loaderVersion: string;
  source: InstanceSource;
  lastPlayed: string | null;
  totalPlayMinutes: number;
  ramMb: number;
  modCount: number;
}

export type ContentType =
  | "mod"
  | "modpack"
  | "resourcepack"
  | "shader"
  | "datapack"
  | "plugin";

export type ContentProvider = "modrinth" | "curseforge";

export interface StoreItem {
  id: string;
  slug: string;
  title: string;
  author: string;
  description: string;
  iconUrl: string;
  downloads: number;
  followers: number;
  type: ContentType;
  provider: ContentProvider;
  categories: string[];
  loaders: LoaderId[];
}

export type DownloadStatus = "queued" | "downloading" | "done" | "error" | "paused";

export interface DownloadTask {
  id: string;
  label: string;
  progress: number; // 0-100
  status: DownloadStatus;
  speed: string;
}

export interface AppSettings {
  ramMb: number;
  gcType: GcType;
  javaPath: string | null;
  closeOnLaunch: boolean;
  optimizeGraphics: boolean;
  gpuCompatMode: "off" | "mesa-d3d12" | "mesa-llvmpipe" | "mesa-zink";
  theme: "dark" | "darker";
  accent: "green" | "ai";
  language: "es" | "en" | "pt";
}

export interface AiMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
}
