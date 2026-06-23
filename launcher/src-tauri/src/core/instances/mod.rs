//! Gestor de instancias: metadata, escaneo/import, perfiles (CRUD) y backups.
//!
//! Las instancias del ecosistema viven en `<mc>/instancias/<folder>` y guardan
//! su metadata en `paraguacraft.json`. Las instancias de otros launchers se
//! detectan en modo lectura y pueden importarse (copia) a la biblioteca.

pub mod backups;
pub mod content;
pub mod icons;
pub mod importers;
pub mod profiles;
pub mod scan;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::paths;
use crate::models::Instance;

const META_FILE: &str = "paraguacraft.json";

/// Metadata persistida por instancia.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMeta {
    pub name: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    pub mc_version: String,
    #[serde(default = "default_loader")]
    pub loader: String,
    #[serde(default)]
    pub loader_version: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub ram_mb: u32,
    #[serde(default)]
    pub total_play_minutes: u64,
    #[serde(default)]
    pub last_played: Option<String>,
    /// Id del perfil instalado a lanzar (`versions/<id>`). Se fija tras instalar
    /// el loader; si es None, se lanza vanilla `mc_version`.
    #[serde(default)]
    pub version_id: Option<String>,
    // --- Override de preferencias del usuario (Regla 2) ---
    /// Si true, la autoconfig de hardware puede ajustar ram/jvm. Si el usuario
    /// edita manualmente, pasa a false y el launcher NO sobrescribe.
    #[serde(default = "default_true")]
    pub auto_managed: bool,
    #[serde(default)]
    pub jvm_args: Option<String>,
    #[serde(default)]
    pub gc: Option<String>,
    #[serde(default)]
    pub java_path: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_icon() -> String {
    "loader:vanilla".into()
}
fn default_loader() -> String {
    "vanilla".into()
}
fn default_source() -> String {
    "paraguacraft".into()
}

/// Icono persistido segun el loader (`loader:fabric`, `loader:forge`, etc.).
pub fn icon_for_loader(loader: &str) -> String {
    format!("loader:{}", crate::core::loaders::normalize(loader))
}

/// Asigna el icono del loader salvo iconos personalizados importados.
pub fn sync_loader_icon(meta: &mut InstanceMeta) -> bool {
    if meta.icon.trim().starts_with("custom:") {
        return false;
    }
    let want = icon_for_loader(&meta.loader);
    if meta.icon != want {
        meta.icon = want;
        return true;
    }
    false
}

impl InstanceMeta {
    pub fn into_instance(self, folder: &str, dir: &Path) -> Instance {
        Instance {
            id: folder.to_string(),
            name: self.name,
            icon: self.icon,
            mc_version: self.mc_version,
            loader: self.loader,
            loader_version: self.loader_version,
            source: self.source,
            last_played: self.last_played,
            total_play_minutes: self.total_play_minutes,
            ram_mb: self.ram_mb,
            mod_count: count_mods(dir),
        }
    }
}

/// Sanitiza el nombre de carpeta como el Python (`carpeta_instancia_paraguacraft`).
pub fn folder_for(mc_version: &str, loader: &str) -> String {
    let norm = loader.trim().to_lowercase().replace(' ', "-");
    if norm.contains("paraguacraft-pvp") || norm == "pvp" {
        return format!("Paraguacraft_{mc_version}_PvP");
    }
    let motor = loader.replace(' ', "_").replace('+', "Plus");
    format!("Paraguacraft_{mc_version}_{motor}")
}

pub fn instance_dir(folder: &str) -> PathBuf {
    paths::instances_dir().join(folder)
}

fn meta_path(folder: &str) -> PathBuf {
    instance_dir(folder).join(META_FILE)
}

pub fn read_meta(folder: &str) -> Option<InstanceMeta> {
    crate::config::read_json::<InstanceMeta>(&meta_path(folder))
}

/// Metadata resuelta: JSON en disco o inferida del nombre de carpeta (legacy Python).
pub fn resolve_meta(folder: &str) -> Option<InstanceMeta> {
    let mut meta = if let Some(m) = read_meta(folder) {
        m
    } else if instance_dir(folder).is_dir() {
        meta_from_folder_name(folder)
    } else {
        return None;
    };
    enrich_legacy_meta(folder, &mut meta);
    Some(meta)
}

/// Completa loader/version desde `_paragua_instance.json` o perfiles en `versions/`.
fn enrich_legacy_meta(folder: &str, meta: &mut InstanceMeta) {
    let dir = instance_dir(folder);
    let legacy_path = dir.join("_paragua_instance.json");
    if let Ok(raw) = std::fs::read_to_string(&legacy_path) {
        if let Ok(leg) = serde_json::from_str::<serde_json::Value>(&raw) {
            if meta.loader_version.is_empty() {
                if let Some(v) = leg
                    .get("fabric_loader_version")
                    .and_then(|x| x.as_str())
                    .filter(|s| !s.is_empty())
                {
                    meta.loader_version = v.to_string();
                }
            }
            if meta.version_id.is_none() {
                if let Some(v) = leg
                    .get("version_id")
                    .and_then(|x| x.as_str())
                    .filter(|s| !s.is_empty())
                {
                    meta.version_id = Some(v.to_string());
                }
            }
            if meta.last_played.is_none() {
                if let Some(v) = leg
                    .get("last_played")
                    .and_then(|x| x.as_str())
                    .filter(|s| !s.is_empty())
                {
                    meta.last_played = Some(v.to_string());
                }
            }
            if meta.total_play_minutes == 0 {
                if let Some(mins) = leg.get("total_playtime").and_then(|x| x.as_u64()) {
                    meta.total_play_minutes = mins / 60;
                }
            }
            if let Some(name) = leg.get("display_name").and_then(|x| x.as_str()).filter(|s| !s.is_empty())
            {
                if meta.name.starts_with("1.") || meta.name == folder {
                    meta.name = name.to_string();
                }
            }
        }
    }

    if meta.version_id.as_ref().is_none_or(|v| crate::core::versions::read_local_json(v).is_none()) {
        if let Some(vid) =
            crate::core::loaders::find_version_id_for_loader(&meta.mc_version, &meta.loader)
        {
            meta.version_id = Some(vid.clone());
            if meta.loader_version.is_empty() {
                if let Some(lv) = crate::core::loaders::loader_version_from_version_id(
                    &meta.loader,
                    &vid,
                    &meta.mc_version,
                ) {
                    meta.loader_version = lv;
                }
            }
        }
    } else if meta.loader_version.is_empty() {
        if let Some(vid) = meta.version_id.clone() {
            if let Some(lv) = crate::core::loaders::loader_version_from_version_id(
                &meta.loader,
                &vid,
                &meta.mc_version,
            ) {
                meta.loader_version = lv;
            }
        }
    }
}

