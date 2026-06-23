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
  // Versión recomendada (la más reciente/estable), si aplica
  recommended?: string | null;
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

export interface ImportInstanceIconResult {
  iconId: string;
  path: string;
  width: number;
  height: number;
}

export interface BedrockStatus {
  platformSupported: boolean;
  installed: boolean;
  premiumAllowed: boolean;
  username: string | null;
}

export interface Instance {
  id: string;
  name: string;
  icon: string; // mc:block id o emoji legacy
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
  follows: number;
  projectType: ContentType;
  provider: ContentProvider;
  categories: string[];
  projectUrl?: string | null;
}

export interface StoreVersion {
  id: string;
  name: string;
  versionNumber: string;
  filename: string;
  gameVersions: string[];
  loaders: string[];
  publishedAt: string;
}

export interface WorldInfo {
  name: string;
  active: boolean;
}

export interface ServerWorldsResult {
  worlds: WorldInfo[];
  defaultWorld: string;
}

export type StoreInstallDestination = "instance" | "server";

/** Error estructurado del backend (p. ej. CurseForge distribution blocked). */
export interface StructuredInvokeError {
  code: string;
  message: string;
  projectUrl?: string | null;
  projectName?: string | null;
}

export interface CrashDiagnosis {
  category: string;
  message: string;
  hint: string;
  errorLine?: string | null;
  exitCode: number;
  crashFile: string | null;
  logTail: string;
  suggestions: string[];
}

export interface SkinProfile {
  username: string;
  uuid: string;
  premium: boolean;
  avatarUrl: string;
  bodyUrl: string;
  localAvatarPath?: string | null;
  avatarDataUrl?: string | null;
  skinUrl?: string | null;
  model?: string | null;
}

export interface SkinLookup {
  ok: boolean;
  username: string;
  uuid: string;
  skinUrl: string | null;
  capeUrl: string | null;
  model: string;
  error?: string | null;
}

export interface SkinCatalogEntry {
  id: string;
  label: string;
  skinUrl: string;
  previewUrl: string;
  kind: "player" | "mineskin";
  model?: string | null;
}

export interface SkinCatalogPage {
  entries: SkinCatalogEntry[];
  page: number;
  totalPages: number;
  totalSkins: number;
  query: string;
}

export interface SkinHistoryEntry {
  nombre: string;
  url: string;
  tipo: string;
}

export interface ApplySkinResult {
  ok: boolean;
  message: string;
  instances: number;
  serverSync: number;
  premium: boolean;
}

export interface UpdateInfo {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  downloadUrl: string | null;
  releaseNotes: string;
  publishedAt: string;
  inAppInstall: boolean;
  assetName: string | null;
}

export interface UpdateProgress {
  phase: string;
  progress: number;
  message: string;
}

export type ServerType = "paper" | "paper-geyser" | "fabric" | "fabric-geyser" | "forge";

export interface ServerProfile {
  id: string;
  name: string;
  mcVersion: string;
  serverType: ServerType | string;
  ramMb: number;
  port: number;
  createdAt: number;
  playitAddress: string | null;
  customFolder?: string | null;
}

export interface ServerStatus {
  id: string;
  running: boolean;
  playitRunning: boolean;
  playitAddress: string | null;
  pid: number | null;
}

export interface ServerContentItem {
  name: string;
  path: string;
  sizeBytes: number;
  kind: string;
}

export interface HangarPlugin {
  slug: string;
  owner: string;
  name: string;
  description: string;
  downloads: number;
  stars: number;
}

export interface ServerBackupResult {
  path: string;
  sizeMb: number;
}

export interface ServerRepairItem {
  severity: "fixed" | "warning" | "error" | "info" | string;
  title: string;
  detail: string;
  path?: string | null;
}

export interface ServerRepairReport {
  items: ServerRepairItem[];
  fixedCount: number;
  warningCount: number;
}

export interface AiAssistResponse {
  message: string;
  suggestions: string[];
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
  curseforgeApiKey?: string | null;
  downloadConcurrency?: number;
  autoUpdateCheck?: boolean;
  hardwareDefaultsApplied?: boolean;
  discordRpc?: boolean;
  discordRpcVersion?: boolean;
  discordRpcTime?: boolean;
  papaMode?: boolean;
  deepCleanOnLaunch?: boolean;
  backupAutoHours?: number;
  javaPriority?: string;
}

export interface ExtrasStatus {
  gameModeActive: boolean;
  turboActive: boolean;
  javaPriority: string;
}

export interface CleanupInfo {
  logsMb: number;
  crashMb: number;
  mcRamMb: number;
}

export interface SpotifyStatus {
  connected: boolean;
  clientId: string | null;
  redirectUri: string;
}

export interface SpotifySetupInfo {
  redirectUri: string;
  dashboardUrl: string;
  authUrlPreview: string | null;
  clientIdOk: boolean;
  clientIdLength: number;
}

export interface SpotifySimpleResult {
  ok: boolean;
  error?: string | null;
}

export interface SpotifyOAuthPoll {
  ready: boolean;
  code?: string | null;
  error?: string | null;
  errorDescription?: string | null;
}

export interface SpotifyNowPlaying {
  ok: boolean;
  error?: string | null;
  playing: boolean;
  title?: string | null;
  artist?: string | null;
  album?: string | null;
  imageUrl?: string | null;
  progressMs: number;
  durationMs: number;
  shuffle: boolean;
  repeatState: string;
}

/** Metadata completa de una instancia (incluye overrides JVM y auto_managed). */
export interface InstanceMeta {
  name: string;
  icon: string;
  mcVersion: string;
  loader: string;
  loaderVersion: string;
  source: string;
  ramMb: number;
  totalPlayMinutes: number;
  lastPlayed: string | null;
  versionId: string | null;
  autoManaged: boolean;
  jvmArgs: string | null;
  gc: string | null;
  javaPath: string | null;
}

export interface InstanceContentItem {
  path: string;
  name: string;
  folder: string;
  kind: string;
  sizeBytes: number;
  sha1: string | null;
  enabled: boolean;
}

export interface AiMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
}

// --- Fase 2: Java, backups y login Microsoft ---

export interface JavaInstallation {
  path: string;
  versionMajor: number;
  versionFull: string;
  vendor: string;
  source: string;
}

export interface BackupInfo {
  name: string;
  sizeBytes: number;
  createdAt: number; // epoch en segundos
  path: string;
}

export interface DeviceCodeStart {
  userCode: string;
  verificationUri: string;
  expiresIn: number;
  interval: number;
}
