//! Despliegue de packs pre-generados (compile time) hacia la instancia.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use super::version::PackProfile;
use crate::error::AppResult;

const PACK_NAME: &str = "ParaguacraftBrandPack";

const EMBED_CLASSIC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/packs/classic.zip"));
const EMBED_LEGACY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/packs/legacy.zip"));
const EMBED_STANDARD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/packs/standard.zip"));
const EMBED_STANDARD_RANGE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/packs/standard_range.zip"));
const EMBED_WIDE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/packs/wide.zip"));
const EMBED_MODERN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/packs/modern.zip"));

fn embedded_zip(profile: PackProfile) -> &'static [u8] {
    match profile {
        PackProfile::Classic => EMBED_CLASSIC,
        PackProfile::Legacy => EMBED_LEGACY,
        PackProfile::Standard => EMBED_STANDARD,
        PackProfile::StandardRange => EMBED_STANDARD_RANGE,
        PackProfile::Wide => EMBED_WIDE,
        PackProfile::Modern => EMBED_MODERN,
    }
}

fn extract_zip(bytes: &[u8], dest_dir: &Path) -> AppResult<()> {
    if dest_dir.exists() {
        fs::remove_dir_all(dest_dir)?;
    }
    fs::create_dir_all(dest_dir)?;
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| crate::error::AppError::msg(format!("Pack branding corrupto: {e}")))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name.ends_with('/') {
            fs::create_dir_all(dest_dir.join(&name))?;
            continue;
        }
        let out_path = dest_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = File::create(&out_path)?;
        std::io::copy(&mut file, &mut out)?;
    }
    Ok(())
}

fn write_zip_file(bytes: &[u8], dest: &Path) -> AppResult<()> {
    if dest.is_file() {
        fs::remove_file(dest)?;
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(dest, bytes)?;
    Ok(())
}

/// Copia el pack pre-generado correcto a la carpeta de juego.
pub fn deploy(game_dir: &Path, profile: PackProfile) -> AppResult<()> {
    let bytes = embedded_zip(profile);

    match profile {
        PackProfile::Classic => {
            let tp_dir = game_dir.join("texturepacks");
            fs::create_dir_all(&tp_dir)?;
            write_zip_file(bytes, &tp_dir.join(format!("{PACK_NAME}.zip")))?;
        }
        PackProfile::Modern => {
            let rp = game_dir.join("resourcepacks");
            fs::create_dir_all(&rp)?;
            // Carpeta (parcheo de skins offline) + zip (schema 1.21.5+ / 26.x).
            extract_zip(bytes, &rp.join(PACK_NAME))?;
            write_zip_file(bytes, &rp.join(format!("{PACK_NAME}.zip")))?;
        }
        PackProfile::Legacy | PackProfile::Standard | PackProfile::StandardRange | PackProfile::Wide => {
            let rp = game_dir.join("resourcepacks");
            fs::create_dir_all(&rp)?;
            let zip_path = rp.join(format!("{PACK_NAME}.zip"));
            if zip_path.is_file() {
                fs::remove_file(&zip_path)?;
            }
            extract_zip(bytes, &rp.join(PACK_NAME))?;
        }
    }

    Ok(())
}

/// Re-empaqueta la carpeta de branding (p. ej. tras inyectar skin offline).
pub fn rebuild_zip_if_needed(game_dir: &Path) -> AppResult<()> {
    let pack_dir = game_dir.join("resourcepacks").join(PACK_NAME);
    let zip_path = game_dir.join("resourcepacks").join(format!("{PACK_NAME}.zip"));
    if !pack_dir.is_dir() || !zip_path.is_file() {
        return Ok(());
    }
    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    walk_into_zip(&pack_dir, &pack_dir, &mut zip, opts)?;
    zip.finish()?;
    Ok(())
}

fn walk_into_zip(
    base: &Path,
    current: &Path,
    zip: &mut ZipWriter<File>,
    opts: SimpleFileOptions,
) -> AppResult<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'))
        {
            continue;
        }
        if path.is_dir() {
            walk_into_zip(base, &path, zip, opts)?;
        } else {
            let arc = path.strip_prefix(base).unwrap_or(&path);
            let name = arc.to_string_lossy().replace('\\', "/");
            zip.start_file(name, opts)?;
            let mut f = File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}
