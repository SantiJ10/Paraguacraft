// Capa de abstraccion IPC.
//
// La UI nunca llama `invoke` directamente: habla con `api.*`. Si corremos dentro
// de Tauri usamos los comandos Rust reales (Fase 2); en navegador suelto caemos
// a mocks para poder previsualizar la UI con `npm run dev`.

import type {
  Account,
  AppSettings,
  BackupInfo,
  ContentProvider,
  ContentType,
  CleanupInfo,
  SpotifyNowPlaying,
  SpotifySimpleResult,
  SpotifyOAuthPoll,
  SpotifyStatus,
  SpotifySetupInfo,
  CrashDiagnosis,
  ExtrasStatus,
  DeviceCodeStart,
  DownloadTask,
  AiAssistResponse,
  ApplySkinResult,
  ServerProfile,
  ServerStatus,
  ServerContentItem,
  HangarPlugin,
  ServerBackupResult,
  ServerRepairReport,
  SkinProfile,
  SkinLookup,
  SkinCatalogPage,
  SkinHistoryEntry,
  StructuredInvokeError,
  UpdateInfo,
  HardwareInfo,
  ImportInstanceIconResult,
  BedrockStatus,
  Instance,
  InstanceContentItem,
  InstanceMeta,
  JavaInstallation,
  LoaderInfo,
  MinecraftVersion,
  StoreItem,
  StoreVersion,
  StoreInstallDestination,
  WorldInfo,
  ServerWorldsResult,
} from "@/lib/types";
import {
  mockAccounts,
  mockHardware,
  mockInstances,
  mockLoaders,
  mockStoreItems,
  mockVersions,
} from "@/lib/mock/data";
import { minotarBody, minotarHelm, minotarSkin, STEVE_AVATAR_URL } from "@/lib/skins";

/** True si corremos dentro del runtime de Tauri (no en un navegador suelto). */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Invoca un comando Tauri. */
async function invokeReal<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

/** Parsea errores estructurados de Tauri (p. ej. cf_distribution_blocked). */
export function parseInvokeError(e: unknown): StructuredInvokeError & { raw: string } {
  const raw = String(e);
  try {
    const parsed = JSON.parse(raw) as StructuredInvokeError;
    if (parsed.code && parsed.message) {
      return { ...parsed, raw };
    }
  } catch {
    /* string plano */
  }
  return { code: "error", message: raw, raw };
}

/** Abre una URL en el navegador del sistema. */
export async function openUrl(url: string): Promise<void> {
  if (!isTauri()) {
    window.open(url, "_blank");
    return;
  }
  const { open } = await import("@tauri-apps/plugin-shell");
  await open(url);
}

function delay<T>(value: T, ms = 120): Promise<T> {
  return new Promise((resolve) => setTimeout(() => resolve(value), ms));
}