/// Asegura `paraguacraft.json` en instancias legacy (carpeta existe, JSON ausente).
pub fn ensure_meta(folder: &str) -> crate::error::AppResult<InstanceMeta> {
    if folder.starts_with("ext::") {
        return Err(crate::error::AppError::msg(
            "Usa get_external_meta para instancias externas",
        ));
    }
    if !instance_dir(folder).is_dir() {
        return Err(crate::error::AppError::msg("Instancia no encontrada"));
    }
    let mut meta = resolve_meta(folder).ok_or_else(|| {
        crate::error::AppError::msg("Instancia no encontrada")
    })?;
    enrich_legacy_meta(folder, &mut meta);
    let disk = read_meta(folder);
    let needs_write = disk.is_none()
        || disk.as_ref().map(|d| d.loader_version.is_empty()).unwrap_or(false)
            && !meta.loader_version.is_empty()
        || disk.as_ref().map(|d| d.version_id.is_none()).unwrap_or(false)
            && meta.version_id.is_some();
    let icon_changed = sync_loader_icon(&mut meta);
    if needs_write || icon_changed {
        write_meta(folder, &meta)?;
    }
    Ok(meta)
}

/// Metadata de instancia detectada en otro launcher (`ext::source::key`).
pub fn resolve_external_meta(ext_id: &str) -> Option<InstanceMeta> {
    let inst = scan::scan_external().into_iter().find(|i| i.id == ext_id)?;
    Some(InstanceMeta {
        name: inst.name,
        icon: icon_for_loader(&inst.loader),
        mc_version: inst.mc_version,
        loader: inst.loader,
        loader_version: inst.loader_version,
        source: inst.source,
        ram_mb: if inst.ram_mb > 0 { inst.ram_mb } else { 4096 },
        total_play_minutes: inst.total_play_minutes,
        last_played: inst.last_played,
        version_id: Some(ext_id.rsplit("::").next()?.to_string()),
        auto_managed: true,
        jvm_args: None,
        gc: None,
        java_path: None,
    })
}

/// Carpeta de juego efectiva (local o externa).
pub fn game_dir_for(id: &str) -> Option<PathBuf> {
    if id.starts_with("ext::") {
        return importers::external_game_dir(id);
    }
    let dir = instance_dir(id);
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

pub fn write_meta(folder: &str, meta: &InstanceMeta) -> crate::error::AppResult<()> {
    crate::config::write_json_atomic(&meta_path(folder), meta)
}

/// Cuenta `.jar` habilitados en `mods/`.
pub fn count_mods(dir: &Path) -> u32 {
    let mods = dir.join("mods");
    let Ok(entries) = std::fs::read_dir(&mods) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("jar"))
                .unwrap_or(false)
        })
        .count() as u32
}

/// Lista las instancias del ecosistema Paraguacraft (sin escanear externos).
pub fn list_local() -> Vec<Instance> {
    let root = paths::instances_dir();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let dir = e.path();
        if !dir.is_dir() {
            continue;
        }
        let folder = e.file_name().to_string_lossy().to_string();
        let mut meta = read_meta(&folder).unwrap_or_else(|| meta_from_folder_name(&folder));
        enrich_legacy_meta(&folder, &mut meta);
        if sync_loader_icon(&mut meta) {
            let _ = write_meta(&folder, &meta);
        }
        out.push(meta.into_instance(&folder, &dir));
    }
    out
}

/// Infere metadata desde el nombre `Paraguacraft_<ver>_<motor>` (instancias
/// creadas por el launcher Python que aun no tienen `paraguacraft.json`).
pub fn meta_from_folder_name(folder: &str) -> InstanceMeta {
    let rest = folder.strip_prefix("Paraguacraft_").unwrap_or(folder);
    let (ver, motor) = rest.split_once('_').unwrap_or((rest, "vanilla"));
    InstanceMeta {
        name: format!("{ver} {}", motor.replace('_', " ")),
        icon: default_icon(),
        mc_version: ver.to_string(),
        loader: motor.replace("Plus", "+").replace('_', "-").to_lowercase(),
        loader_version: String::new(),
        source: "paraguacraft".into(),
        ram_mb: 4096,
        total_play_minutes: 0,
        last_played: None,
        version_id: None,
        auto_managed: true,
        jvm_args: None,
        gc: None,
        java_path: None,
    }
}
