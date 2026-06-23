//! Exportación de instancias a .zip (compartir o respaldar fuera del launcher).

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;

use crate::error::{AppError, AppResult};

use super::instance_dir;

const EXPORT_ITEMS: [&str; 8] = [
    "paraguacraft.json",
    "mods",
    "resourcepacks",
    "shaderpacks",
    "config",
    "options.txt",
    "servers.dat",
    "saves",
];

pub fn export_to(instance_id: &str, dest_zip: &Path) -> AppResult<String> {
    if instance_id.starts_with("ext::") {
        return Err(AppError::msg("No se puede exportar una instancia externa sin importarla."));
    }
    let src = instance_dir(instance_id);
    if !src.is_dir() {
        return Err(AppError::msg("Instancia no encontrada"));
    }
    if let Some(parent) = dest_zip.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = dest_zip.with_extension("zip.tmp");
    {
        let file = fs::File::create(&tmp)?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for item in EXPORT_ITEMS {
            let path = src.join(item);
            if path.is_file() {
                add_file(&mut zip, &src, &path, opts)?;
            } else if path.is_dir() {
                add_dir(&mut zip, &src, &path, opts)?;
            }
        }
        zip.finish()?;
    }
    if dest_zip.is_file() {
        fs::remove_file(dest_zip)?;
    }
    fs::rename(&tmp, dest_zip)?;
    Ok(dest_zip.to_string_lossy().to_string())
}

fn add_file(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    path: &Path,
    opts: SimpleFileOptions,
) -> AppResult<()> {
    let rel = path
        .strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    zip.start_file(rel, opts)?;
    let mut f = fs::File::open(path)?;
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
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            add_file(zip, base, &path, opts)?;
        } else if path.is_dir() {
            add_dir(zip, base, &path, opts)?;
        }
    }
    Ok(())
}