export const api = {
  // --- Hardware ---
  async getHardwareInfo(): Promise<HardwareInfo> {
    if (isTauri()) {
      try {
        return await invokeReal<HardwareInfo>("get_hardware_info");
      } catch (err) {
        console.warn("[ipc] get_hardware_info fallback a mock:", err);
      }
    }
    return delay(mockHardware);
  },

  // --- Settings ---
  async getSettings(): Promise<AppSettings> {
    if (isTauri()) {
      return invokeReal<AppSettings>("get_settings");
    }
    return delay({
      ramMb: mockHardware.recommendedRamMb,
      gcType: mockHardware.recommendedGc,
      javaPath: null,
      closeOnLaunch: false,
      optimizeGraphics: false,
      gpuCompatMode: "off",
      theme: "dark",
      accent: "green",
      language: "es",
      discordRpc: true,
      discordRpcVersion: true,
      discordRpcTime: true,
      papaMode: false,
      deepCleanOnLaunch: false,
      backupAutoHours: 0,
      javaPriority: "high",
    });
  },

  async saveSettings(settings: AppSettings): Promise<void> {
    if (isTauri()) {
      await invokeReal<void>("save_settings", { settings });
    }
  },

  async syncDiscordRpc(): Promise<void> {
    if (isTauri()) {
      await invokeReal<void>("sync_discord_rpc");
    }
  },

  async getExtrasStatus(): Promise<ExtrasStatus> {
    if (isTauri()) {
      return invokeReal<ExtrasStatus>("get_extras_status");
    }
    return delay({ gameModeActive: false, turboActive: false, javaPriority: "high" });
  },

  async activateGameMode(): Promise<string[]> {
    if (isTauri()) return invokeReal<string[]>("activate_game_mode");
    return delay(["Game Mode activo (mock)"]);
  },

  async deactivateGameMode(): Promise<string[]> {
    if (isTauri()) return invokeReal<string[]>("deactivate_game_mode");
    return delay(["Game Mode desactivado"]);
  },

  async activateTurboMode(): Promise<string[]> {
    if (isTauri()) return invokeReal<string[]>("activate_turbo_mode");
    return delay(["Modo Turbo activo (mock)"]);
  },

  async deactivateTurboMode(): Promise<string[]> {
    if (isTauri()) return invokeReal<string[]>("deactivate_turbo_mode");
    return delay(["Modo Turbo desactivado"]);
  },

  async setJavaPriority(level: string): Promise<number> {
    if (isTauri()) return invokeReal<number>("set_java_priority", { level });
    return delay(0);
  },

  async getCleanupInfo(): Promise<CleanupInfo> {
    if (isTauri()) return invokeReal<CleanupInfo>("get_cleanup_info");
    return delay({ logsMb: 0, crashMb: 0, mcRamMb: 0 });
  },

  async runCleanup(kind: "logs" | "crash" | "both"): Promise<number> {
    if (isTauri()) return invokeReal<number>("run_cleanup", { kind });
    return delay(0);
  },

  async shutdownBackgroundServices(): Promise<void> {
    if (isTauri()) {
      await invokeReal<void>("shutdown_background_services");
    }
  },

  async applyRecommendedPerformance(): Promise<HardwareInfo> {
    if (isTauri()) {
      return invokeReal<HardwareInfo>("apply_recommended_performance");
    }
    return delay(mockHardware);
  },

  async optimizeMinecraftOptions(): Promise<{ tier: string; applied: Record<string, string>; path: string }> {
    if (isTauri()) {
      return invokeReal("optimize_minecraft_options");
    }
    return delay({ tier: "media", applied: {}, path: "" });
  },

  // --- Cuentas ---
  async getAccounts(): Promise<Account[]> {
    if (isTauri()) {
      return invokeReal<Account[]>("get_accounts");
    }
    return delay(mockAccounts);
  },

  async setActiveAccount(id: string): Promise<Account[]> {
    if (isTauri()) {
      return invokeReal<Account[]>("set_active_account", { id });
    }
    return delay(mockAccounts.map((a) => ({ ...a, active: a.id === id })));
  },

  async addOfflineAccount(username: string): Promise<Account[]> {
    if (isTauri()) {
      return invokeReal<Account[]>("add_offline_account", { username });
    }
    return delay(mockAccounts);
  },

  async removeAccount(id: string): Promise<Account[]> {
    if (isTauri()) {
      return invokeReal<Account[]>("remove_account", { id });
    }
    return delay(mockAccounts.filter((a) => a.id !== id));
  },

  async msLoginUrl(): Promise<string> {
    return invokeReal<string>("ms_login_url");
  },

  async msLoginCompleteCode(code: string): Promise<Account[]> {
    return invokeReal<Account[]>("ms_login_complete_code", { code });
  },

  async msLoginStart(): Promise<DeviceCodeStart> {
    return invokeReal<DeviceCodeStart>("ms_login_start");
  },

  /** Devuelve la lista de cuentas cuando el usuario autoriza, o null si sigue pendiente. */
  async msLoginPoll(): Promise<Account[] | null> {
    return invokeReal<Account[] | null>("ms_login_poll");
  },

  async ensureAccountToken(id: string): Promise<boolean> {
    return invokeReal<boolean>("ensure_account_token", { id });
  },

  // --- Java ---
  async detectJavas(forceRefresh = false): Promise<JavaInstallation[]> {
    if (isTauri()) {
      return invokeReal<JavaInstallation[]>("detect_javas", { forceRefresh });
    }
    return delay([]);
  },

  async verifyJavaPath(path: string): Promise<JavaInstallation | null> {
    return invokeReal<JavaInstallation | null>("verify_java_path", { path });
  },

  async javaRequiredForMc(version: string): Promise<number> {
    if (isTauri()) {
      return invokeReal<number>("java_required_for_mc", { version });
    }
    return delay(17);
  },

  async downloadTemurin(major: number, force = false): Promise<string> {
    return invokeReal<string>("download_temurin", { major, force });
  },

  // --- Instancias ---
  async scanInstances(): Promise<Instance[]> {
    if (isTauri()) {
      return invokeReal<Instance[]>("scan_instances");
    }
    return delay(mockInstances);
  },

  async getInstances(): Promise<Instance[]> {
    if (isTauri()) {
      return invokeReal<Instance[]>("list_instances");
    }
    return delay(mockInstances);
  },

  async createInstance(payload: {
    name: string;
    mcVersion: string;
    loader: string;
    loaderVersion: string;
    icon: string;
    ramMb: number;
  }): Promise<Instance> {
    return invokeReal<Instance>("create_instance", { ...payload });
  },

  async renameInstance(id: string, name: string, icon: string): Promise<Instance> {
    return invokeReal<Instance>("rename_instance", { id, name, icon });
  },

  async getInstanceIconPath(icon: string): Promise<string | null> {
    if (!isTauri()) return null;
    return invokeReal<string | null>("get_instance_icon_path", { icon });
  },

  async pickAndImportInstanceIcon(): Promise<ImportInstanceIconResult> {
    return invokeReal<ImportInstanceIconResult>("pick_and_import_instance_icon");
  },

  async setInstanceRam(id: string, ramMb: number): Promise<Instance> {
    return invokeReal<Instance>("set_instance_ram", { id, ramMb });
  },

  async duplicateInstance(id: string, newName: string): Promise<Instance> {
    return invokeReal<Instance>("duplicate_instance", { id, newName });
  },

  async deleteInstance(id: string): Promise<void> {
    await invokeReal<void>("delete_instance", { id });
  },

  async importInstance(id: string): Promise<Instance> {
    return invokeReal<Instance>("import_instance", { id });
  },

  // --- Backups ---
  async createBackup(id: string): Promise<BackupInfo> {
    return invokeReal<BackupInfo>("create_backup", { id });
  },

  async listBackups(id: string): Promise<BackupInfo[]> {
    if (isTauri()) {
      return invokeReal<BackupInfo[]>("list_backups", { id });
    }
    return delay([]);
  },

  async restoreBackup(id: string, name: string): Promise<void> {
    await invokeReal<void>("restore_backup", { id, name });
  },

  async deleteBackup(id: string, name: string): Promise<void> {
    await invokeReal<void>("delete_backup", { id, name });
  },

  // --- Versiones (Fase 3) ---
  async getVersions(): Promise<MinecraftVersion[]> {
    if (isTauri()) {
      return invokeReal<MinecraftVersion[]>("list_minecraft_versions");
    }
    return delay(mockVersions);
  },

  async installVersion(id: string): Promise<void> {
    await invokeReal<void>("install_minecraft_version", { id });
  },

  // --- Loaders (Fase 3): compatibilidad estricta ---
  async getLoaders(mcVersion: string): Promise<LoaderInfo[]> {
    if (isTauri()) {
      return invokeReal<LoaderInfo[]>("list_loaders", { mc: mcVersion });
    }
    return delay(mockLoaders);
  },

  async getLoaderVersions(loader: string, mcVersion: string): Promise<string[]> {
    if (isTauri()) {
      return invokeReal<string[]>("list_loader_versions", { loader, mc: mcVersion });
    }
    return delay([]);
  },

  async installLoader(mc: string, loader: string, loaderVersion: string): Promise<string> {
    return invokeReal<string>("install_loader", { mc, loader, loaderVersion });
  },

  async installFabricIrisBundle(instanceId: string): Promise<void> {
    await invokeReal<void>("install_fabric_iris_bundle", { instanceId });
  },

  async installPvpBundle(instanceId: string): Promise<void> {
    await invokeReal<void>("install_pvp_bundle", { instanceId });
  },

  // --- Tienda: catálogo global (browse) + install por instancia ---
  async searchStore(payload: {
    provider: ContentProvider;
    query: string;
    projectType: ContentType;
    /** Vacío = exploración global sin filtro de versión */
    mc?: string;
    /** Vacío = exploración global sin filtro de loader */
    loader?: string;
  }): Promise<StoreItem[]> {
    const mc = payload.mc ?? "";
    const loader = payload.loader ?? "";
    if (isTauri()) {
      return invokeReal<StoreItem[]>("store_search", {
        provider: payload.provider,
        query: payload.query,
        projectType: payload.projectType,
        mc,
        loader,
      });
    }
    const q = payload.query.trim().toLowerCase();
    const items = q
      ? mockStoreItems.filter((i) => i.title.toLowerCase().includes(q))
      : mockStoreItems;
    return delay(items);
  },

  async installContent(payload: {
    provider: ContentProvider;
    projectId: string;
    projectType: ContentType;
    instanceId: string;
  }): Promise<string> {
    return invokeReal<string>("store_install", { ...payload });
  },

  async listStoreVersions(payload: {
    provider: ContentProvider;
    projectId: string;
    projectType: ContentType;
    mc: string;
    loader: string;
  }): Promise<StoreVersion[]> {
    if (isTauri()) {
      return invokeReal<StoreVersion[]>("store_list_versions", { ...payload });
    }
    return delay([
      {
        id: "mock-v1",
        name: "1.0.0",
        versionNumber: "1.0.0",
        filename: "mod.jar",
        gameVersions: [payload.mc],
        loaders: [payload.loader || "fabric"],
        publishedAt: "",
      },
    ]);
  },

  async listStoreProjectVersions(payload: {
    provider: ContentProvider;
    projectId: string;
    projectType: ContentType;
  }): Promise<StoreVersion[]> {
    return invokeReal<StoreVersion[]>("store_list_project_versions", { ...payload });
  },

  async installStoreVersion(payload: {
    provider: ContentProvider;
    projectId: string;
    projectType: ContentType;
    versionId: string;
    mc: string;
    loader: string;
    instanceId?: string;
    destination?: StoreInstallDestination;
    serverId?: string;
    worldName?: string;
  }): Promise<string> {
    return invokeReal<string>("store_install_version", { ...payload });
  },

  async importMrpack(source: string, mc: string): Promise<Instance> {
    return invokeReal<Instance>("import_mrpack", { source, mc });
  },

  async importMrpackVersion(versionId: string): Promise<Instance> {
    return invokeReal<Instance>("import_mrpack_version", { versionId });
  },

  async listInstanceWorlds(instanceId: string): Promise<WorldInfo[]> {
    return invokeReal<WorldInfo[]>("list_instance_worlds", { instanceId });
  },

  async listServerWorlds(serverId: string): Promise<ServerWorldsResult> {
    return invokeReal<ServerWorldsResult>("list_server_worlds", { serverId });
  },

  async updateInstanceContent(instanceId: string): Promise<number> {
    return invokeReal<number>("update_instance_content", { instanceId });
  },

  // --- Lanzamiento (Fase 3) ---
  async launchInstance(instanceId: string): Promise<number> {
    return invokeReal<number>("launch_instance", { instanceId });
  },

  async getBedrockStatus(): Promise<BedrockStatus> {
    if (isTauri()) {
      return invokeReal<BedrockStatus>("get_bedrock_status");
    }
    return delay({
      platformSupported: false,
      installed: false,
      premiumAllowed: false,
      username: null,
    });
  },

  async launchBedrock(): Promise<void> {
    await invokeReal<void>("launch_bedrock");
  },

  // --- Config por instancia (Regla 2: override del usuario) ---
  async getInstanceMeta(id: string): Promise<InstanceMeta> {
    return invokeReal<InstanceMeta>("get_instance_meta", { id });
  },

  async setInstanceConfig(payload: {
    id: string;
    ramMb?: number | null;
    jvmArgs?: string | null;
    gc?: string | null;
    javaPath?: string | null;
  }): Promise<InstanceMeta> {
    return invokeReal<InstanceMeta>("set_instance_config", { ...payload });
  },

  async setInstanceAutoManaged(id: string, auto: boolean): Promise<InstanceMeta> {
    return invokeReal<InstanceMeta>("set_instance_auto_managed", { id, auto });
  },

  async listInstanceContent(id: string): Promise<InstanceContentItem[]> {
    if (isTauri()) return invokeReal<InstanceContentItem[]>("list_instance_content", { id });
    return delay([]);
  },

  async toggleInstanceContent(id: string, path: string, enabled: boolean): Promise<void> {
    await invokeReal<void>("toggle_instance_content", { id, path, enabled });
  },

  async openInstanceFolder(id: string): Promise<void> {
    await invokeReal<void>("open_instance_folder", { id });
  },

  async getInstanceFolderPath(id: string): Promise<string> {
    return invokeReal<string>("get_instance_folder_path", { id });
  },

  async setInstanceLoader(id: string, loader: string, loaderVersion: string): Promise<InstanceMeta> {
    return invokeReal<InstanceMeta>("set_instance_loader", { id, loader, loaderVersion });
  },

  async reinstallInstanceLoader(id: string): Promise<Instance> {
    return invokeReal<Instance>("reinstall_instance_loader", { id });
  },

  async getDownloads(): Promise<DownloadTask[]> {
    return delay([]);
  },

  // --- Fase 4: diagnostico, servidores, skins, update ---
  async diagnoseInstance(instanceId: string, exitCode: number): Promise<CrashDiagnosis> {
    return invokeReal<CrashDiagnosis>("diagnose_instance", { instanceId, exitCode });
  },

  async aiAssist(prompt: string, diagnosis?: CrashDiagnosis | null): Promise<AiAssistResponse> {
    return invokeReal<AiAssistResponse>("ai_assist", { prompt, diagnosis: diagnosis ?? null });
  },

  async listServers(): Promise<ServerProfile[]> {
    if (isTauri()) return invokeReal<ServerProfile[]>("list_servers");
    return delay([]);
  },

  async createServer(payload: {
    name: string;
    mcVersion: string;
    serverType: string;
    ramMb: number;
  }): Promise<ServerProfile> {
    return invokeReal<ServerProfile>("create_server", { ...payload });
  },

  async deleteServer(id: string): Promise<void> {
    await invokeReal<void>("delete_server", { id });
  },

  async serverStatus(id: string): Promise<ServerStatus> {
    return invokeReal<ServerStatus>("server_status", { id });
  },

  async stopServer(id: string): Promise<void> {
    await invokeReal<void>("stop_server", { id });
  },

  async startServer(id: string): Promise<number> {
    return invokeReal<number>("start_server", { id });
  },

  async serverPluginCount(id: string): Promise<number> {
    if (isTauri()) return invokeReal<number>("server_plugin_count", { id });
    return delay(0);
  },

  async startPlayit(id: string): Promise<string> {
    return invokeReal<string>("start_playit", { id });
  },

  async prepareServerJar(id: string): Promise<string> {
    return invokeReal<string>("prepare_server_jar", { id });
  },

  async stopServerForce(id: string): Promise<void> {
    await invokeReal<void>("stop_server_force", { id });
  },

  async stopPlayit(id: string): Promise<void> {
    await invokeReal<void>("stop_playit", { id });
  },

  async getServerLog(id: string, maxLines?: number): Promise<string[]> {
    return invokeReal<string[]>("get_server_log", { id, maxLines: maxLines ?? 200 });
  },

  async exportServerLog(id: string): Promise<string> {
    return invokeReal<string>("export_server_log", { id });
  },

  async sendServerCommand(id: string, command: string): Promise<void> {
    await invokeReal<void>("send_server_command", { id, command });
  },

  async readServerProperties(id: string): Promise<Record<string, string>> {
    return invokeReal<Record<string, string>>("read_server_properties", { id });
  },

  async writeServerProperties(id: string, props: Record<string, string>): Promise<void> {
    await invokeReal<void>("write_server_properties", { id, props });
  },

  async openServerFolder(id: string): Promise<void> {
    await invokeReal<void>("open_server_folder", { id });
  },

  async getServerFolderPath(id: string): Promise<string> {
    return invokeReal<string>("get_server_folder_path", { id });
  },

  async listServerContent(id: string): Promise<ServerContentItem[]> {
    return invokeReal<ServerContentItem[]>("list_server_content", { id });
  },

  async serverWhitelistList(id: string): Promise<string[]> {
    return invokeReal<string[]>("server_whitelist_list", { id });
  },

  async serverWhitelistAdd(id: string, name: string): Promise<void> {
    await invokeReal<void>("server_whitelist_add", { id, name });
  },

  async serverWhitelistRemove(id: string, name: string): Promise<void> {
    await invokeReal<void>("server_whitelist_remove", { id, name });
  },

  async serverOpList(id: string): Promise<string[]> {
    return invokeReal<string[]>("server_op_list", { id });
  },

  async serverOpAdd(id: string, name: string): Promise<void> {
    await invokeReal<void>("server_op_add", { id, name });
  },

  async serverOpRemove(id: string, name: string): Promise<void> {
    await invokeReal<void>("server_op_remove", { id, name });
  },

  async serverBanList(id: string): Promise<string[]> {
    return invokeReal<string[]>("server_ban_list", { id });
  },

  async serverBanAdd(id: string, name: string): Promise<void> {
    await invokeReal<void>("server_ban_add", { id, name });
  },

  async serverBanRemove(id: string, name: string): Promise<void> {
    await invokeReal<void>("server_ban_remove", { id, name });
  },

  async hangarSearchPlugins(query: string): Promise<HangarPlugin[]> {
    return invokeReal<HangarPlugin[]>("hangar_search_plugins", { query });
  },

  async hangarInstallPlugin(id: string, owner: string, slug: string): Promise<string> {
    return invokeReal<string>("hangar_install_plugin", { id, owner, slug });
  },

  async serverBackupWorlds(id: string): Promise<ServerBackupResult> {
    return invokeReal<ServerBackupResult>("server_backup_worlds", { id });
  },

  async repairServer(id: string): Promise<ServerRepairReport> {
    return invokeReal<ServerRepairReport>("repair_server", { id });
  },

  async listServerBackups(id: string): Promise<ServerBackupResult[]> {
    return invokeReal<ServerBackupResult[]>("list_server_backups", { id });
  },

  async importServerFolder(path: string, name?: string): Promise<ServerProfile> {
    return invokeReal<ServerProfile>("import_server_folder", { path, name: name ?? null });
  },

  async pickServerFolder(): Promise<string | null> {
    return invokeReal<string | null>("pick_server_folder");
  },

  async setPlayitAddress(id: string, address: string): Promise<void> {
    await invokeReal<void>("set_playit_address", { id, address });
  },

  async getActiveSkin(): Promise<SkinProfile> {
    if (isTauri()) return invokeReal<SkinProfile>("get_active_skin");
    const acc = mockAccounts.find((a) => a.active);
    if (!acc) {
      return delay({
        username: "Steve",
        uuid: "8667ba71-b85a-4004-af54-4576b382cd64",
        premium: false,
        avatarUrl: STEVE_AVATAR_URL,
        bodyUrl: STEVE_AVATAR_URL,
      });
    }
    return delay({
      username: acc.username,
      uuid: acc.uuid,
      premium: acc.premium,
      avatarUrl: acc.avatarUrl || minotarHelm(acc.uuid, 64),
      bodyUrl: minotarBody(acc.uuid, 160),
    });
  },

  async getSkinForAccount(id: string): Promise<SkinProfile> {
    if (isTauri()) return invokeReal<SkinProfile>("get_skin_for_account", { id });
    const acc = mockAccounts.find((a) => a.id === id);
    if (!acc) throw new Error("Cuenta no encontrada");
    return delay({
      username: acc.username,
      uuid: acc.uuid,
      premium: acc.premium,
      avatarUrl: acc.avatarUrl || minotarHelm(acc.uuid, 64),
      bodyUrl: minotarBody(acc.uuid, 160),
    });
  },

  async getOfflineSkin(username: string): Promise<SkinProfile> {
    if (isTauri()) return invokeReal<SkinProfile>("get_offline_skin", { username });
    return delay({
      username,
      uuid: "offline",
      premium: false,
      avatarUrl: STEVE_AVATAR_URL,
      bodyUrl: STEVE_AVATAR_URL,
    });
  },

  async hasOfflineSkinFile(): Promise<boolean> {
    if (isTauri()) return invokeReal<boolean>("has_offline_skin_file");
    return delay(false);
  },

  async pickAndApplyOfflineSkin(): Promise<ApplySkinResult> {
    return invokeReal<ApplySkinResult>("pick_and_apply_offline_skin");
  },

  async lookupSkinPlayer(username: string): Promise<SkinLookup> {
    return invokeReal<SkinLookup>("lookup_skin_player", { username });
  },

  async skinCatalogSearch(query: string, page: number, random = false): Promise<SkinCatalogPage> {
    if (isTauri()) {
      return invokeReal<SkinCatalogPage>("skin_catalog_search", { query, page, random });
    }
    return delay({
      entries: [
        { id: "Steve", label: "Steve", skinUrl: minotarSkin("Steve"), previewUrl: minotarBody("Steve", 160), kind: "player" },
        { id: "Alex", label: "Alex", skinUrl: minotarSkin("Alex"), previewUrl: minotarBody("Alex", 160), kind: "player" },
      ],
      page: 1,
      totalPages: 1,
      totalSkins: 2,
      query: "",
    });
  },

  async skinPreviewImage(id: string, kind: "helm" | "body", size = 160): Promise<string | null> {
    if (isTauri()) return invokeReal<string | null>("skin_preview_image", { id, kind, size });
    return null;
  },

  async applySkinFromUsername(username: string, variant = "classic"): Promise<ApplySkinResult> {
    return invokeReal<ApplySkinResult>("apply_skin_from_username", { username, variant });
  },

  async applySkinFromUrl(url: string, variant: string, name: string): Promise<ApplySkinResult> {
    return invokeReal<ApplySkinResult>("apply_skin_from_url", { url, variant, name });
  },

  async downloadSkinFile(url: string, name: string): Promise<string> {
    return invokeReal<string>("download_skin_file", { url, name });
  },

  async getSkinHistory(): Promise<SkinHistoryEntry[]> {
    if (isTauri()) return invokeReal<SkinHistoryEntry[]>("get_skin_history");
    return delay([]);
  },

  async clearSkinHistory(): Promise<void> {
    if (isTauri()) await invokeReal<void>("clear_skin_history");
  },

  async canUploadPremiumSkin(): Promise<boolean> {
    if (isTauri()) return invokeReal<boolean>("can_upload_premium_skin");
    return delay(false);
  },

  async pickSkinFileForPreview(): Promise<string | null> {
    return invokeReal<string | null>("pick_skin_file_for_preview");
  },

  async applySkinFileWithVariant(path: string, variant: string): Promise<ApplySkinResult> {
    return invokeReal<ApplySkinResult>("apply_skin_file_with_variant", { path, variant });
  },

  async checkLauncherUpdate(): Promise<UpdateInfo> {
    if (isTauri()) return invokeReal<UpdateInfo>("check_launcher_update");
    return delay({
      currentVersion: "0.1.0",
      latestVersion: "0.1.0",
      updateAvailable: false,
      downloadUrl: null,
      releaseNotes: "",
      publishedAt: "",
      inAppInstall: false,
      assetName: null,
    });
  },

  async downloadAndInstallLauncherUpdate(): Promise<void> {
    await invokeReal<void>("download_and_install_launcher_update");
  },

  async spotifyStatus(): Promise<SpotifyStatus> {
    if (isTauri()) return invokeReal<SpotifyStatus>("spotify_status");
    return delay({
      connected: false,
      clientId: null,
      redirectUri: "http://127.0.0.1:8888/callback",
    });
  },

  async spotifySaveCredentials(
    clientId: string,
    clientSecret: string,
    redirectUri?: string,
  ): Promise<void> {
    await invokeReal<void>("spotify_save_credentials", {
      clientId,
      clientSecret,
      redirectUri: redirectUri ?? null,
    });
  },

  async spotifyValidateApp(): Promise<SpotifySimpleResult> {
    return invokeReal<SpotifySimpleResult>("spotify_validate_app");
  },

  async spotifySetupInfo(): Promise<SpotifySetupInfo> {
    if (isTauri()) return invokeReal<SpotifySetupInfo>("spotify_setup_info");
    return delay({
      redirectUri: "http://127.0.0.1:8888/callback",
      dashboardUrl: "https://developer.spotify.com/dashboard",
      authUrlPreview: null,
      clientIdOk: false,
      clientIdLength: 0,
    });
  },

  async spotifyAuthStart(): Promise<string> {
    return invokeReal<string>("spotify_auth_start");
  },

  async spotifyPollAuth(): Promise<SpotifyOAuthPoll> {
    if (isTauri()) return invokeReal<SpotifyOAuthPoll>("spotify_poll_auth");
    return delay({ ready: false });
  },

  async spotifyOauthHint(error: string): Promise<string> {
    if (isTauri()) return invokeReal<string>("spotify_oauth_hint", { error });
    return delay("Error de autorizacion Spotify");
  },

  async spotifyOpenDashboard(clientId?: string): Promise<void> {
    if (isTauri()) await invokeReal<void>("spotify_open_dashboard", { clientId: clientId ?? "" });
  },

  async spotifyConnect(code: string): Promise<SpotifySimpleResult> {
    return invokeReal<SpotifySimpleResult>("spotify_connect", { code });
  },

  async spotifyTryAutoconnect(): Promise<SpotifySimpleResult> {
    if (isTauri()) return invokeReal<SpotifySimpleResult>("spotify_try_autoconnect");
    return delay({ ok: false, error: "Solo en escritorio" });
  },

  async spotifyDisconnect(): Promise<void> {
    if (isTauri()) await invokeReal<void>("spotify_disconnect");
  },

  async spotifyNowPlaying(): Promise<SpotifyNowPlaying> {
    if (isTauri()) return invokeReal<SpotifyNowPlaying>("spotify_now_playing");
    return delay({ ok: false, playing: false, progressMs: 0, durationMs: 0, shuffle: false, repeatState: "off" });
  },

  async spotifyControl(action: "play" | "pause" | "next" | "prev"): Promise<SpotifySimpleResult> {
    return invokeReal<SpotifySimpleResult>("spotify_control", { action });
  },

  async spotifyShuffle(enabled: boolean): Promise<SpotifySimpleResult> {
    return invokeReal<SpotifySimpleResult>("spotify_shuffle", { enabled });
  },

  async spotifyRepeat(mode: "off" | "track" | "context"): Promise<SpotifySimpleResult> {
    return invokeReal<SpotifySimpleResult>("spotify_repeat", { mode });
  },
};
