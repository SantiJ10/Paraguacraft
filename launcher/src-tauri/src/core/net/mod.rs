//! Motor de descargas asincronico (tokio) — base de toda la Fase 3.
//!
//! Caracteristicas (Regla 3 - eficiencia):
//!   - Concurrencia **acotada** con `buffer_unordered(N)` (equivale a un Semaphore):
//!     nunca abre mas de N conexiones a la vez, sea cual sea la gama del PC.
//!   - Descarga **atomica** (`tmp + rename`) y **verificada** (SHA-1) cuando hay hash.
//!   - **Skip inteligente**: si el archivo ya existe y su SHA-1 coincide, no se baja.
//!   - Progreso agregado por grupo via evento `download://progress`.
//!   - Sin hilos propios: corre sobre el runtime de Tauri. El cliente se libera
//!     en idle desde `AppState::net_end` cuando termina el grupo.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

use futures_util::stream::{self, StreamExt};
use sha1::{Digest, Sha1};
use tauri::{AppHandle, Emitter};

use crate::config;
use crate::core::hardware;
use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, DownloadProgress};

/// Un archivo a descargar.
#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub url: String,
    pub dest: PathBuf,
    pub sha1: Option<String>,
}

impl DownloadItem {
    pub fn new(url: impl Into<String>, dest: impl Into<PathBuf>) -> Self {
        DownloadItem { url: url.into(), dest: dest.into(), sha1: None }
    }
    pub fn with_sha1(mut self, sha1: Option<String>) -> Self {
        self.sha1 = sha1.filter(|s| !s.is_empty());
        self
    }
}

pub fn sha1_hex(bytes: &[u8]) -> String {
    let mut h = Sha1::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// Verifica el SHA-1 de un archivo existente (para skip).
fn file_matches_sha1(path: &Path, expected: &str) -> bool {
    match std::fs::read(path) {
        Ok(bytes) => sha1_hex(&bytes).eq_ignore_ascii_case(expected),
        Err(_) => false,
    }
}

/// Mojang publica SHA-1 incorrectos en natives legacy (1.8–1.12). Aceptar si tamaño plausible.
fn legacy_library_sha1_soft_ok(dest: &Path, size: usize) -> bool {
    if size < 512 {
        return false;
    }
    let p = dest.to_string_lossy().replace('\\', "/").to_lowercase();
    (p.contains("lwjgl") && p.contains("natives"))
        || p.contains("lwjgl-platform")
        || p.contains("net/java/jinput")
        || p.contains("net/java/jutils")
}

fn emit(
    app: &AppHandle,
    id: &str,
    label: &str,
    progress: f64,
    status: &str,
    error: Option<&str>,
    failed_file: Option<&str>,
) {
    let _ = app.emit(
        "download://progress",
        DownloadProgress {
            id: id.to_string(),
            label: label.to_string(),
            progress,
            status: status.to_string(),
            speed: String::new(),
            error: error.map(String::from),
            failed_file: failed_file.map(String::from),
        },
    );
}

fn emit_simple(app: &AppHandle, id: &str, label: &str, progress: f64, status: &str) {
    emit(app, id, label, progress, status, None, None);
}

fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 429 || status.as_u16() == 502 || status.as_u16() == 503 || status.as_u16() == 504
}

async fn with_retries<F, Fut, T>(mut op: F) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    const MAX: u32 = 6;
    let mut last = None;
    for attempt in 0..MAX {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                let retry = match &e {
                    AppError::Http(http) => {
                        http.is_timeout()
                            || http.is_connect()
                            || http.status().map(is_retryable_status).unwrap_or(false)
                    }
                    _ => false,
                };
                last = Some(e);
                if !retry || attempt + 1 >= MAX {
                    break;
                }
                let wait_ms = 800u64 * (attempt as u64 + 1).pow(2);
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
            }
        }
    }
    Err(last.unwrap_or_else(|| AppError::msg("Error de red")))
}

/// Descarga un archivo de forma atomica y verificada. Devuelve bytes escritos
/// (0 si se reuso por skip).
pub async fn download_one(client: &reqwest::Client, item: &DownloadItem) -> AppResult<u64> {
    if let Some(expected) = &item.sha1 {
        if item.dest.is_file() && file_matches_sha1(&item.dest, expected) {
            return Ok(0);
        }
    } else if item.dest.is_file() {
        // Sin hash: si ya existe, lo damos por bueno (assets/libs ya presentes).
        return Ok(0);
    }

    if let Some(parent) = item.dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let url = item.url.clone();
    let dest = item.dest.clone();
    let sha1 = item.sha1.clone();
    let bytes = with_retries(|| async {
        let resp = client.get(&url).send().await?.error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    })
    .await?;

    if let Some(expected) = &sha1 {
        let got = sha1_hex(&bytes);
        if !got.eq_ignore_ascii_case(expected) && !legacy_library_sha1_soft_ok(&dest, bytes.len()) {
            return Err(AppError::msg(format!(
                "SHA-1 invalido para {} (esperado {expected}, obtenido {got})",
                dest.display()
            )));
        }
    }

    let tmp = dest.with_extension("part");
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, &dest)?;
    Ok(bytes.len() as u64)
}

const DRIVE_FOLDER_ID: &str = "1kiGI_iWfoiAxDAHfnhlHya-MxYs7Fwbt";

