//! Descarga de Eclipse Temurin (Adoptium) — espeja `descargar_adoptium`.
//!
//! Descarga el JRE para el major pedido, lo extrae y devuelve la ruta al binario
//! `java(w)`. Streaming con eventos de progreso; sin hilos propios (corre sobre
//! el runtime de Tauri). Auto-update: si ya existe un JRE de ese major, se
//! reutiliza salvo `force`.

use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};

use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::DownloadProgress;

fn adoptium_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    }
}

fn adoptium_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "x86" => "x86",
        "aarch64" => "aarch64",
        _ => "x64",
    }
}

fn emit(app: &AppHandle, id: &str, label: &str, progress: f64, status: &str) {
    let _ = app.emit(
        "download://progress",
        DownloadProgress {
            id: id.to_string(),
            label: label.to_string(),
            progress,
            status: status.to_string(),
            speed: String::new(),
            error: None,
            failed_file: None,
        },
    );
}

/// Devuelve la ruta al `java(w)` de un Temurin del `major` dado si ya esta instalado.
pub fn find_installed(major: u32) -> Option<PathBuf> {
    let root = paths::java_dir().join(format!("jre-{major}"));
    if !root.is_dir() {
        return None;
    }
    find_java_binary(&root)
}

/// Descarga e instala Temurin `major`. Si `force` es false y ya existe, lo reutiliza.
pub async fn download(
    app: &AppHandle,
    http: &reqwest::Client,
    major: u32,
    force: bool,
) -> AppResult<String> {
    if !force {
        if let Some(p) = find_installed(major) {
            return Ok(p.to_string_lossy().to_string());
        }
    }

    let id = format!("java-{major}");
    let label = format!("Descargando Java {major} (Temurin)");
    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{major}/ga/{}/{}/jre/hotspot/normal/eclipse",
        adoptium_os(),
        adoptium_arch()
    );

    emit(app, &id, &label, 0.0, "downloading");
    let resp = http.get(&url).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let is_zip = resp.url().path().ends_with(".zip") || cfg!(target_os = "windows");

    let ext = if is_zip { "zip" } else { "tar.gz" };
    let tmp = paths::java_dir().join(format!("temurin-{major}.{ext}"));

    let mut file = std::fs::File::create(&tmp)?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        if total > 0 {
            let pct = (downloaded as f64 / total as f64) * 85.0;
            emit(app, &id, &label, pct, "downloading");
        }
    }
    file.flush()?;
    drop(file);

    emit(app, &id, &format!("Extrayendo Java {major}"), 90.0, "downloading");

    let extract_root = paths::java_dir().join(format!("jre-{major}"));
    if extract_root.is_dir() {
        let _ = std::fs::remove_dir_all(&extract_root);
    }
    std::fs::create_dir_all(&extract_root)?;

    if is_zip {
        extract_zip(&tmp, &extract_root)?;
    } else {
        extract_tar_gz(&tmp, &extract_root)?;
    }
    let _ = std::fs::remove_file(&tmp);

    let java = find_java_binary(&extract_root)
        .ok_or_else(|| AppError::msg("No se encontro el binario Java tras extraer"))?;

    emit(app, &id, &format!("Java {major} listo"), 100.0, "done");
    Ok(java.to_string_lossy().to_string())
}

fn extract_zip(archive: &Path, dest: &Path) -> AppResult<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
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
            let mut out = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> AppResult<()> {
    let file = std::fs::File::open(archive)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);
    tar.unpack(dest)?;
    Ok(())
}

/// Busca recursivamente el binario `java(w)` dentro del JRE extraido.
fn find_java_binary(root: &Path) -> Option<PathBuf> {
    let target = if cfg!(target_os = "windows") { "javaw.exe" } else { "java" };
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().and_then(|n| n.to_str()) == Some(target) {
                return Some(p);
            }
        }
    }
    None
}
