//! Rutas de instalación según destino (instancia vs servidor local + mundo).

use std::path::{Path, PathBuf};

use crate::core::instances;
use crate::core::servers;
use crate::error::{AppError, AppResult};
use crate::models::WorldInfo;

fn read_level_name(server_dir: &Path) -> String {
    let props = server_dir.join("server.properties");
    if let Ok(text) = std::fs::read_to_string(&props) {
        for line in text.lines() {
            if let Some(v) = line.strip_prefix("level-name=") {
                let name = v.trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    "world".into()
}

fn scan_world_dirs(base: &Path, default_name: Option<&str>) -> Vec<WorldInfo> {
    let default = default_name.unwrap_or("world");
    let mut out = Vec::new();
    if !base.is_dir() {
        return out;
    }
    if let Ok(rd) = std::fs::read_dir(base) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && p.join("level.dat").is_file() {
                let name = e.file_name().to_string_lossy().to_string();
                out.push(WorldInfo {
                    name: name.clone(),
                    active: name == default,
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    if out.is_empty() && base.join("world").join("level.dat").is_file() {
        out.push(WorldInfo {
            name: "world".into(),
            active: default == "world",
        });
    }
    out
}

/// Mundos en `saves/` de una instancia (carpetas con `level.dat`).
pub fn list_instance_worlds(instance_id: &str) -> AppResult<Vec<WorldInfo>> {
    let base = if instance_id.starts_with("ext::") {
        instances::importers::external_game_dir(instance_id).ok_or_else(|| {
            AppError::msg("No se pudo resolver la carpeta de juego de la instancia externa.")
        })?
    } else {
        instances::instance_dir(instance_id)
    };
    Ok(scan_world_dirs(&base.join("saves"), None))
}

/// Mundos en la carpeta de un servidor local.
pub fn list_server_worlds(server_id: &str) -> AppResult<(Vec<WorldInfo>, String)> {
    let prof = servers::profile_by_id(server_id)?;
    let dir = servers::folder_for(&prof);
    let default = read_level_name(&dir);
    Ok((scan_world_dirs(&dir, Some(&default)), default))
}

pub fn plugin_dest_dir(server_id: &str) -> AppResult<PathBuf> {
    let prof = servers::profile_by_id(server_id)?;
    let dir = servers::folder_for(&prof);
    let sub = if prof.server_type.starts_with("fabric") {
        "mods"
    } else {
        "plugins"
    };
    let dest = dir.join(sub);
    std::fs::create_dir_all(&dest)?;
    Ok(dest)
}

/// Carpeta `mods/` de un servidor Fabric/Forge local.
pub fn mod_dest_dir(server_id: &str) -> AppResult<PathBuf> {
    let prof = servers::profile_by_id(server_id)?;
    let st = prof.server_type.as_str();
    if !st.starts_with("fabric") && !st.starts_with("forge") {
        return Err(AppError::msg(format!(
            "El servidor \"{}\" no tiene carpeta mods/ (solo Fabric/Forge).",
            prof.name
        )));
    }
    let dest = servers::folder_for(&prof).join("mods");
    std::fs::create_dir_all(&dest)?;
    Ok(dest)
}

fn resolve_server_world_dir(server_dir: &Path, world: Option<&str>) -> AppResult<PathBuf> {
    let level = world
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| read_level_name(server_dir));
    let mut world_dir = server_dir.join(&level);
    if !world_dir.is_dir() {
        world_dir = server_dir.join("world");
    }
    if !world_dir.is_dir() {
        return Err(AppError::msg(format!(
            "No se encontró el mundo '{level}' en el servidor"
        )));
    }
    Ok(world_dir)
}

pub fn datapack_dest_server(server_id: &str, world: Option<&str>) -> AppResult<PathBuf> {
    let dir = servers::folder_for_id(server_id)?;
    let world_dir = resolve_server_world_dir(&dir, world)?;
    let dest = world_dir.join("datapacks");
    std::fs::create_dir_all(&dest)?;
    Ok(dest)
}

pub fn datapack_dest_instance(instance_id: &str, world: Option<&str>) -> AppResult<PathBuf> {
    let base = if instance_id.starts_with("ext::") {
        instances::importers::external_game_dir(instance_id).ok_or_else(|| {
            AppError::msg("No se pudo resolver la carpeta de juego de la instancia externa.")
        })?
    } else {
        instances::instance_dir(instance_id)
    };
    let saves = base.join("saves");
    let world_name = world.map(str::trim).filter(|s| !s.is_empty());
    if let Some(name) = world_name {
        let dest = saves.join(name).join("datapacks");
        std::fs::create_dir_all(&dest)?;
        return Ok(dest);
    }
    let mut candidates: Vec<(String, u64)> = Vec::new();
    if saves.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&saves) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() && p.join("level.dat").is_file() {
                    let mtime = e
                        .metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    candidates.push((e.file_name().to_string_lossy().to_string(), mtime));
                }
            }
        }
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    let folder = candidates
        .first()
        .map(|(n, _)| n.clone())
        .unwrap_or_else(|| "world".into());
    let dest = saves.join(folder).join("datapacks");
    std::fs::create_dir_all(&dest)?;
    Ok(dest)
}
