//! Backup ZIP del mundo del servidor.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zip::write::SimpleFileOptions;

use crate::core::paths;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerBackupResult {
    pub path: String,
    pub size_mb: f64,
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn backups_dir(server_id: &str) -> PathBuf {
    let dir = paths::data_dir().join("server_backups").join(server_id);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn world_dir(server_dir: &Path, props_level: Option<&str>) -> PathBuf {
    if let Some(name) = props_level.filter(|s| !s.is_empty()) {
        let named = server_dir.join(name);
        if named.is_dir() {
            return named;
        }
    }
    let world = server_dir.join("world");
    if world.is_dir() {
        return world;
    }
    server_dir.join("world")
}

pub fn backup_worlds(server_id: &str, server_dir: &Path) -> AppResult<ServerBackupResult> {
    let level = crate::core::server_properties::read(server_dir)
        .ok()
        .and_then(|p| p.get("level-name").cloned());
    let world = world_dir(server_dir, level.as_deref());
    if !world.is_dir() {
        return Err(AppError::msg("No hay mundo para respaldar (carpeta world vacía o inexistente)."));
    }

    let ts = now_ts();
    let out = backups_dir(server_id).join(format!("world_{ts}.zip"));
    let tmp = out.with_extension("zip.tmp");

    {
        let file = std::fs::File::create(&tmp)?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        add_dir_recursive(&mut zip, &world, &world, opts)?;
        zip.finish()?;
    }
    std::fs::rename(&tmp, &out)?;
    let size = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
    Ok(ServerBackupResult {
        path: out.to_string_lossy().to_string(),
        size_mb: (size as f64) / (1024.0 * 1024.0),
    })
}

fn add_dir_recursive(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    dir: &Path,
    opts: SimpleFileOptions,
) -> AppResult<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            add_dir_recursive(zip, base, &path, opts)?;
        } else if path.is_file() {
            let rel = path.strip_prefix(base).unwrap_or(&path);
            let name = rel.to_string_lossy().replace('\\', "/");
            zip.start_file(name, opts)?;
            let mut f = std::fs::File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}

pub fn list_backups(server_id: &str) -> Vec<ServerBackupResult> {
    let root = backups_dir(server_id);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension()?.to_str()? != "zip" {
                return None;
            }
            let size = e.metadata().ok()?.len();
            Some(ServerBackupResult {
                path: p.to_string_lossy().to_string(),
                size_mb: (size as f64) / (1024.0 * 1024.0),
            })
        })
        .collect()
}
