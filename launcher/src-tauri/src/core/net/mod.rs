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
use std::sync::atomic::{AtomicUsize, Ordering};
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

fn emit(app: &AppHandle, id: &str, label: &str, progress: f64, status: &str) {
    let _ = app.emit(
        "download://progress",
        DownloadProgress {
            id: id.to_string(),
            label: label.to_string(),
            progress,
            status: status.to_string(),
            speed: String::new(),
        },
    );
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

    let resp = client.get(&item.url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;

    if let Some(expected) = &item.sha1 {
        let got = sha1_hex(&bytes);
        if !got.eq_ignore_ascii_case(expected) {
            return Err(AppError::msg(format!(
                "SHA-1 invalido para {} (esperado {expected}, obtenido {got})",
                item.dest.display()
            )));
        }
    }

    let tmp = item.dest.with_extension("part");
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, &item.dest)?;
    Ok(bytes.len() as u64)
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
    let concurrency = concurrency.clamp(1, 32);

    emit(app, group_id, label, 0.0, "downloading");

    let mut stream = stream::iter(items.into_iter().map(|item| {
        let client = client.clone();
        let app = app.clone();
        let done = done.clone();
        let group = group_id.to_string();
        let label = label.to_string();
        async move {
            let res = download_one(&client, &item).await;
            let n = done.fetch_add(1, Ordering::SeqCst) + 1;
            let pct = (n as f64 / total as f64) * 100.0;
            emit(&app, &group, &label, pct, "downloading");
            res.map(|_| ())
        }
    }))
    .buffer_unordered(concurrency);

    while let Some(res) = stream.next().await {
        res?;
    }
    emit(app, group_id, label, 100.0, "done");
    Ok(())
}

/// Descarga un recurso a memoria (JSON/metadata). No toca disco.
pub async fn fetch_bytes(client: &reqwest::Client, url: &str) -> AppResult<Vec<u8>> {
    let resp = client.get(url).send().await?.error_for_status()?;
    Ok(resp.bytes().await?.to_vec())
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
) -> AppResult<T> {
    let resp = client.get(url).send().await?.error_for_status()?;
    Ok(resp.json::<T>().await?)
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
