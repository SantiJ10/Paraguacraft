//! Backups seguros de instancias (zip atomico de saves + config clave).
//!
//! Espeja `crear_backup_instancia` / `listar_backups` / `restaurar_backup`.
//! Se guardan en `<data>/backups/<folder>/backup_<epoch>.zip`. Escritura a
//! `.tmp` + rename para no dejar zips corruptos.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zip::write::SimpleFileOptions;

use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::BackupInfo;

use super::instance_dir;

/// Subcarpetas/archivos que se respaldan (datos del jugador).
const BACKED_UP: [&str; 4] = ["saves", "options.txt", "servers.dat", "config"];

fn backups_root(folder: &str) -> PathBuf {
    let dir = paths::data_dir().join("backups").join(folder);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn ensure_local(id: &str) -> AppResult<()> {
    if id.starts_with("ext::") || !instance_dir(id).is_dir() {
        return Err(AppError::msg("Instancia no valida para backup"));
    }
    Ok(())
}

pub fn create(id: &str) -> AppResult<BackupInfo> {
    ensure_local(id)?;
    let src = instance_dir(id);
    let ts = now();
    let name = format!("backup_{ts}.zip");
    let dest = backups_root(id).join(&name);
    let tmp = dest.with_extension("zip.tmp");

    {
        let file = std::fs::File::create(&tmp)?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for item in BACKED_UP {
            let path = src.join(item);
            if path.is_file() {
                add_file(&mut zip, &src, &path, opts)?;
            } else if path.is_dir() {
                add_dir(&mut zip, &src, &path, opts)?;
            }
        }
        zip.finish()?;
    }
    std::fs::rename(&tmp, &dest)?;

    let size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    Ok(BackupInfo {
        name,
        size_bytes: size,
        created_at: ts,
        path: dest.to_string_lossy().to_string(),
    })
}

pub fn list(id: &str) -> Vec<BackupInfo> {
    let root = backups_root(id);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut out: Vec<BackupInfo> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if !name.ends_with(".zip") {
                return None;
            }
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            let ts = name
                .trim_start_matches("backup_")
                .trim_end_matches(".zip")
                .parse::<u64>()
                .unwrap_or(0);
            Some(BackupInfo {
                name,
                size_bytes: size,
                created_at: ts,
                path: p.to_string_lossy().to_string(),
            })
        })
        .collect();
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    out
}

pub fn restore(id: &str, name: &str) -> AppResult<()> {
    ensure_local(id)?;
    let zip_path = backups_root(id).join(name);
    if !zip_path.is_file() {
        return Err(AppError::msg("Backup no encontrado"));
    }
    let dest = instance_dir(id);
    let file = std::fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let out_path = dest.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf)?;
            std::fs::write(&out_path, &buf)?;
        }
    }
    Ok(())
}

pub fn delete(id: &str, name: &str) -> AppResult<()> {
    let zip_path = backups_root(id).join(name);
    if zip_path.is_file() {
        std::fs::remove_file(&zip_path)?;
    }
    Ok(())
}

fn add_file(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    path: &Path,
    opts: SimpleFileOptions,
) -> AppResult<()> {
    let rel = path.strip_prefix(base).unwrap_or(path).to_string_lossy().replace('\\', "/");
    zip.start_file(rel, opts)?;
    let mut f = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    zip.write_all(&buf)?;
    Ok(())
}

fn add_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    dir: &Path,
    opts: SimpleFileOptions,
) -> AppResult<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            add_dir(zip, base, &p, opts)?;
        } else {
            add_file(zip, base, &p, opts)?;
        }
    }
    Ok(())
}
