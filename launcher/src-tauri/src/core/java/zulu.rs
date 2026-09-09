//! Descarga de Azul Zulu (Community) — JRE HotSpot para Minecraft.
//!
//! API: `https://api.azul.com/metadata/v1/zulu/packages/` (sin token, builds CA).
//! Se instala en `java/zulu-jre-{major}` para no pisar Temurin.

use std::io::Write;
use std::path::PathBuf;

use futures_util::StreamExt;
use serde::Deserialize;
use tauri::{AppHandle, Emitter};

use crate::core::java::adoptium;
use crate::core::net;
use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::DownloadProgress;

#[derive(Debug, Deserialize)]
struct ZuluPackage {
    download_url: String,
}

fn zulu_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

fn zulu_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "x86" => "x86",
        "aarch64" => "aarch64",
        _ => "x64",
    }
}

fn archive_type() -> &'static str {
    if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
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

pub fn find_installed(major: u32) -> Option<PathBuf> {
    let root = paths::java_dir().join(format!("zulu-jre-{major}"));
    if !root.is_dir() {
        return None;
    }
    adoptium::find_java_binary(&root)
}

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

    let id = format!("zulu-{major}");
    let label = format!("Descargando Java {major} (Zulu)");
    let meta_url = format!(
        "https://api.azul.com/metadata/v1/zulu/packages/?java_version={major}&os={}&arch={}&archive_type={}&javafx_bundled=false&java_package_type=jre&latest=true&release_status=ga&availability_types=CA&page_size=1",
        zulu_os(),
        zulu_arch(),
        archive_type(),
    );

    emit(app, &id, &label, 0.0, "downloading");
    let packages: Vec<ZuluPackage> = net::fetch_json(http, &meta_url).await?;
    let pkg = packages.first().ok_or_else(|| {
        AppError::msg(format!(
            "Azul Zulu no publicó un JRE {major} para este sistema"
        ))
    })?;

    let resp = http
        .get(&pkg.download_url)
        .send()
        .await?
        .error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let is_zip = pkg.download_url.contains(".zip") || cfg!(target_os = "windows");
    let ext = if is_zip { "zip" } else { "tar.gz" };
    let tmp = paths::java_dir().join(format!("zulu-{major}.{ext}"));

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

    emit(
        app,
        &id,
        &format!("Extrayendo Java {major} (Zulu)"),
        90.0,
        "downloading",
    );

    let extract_root = paths::java_dir().join(format!("zulu-jre-{major}"));
    if extract_root.is_dir() {
        let _ = std::fs::remove_dir_all(&extract_root);
    }
    std::fs::create_dir_all(&extract_root)?;

    if is_zip {
        adoptium::extract_zip(&tmp, &extract_root)?;
    } else {
        adoptium::extract_tar_gz(&tmp, &extract_root)?;
    }
    let _ = std::fs::remove_file(&tmp);

    let java = adoptium::find_java_binary(&extract_root)
        .ok_or_else(|| AppError::msg("No se encontro el binario Java tras extraer Zulu"))?;

    emit(app, &id, &format!("Java {major} (Zulu) listo"), 100.0, "done");
    Ok(java.to_string_lossy().to_string())
}
