//! Libro de overrides de un modpack (Prism `overrides.txt`).
//! Al reimportar/actualizar se borran los archivos de la versión anterior.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::AppResult;

const LIST_FILE: &str = ".paraguacraft-overrides.txt";

pub fn list_path(instance_dir: &Path) -> PathBuf {
    instance_dir.join(LIST_FILE)
}

pub fn read_list(instance_dir: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(list_path(instance_dir)) else {
        return Vec::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

pub fn write_list(instance_dir: &Path, rels: &[String]) -> AppResult<()> {
    let mut f = fs::File::create(list_path(instance_dir))?;
    writeln!(f, "# overrides del último modpack (no editar a mano)")?;
    for r in rels {
        writeln!(f, "{r}")?;
    }
    Ok(())
}

/// Borra los overrides de la importación anterior (archivos listados, no carpetas vacías críticas).
pub fn cleanup_previous(instance_dir: &Path) -> AppResult<u32> {
    let mut n = 0u32;
    for rel in read_list(instance_dir) {
        let p = instance_dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        if p.is_file() {
            let _ = fs::remove_file(&p);
            n += 1;
        }
    }
    Ok(n)
}

/// Relativos dentro de un ZIP para los prefijos dados (`overrides/`, `client-overrides/`, …).
pub fn collect_from_zip(bytes: &[u8], prefixes: &[&str]) -> AppResult<Vec<String>> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut out = Vec::new();
    for i in 0..zip.len() {
        let entry = zip.by_index(i)?;
        let Some(name) = entry.enclosed_name().map(|p| p.to_string_lossy().replace('\\', "/")) else {
            continue;
        };
        if entry.is_dir() {
            continue;
        }
        for prefix in prefixes {
            let p = if prefix.ends_with('/') {
                prefix.to_string()
            } else {
                format!("{prefix}/")
            };
            if let Some(rel) = name.strip_prefix(&p) {
                if !rel.is_empty() {
                    out.push(rel.to_string());
                }
                break;
            }
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

pub fn record_from_zip(instance_dir: &Path, bytes: &[u8], prefixes: &[&str]) -> AppResult<()> {
    let rels = collect_from_zip(bytes, prefixes)?;
    write_list(instance_dir, &rels)
}

/// Extrae un archivo concreto del zip (tests / callers internos).
#[allow(dead_code)]
pub fn read_zip_entry(bytes: &[u8], name: &str) -> AppResult<Vec<u8>> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut file = zip.by_name(name)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
