//! Contenido instalado en una instancia (mods, resource packs, shaders).

use std::path::{Path, PathBuf};

use crate::core::net;
use crate::error::{AppError, AppResult};

use super::game_dir_for;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceContentItem {
    /// Ruta relativa desde la carpeta de juego (ej. `mods/foo.jar`).
    pub path: String,
    pub name: String,
    pub folder: String,
    pub kind: String,
    pub size_bytes: u64,
    pub sha1: Option<String>,
    pub enabled: bool,
}

fn sha1_file(path: &Path) -> Option<String> {
    let meta = path.metadata().ok()?;
    if meta.len() > 8_000_000 {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    Some(net::sha1_hex(&bytes))
}

fn push_jar(entries: &mut Vec<InstanceContentItem>, base: &Path, folder: &str, kind: &str, path: &Path) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    if name.is_empty() {
        return;
    }
    let enabled = !name.ends_with(".disabled");
    let display = name.trim_end_matches(".disabled").to_string();
    let rel = path
        .strip_prefix(base)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| format!("{folder}/{name}"));
    let size = path.metadata().map(|m| m.len()).unwrap_or(0);
    entries.push(InstanceContentItem {
        path: rel,
        name: display,
        folder: folder.to_string(),
        kind: kind.to_string(),
        size_bytes: size,
        sha1: sha1_file(path),
        enabled,
    });
}

fn scan_folder_dir(entries: &mut Vec<InstanceContentItem>, base: &Path, folder: &str, kind: &str) {
    let dir = base.join(folder);
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name.ends_with(".disabled") {
                continue;
            }
            let rel = path
                .strip_prefix(base)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| format!("{folder}/{name}"));
            entries.push(InstanceContentItem {
                path: rel.clone(),
                name: name.to_string(),
                folder: folder.to_string(),
                kind: kind.to_string(),
                size_bytes: dir_size(&path),
                sha1: None,
                enabled: true,
            });
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) == Some("jar")
            || path.extension().and_then(|e| e.to_str()) == Some("zip")
        {
            push_jar(entries, base, folder, kind, &path);
        }
    }
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(path) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() {
                total += p.metadata().map(|m| m.len()).unwrap_or(0);
            } else if p.is_dir() {
                total += dir_size(&p);
            }
        }
    }
    total
}

pub fn list(id: &str) -> AppResult<Vec<InstanceContentItem>> {
    let base = game_dir_for(id).ok_or_else(|| AppError::msg("Instancia no encontrada"))?;
    let mut entries = Vec::new();
    scan_folder_dir(&mut entries, &base, "mods", "mod");
    scan_folder_dir(&mut entries, &base, "resourcepacks", "resourcepack");
    scan_folder_dir(&mut entries, &base, "shaderpacks", "shader");
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

pub fn toggle(id: &str, rel_path: &str, enabled: bool) -> AppResult<()> {
    let base = game_dir_for(id).ok_or_else(|| AppError::msg("Instancia no encontrada"))?;
    let rel = rel_path.replace('\\', "/");
    if rel.contains("..") {
        return Err(AppError::msg("Ruta invalida"));
    }
    let path = base.join(&rel);
    if !path.exists() {
        return Err(AppError::msg("Archivo no encontrado"));
    }
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::msg("Nombre invalido"))?;
    let new_name = if enabled {
        file_name.trim_end_matches(".disabled").to_string()
    } else if file_name.ends_with(".disabled") {
        file_name.to_string()
    } else {
        format!("{file_name}.disabled")
    };
    if new_name == file_name {
        return Ok(());
    }
    let dest = path.with_file_name(&new_name);
    std::fs::rename(&path, &dest)?;
    Ok(())
}

pub fn folder_path(id: &str) -> AppResult<PathBuf> {
    game_dir_for(id).ok_or_else(|| AppError::msg("Instancia no encontrada"))
}

pub fn open_folder(id: &str) -> AppResult<()> {
    let dir = folder_path(id)?;
    open_in_file_manager(&dir)
}

pub fn remove(id: &str, rel_path: &str) -> AppResult<()> {
    let base = game_dir_for(id).ok_or_else(|| AppError::msg("Instancia no encontrada"))?;
    let rel = rel_path.replace('\\', "/");
    if rel.contains("..") {
        return Err(AppError::msg("Ruta invalida"));
    }
    let path = base.join(&rel);
    if !path.exists() {
        return Err(AppError::msg("Archivo no encontrado"));
    }
    if path.is_dir() {
        std::fs::remove_dir_all(&path)?;
    } else {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn reveal(id: &str, rel_path: &str) -> AppResult<()> {
    let base = game_dir_for(id).ok_or_else(|| AppError::msg("Instancia no encontrada"))?;
    let rel = rel_path.replace('\\', "/");
    if rel.contains("..") {
        return Err(AppError::msg("Ruta invalida"));
    }
    let path = base.join(&rel);
    if !path.exists() {
        return Err(AppError::msg("Archivo no encontrado"));
    }
    reveal_in_file_manager(&path)
}

pub fn add_files(id: &str, folder: &str, sources: &[PathBuf]) -> AppResult<u32> {
    let base = game_dir_for(id).ok_or_else(|| AppError::msg("Instancia no encontrada"))?;
    let dest_dir = match folder {
        "mods" | "resourcepacks" | "shaderpacks" => base.join(folder),
        other => return Err(AppError::msg(format!("Carpeta no soportada: {other}"))),
    };
    std::fs::create_dir_all(&dest_dir)?;
    let mut copied = 0u32;
    for src in sources {
        if !src.is_file() {
            continue;
        }
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "jar" && ext != "zip" {
            continue;
        }
        let name = src
            .file_name()
            .ok_or_else(|| AppError::msg("Nombre invalido"))?;
        let dest = dest_dir.join(name);
        std::fs::copy(src, &dest)?;
        copied += 1;
    }
    if copied == 0 {
        return Err(AppError::msg("No se copió ningún archivo .jar/.zip válido"));
    }
    Ok(copied)
}

fn reveal_in_file_manager(path: &Path) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir el explorador: {e}")))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo revelar el archivo: {e}")))?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
        }
    }
    Ok(())
}

fn open_in_file_manager(path: &Path) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    Ok(())
}