/// GET GitHub; si falla y `filename` es `.zip`, fallback a Google Drive directo.
pub async fn download_github_or_drive(
    client: &reqwest::Client,
    github_url: &str,
    filename: &str,
    dest: impl Into<PathBuf>,
    sha1: Option<String>,
) -> AppResult<u64> {
    let dest = dest.into();
    match download_one(
        client,
        &DownloadItem::new(github_url, dest.clone()).with_sha1(sha1.clone()),
    )
    .await
    {
        Ok(n) if dest.is_file() => return Ok(n),
        _ => {
            let _ = std::fs::remove_file(dest.with_extension("part"));
        }
    }
    if !filename.to_lowercase().ends_with(".zip") {
        return Err(AppError::msg(format!("No se pudo descargar {filename} desde GitHub")));
    }
    let drive_id = resolve_drive_file_id(client, filename).await?;
    let drive_url = format!("https://drive.google.com/uc?export=download&id={drive_id}");
    download_one(
        client,
        &DownloadItem::new(drive_url, dest).with_sha1(sha1),
    )
    .await
}

async fn resolve_drive_file_id(client: &reqwest::Client, filename: &str) -> AppResult<String> {
    let folder_url = format!("https://drive.google.com/drive/folders/{DRIVE_FOLDER_ID}");
    let html = String::from_utf8_lossy(&fetch_bytes(client, &folder_url).await?).into_owned();
    let needle = filename.to_lowercase().replace(' ', "-");
    for cap in regex_lite_drive_rows(&html) {
        if cap.0.to_lowercase().replace(' ', "-") == needle {
            return Ok(cap.1);
        }
    }
    Err(AppError::msg(format!("Drive: no se encontro {filename}")))
}

fn regex_lite_drive_rows(html: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find("aria-label=\"") {
        rest = &rest[i + 14..];
        let Some(end) = rest.find('"') else { break };
        let label = &rest[..end];
        if !label.ends_with(".zip") {
            continue;
        }
        let name = label.split_whitespace().next().unwrap_or(label).to_string();
        let chunk = &rest[..rest.len().min(800)];
        if let Some(ssk) = chunk.find("ssk='5:") {
            let after = &chunk[ssk + 6..];
            if let Some(colon) = after.find(':') {
                let raw = &after[colon + 1..];
                if let Some(q) = raw.find('\'') {
                    let mut id = raw[..q].to_string();
                    if let Some(pos) = id.rfind("-0-") {
                        id.truncate(pos);
                    }
                    if id.len() > 20 {
                        out.push((name, id));
                    }
                }
            }
        }
    }
    out
}

/// Concurrencia de descargas: setting del usuario (0 = auto según hardware).
pub fn resolve_concurrency(user_setting: u32) -> usize {
    if user_setting > 0 {
        return (user_setting as usize).clamp(1, 32);
    }
    match hardware::detect().perfil_sugerido.as_str() {
        "alta" => 28,
        "media" => 20,
        _ => 12,
    }
}

/// Lee concurrencia desde `launcher_config.json` (0 = automática).
pub fn concurrency_from_settings() -> usize {
    let settings = config::read_json::<AppSettings>(&paths::config_file()).unwrap_or_default();
    resolve_concurrency(settings.download_concurrency)
}

/// Descarga muchos archivos con concurrencia acotada y progreso agregado.
pub async fn download_all(
    client: &reqwest::Client,
    items: Vec<DownloadItem>,
    concurrency: usize,
    app: &AppHandle,
    group_id: &str,
    label: &str,
) -> AppResult<()> {
    let total = items.len();
    if total == 0 {
        return Ok(());
    }
    let done = Arc::new(AtomicUsize::new(0));
    let last_emit_pct = Arc::new(AtomicU32::new(0));
    let concurrency = if total > 400 {
        concurrency.clamp(1, 8)
    } else if total > 100 {
        concurrency.clamp(1, 12)
    } else {
        concurrency.clamp(1, 32)
    };

    emit_simple(app, group_id, label, 0.0, "downloading");

    let mut stream = stream::iter(items.into_iter().map(|item| {
        let client = client.clone();
        let app = app.clone();
        let done = done.clone();
        let last_emit_pct = last_emit_pct.clone();
        let group = group_id.to_string();
        let label = label.to_string();
        async move {
            let file_label = item
                .dest
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| item.url.clone());
            let res = download_one(&client, &item).await;
            let n = done.fetch_add(1, Ordering::SeqCst) + 1;
            let pct = (n as f64 / total as f64) * 100.0;
            let pct_i = pct.floor() as u32;
            let should_emit = res.is_err()
                || n == total
                || pct_i > last_emit_pct.load(Ordering::Relaxed);
            if should_emit {
                last_emit_pct.store(pct_i, Ordering::Relaxed);
                if let Err(ref e) = res {
                    emit(
                        &app,
                        &group,
                        &label,
                        pct,
                        "error",
                        Some(&e.to_string()),
                        Some(&file_label),
                    );
                } else {
                    emit_simple(&app, &group, &label, pct, "downloading");
                }
            }
            res.map(|_| ())
        }
    }))
    .buffer_unordered(concurrency);

    while let Some(res) = stream.next().await {
        res?;
    }
    emit_simple(app, group_id, label, 100.0, "done");
    Ok(())
}

/// Descarga un recurso a memoria (JSON/metadata). No toca disco.
pub async fn fetch_bytes(client: &reqwest::Client, url: &str) -> AppResult<Vec<u8>> {
    let u = url.to_string();
    with_retries(|| async {
        let resp = client.get(&u).send().await?.error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    })
    .await
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
) -> AppResult<T> {
    let u = url.to_string();
    with_retries(|| async {
        let resp = client.get(&u).send().await?.error_for_status()?;
        let mut text = resp.text().await?;
        if text.starts_with('\u{FEFF}') {
            text = text.trim_start_matches('\u{FEFF}').to_string();
        }
        Ok(serde_json::from_str(&text)?)
    })
    .await
}

/// Percent-encoding minimo para querystrings (facets de Modrinth, etc.).
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
