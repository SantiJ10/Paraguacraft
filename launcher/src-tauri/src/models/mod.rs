//! Modelos compartidos con la UI. `rename_all = "camelCase"` para que el JSON
//! coincida con los tipos TypeScript de `src/lib/types.ts`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub ram_gb: f64,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub cpu_name: String,
    pub gpu_name: String,
    pub os: String,
    pub arch: String,
    /// "baja" | "media" | "alta"
    pub perfil_sugerido: String,
    pub recommended_ram_mb: u32,
    /// "G1GC" | "ZGC" | ...
    pub recommended_gc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub ram_mb: u32,
    pub gc_type: String,
    pub java_path: Option<String>,
    pub close_on_launch: bool,
    pub optimize_graphics: bool,
    pub gpu_compat_mode: String,
    pub theme: String,
    pub accent: String,
    pub language: String,
    /// API key opcional de CurseForge (la tienda CF la requiere).
    #[serde(default)]
    pub curseforge_api_key: Option<String>,
    /// Concurrencia de descargas (0 = automatico segun hardware).
    #[serde(default)]
    pub download_concurrency: u32,
    #[serde(default = "default_auto_update_check")]
    pub auto_update_check: bool,
    /// Si false, al cargar ajustes se aplican RAM/GC según hardware detectado.
    #[serde(default)]
    pub hardware_defaults_applied: bool,
    #[serde(default = "default_true")]
    pub discord_rpc: bool,
    #[serde(default = "default_true")]
    pub discord_rpc_version: bool,
    #[serde(default = "default_true")]
    pub discord_rpc_time: bool,
    #[serde(default)]
    pub papa_mode: bool,
    #[serde(default)]
    pub deep_clean_on_launch: bool,
    #[serde(default)]
    pub backup_auto_hours: u32,
    #[serde(default)]
    pub java_priority: String,
}

fn default_auto_update_check() -> bool {
    true
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            ram_mb: 4096,
            gc_type: "Auto".into(),
            java_path: None,
            close_on_launch: false,
            optimize_graphics: false,
            gpu_compat_mode: "off".into(),
            theme: "dark".into(),
            accent: "green".into(),
            language: "es".into(),
            curseforge_api_key: None,
            download_concurrency: 0,
            auto_update_check: true,
            hardware_defaults_applied: false,
            discord_rpc: true,
            discord_rpc_version: true,
            discord_rpc_time: true,
            papa_mode: false,
            deep_clean_on_launch: false,
            backup_auto_hours: 0,
            java_priority: "high".into(),
        }
    }
}

/// Cuenta tal como la ve la UI (SIN tokens; los tokens viven en `secrets.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    /// "microsoft" | "offline"
    #[serde(rename = "type")]
    pub kind: String,
    pub username: String,
    pub uuid: String,
    pub avatar_url: String,
    pub active: bool,
    pub premium: bool,
}

/// Tokens sensibles persistidos por cuenta (archivo separado y restringido).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRecord {
    /// refresh_token OAuth de Microsoft (rota en cada refresh).
    pub ms_refresh_token: String,
    /// access_token de Minecraft (Bearer para api.minecraftservices.com).
    pub mc_access_token: String,
    /// client_id usado (auth-code vs device-code) para refrescar correctamente.
    pub ms_client_id: String,
    /// epoch (segundos) del ultimo refresh exitoso.
    pub last_refresh: u64,
}

/// Resultado de iniciar el device-code flow (QR / microsoft.com/link).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeStart {
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftVersion {
    pub id: String,
    /// "release" | "snapshot" | "old_beta" | "old_alpha"
    pub channel: String,
    pub release_date: String,
    pub installed: bool,
}

/// Info de un loader disponible para una version de MC (compatibilidad estricta).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderInfo {
    /// "vanilla" | "fabric" | "forge" | "neoforge" | "quilt" | "optifine"
    pub id: String,
    pub name: String,
    pub description: String,
    /// Versiones exactas del loader disponibles para esa version de MC.
    pub versions: Vec<String>,
    /// true si la ultima version es estable/recomendada.
    pub recommended: Option<String>,
}

/// Item de la tienda (Modrinth/CurseForge) normalizado para la UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreItem {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub icon_url: String,
    pub downloads: u64,
    pub follows: u64,
    /// "mod" | "modpack" | "resourcepack" | "shader" | "datapack" | "plugin"
    pub project_type: String,
    /// "modrinth" | "curseforge"
    pub provider: String,
    pub categories: Vec<String>,
    /// URL publica del proyecto (CurseForge: pagina del mod para descarga manual).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_url: Option<String>,
}

/// Version/archivo de un proyecto de tienda compatible con mc+loader.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub filename: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub published_at: String,
}

/// Mundo detectado en saves/ o carpeta de servidor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInfo {
    pub name: String,
    pub active: bool,
}

/// Respuesta al listar mundos de un servidor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerWorldsResult {
    pub worlds: Vec<WorldInfo>,
    pub default_world: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInstallation {
    pub path: String,
    pub version_major: u32,
    pub version_full: String,
    pub vendor: String,
    /// "system" | "mojang" | "java_home" | "path" | "paraguacraft"
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub mc_version: String,
    pub loader: String,
    pub loader_version: String,
    /// "paraguacraft" | "vanilla" | "lunar" | "prism" | "tlauncher" | "sklauncher" | ...
    pub source: String,
    pub last_played: Option<String>,
    pub total_play_minutes: u64,
    pub ram_mb: u32,
    pub mod_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub name: String,
    pub size_bytes: u64,
    /// epoch en segundos (UTC) de creacion.
    pub created_at: u64,
    pub path: String,
}

/// Progreso de descarga emitido por eventos Tauri ("download://progress").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub id: String,
    pub label: String,
    pub progress: f64,
    pub status: String,
    pub speed: String,
}
