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

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
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

/// SHA-1 de un archivo leyendo por bloques (un index de assets son cientos de
/// MB; cargarlos enteros en RAM dispara el pico de memoria del launcher).
fn sha1_file(path: &Path) -> std::io::Result<String> {
    let (hasher, _) = seed_hasher(path)?;
    Ok(hex::encode(hasher.finalize()))
}

/// Estado del SHA-1 tras consumir el archivo entero, más cuántos bytes cubrió.
///
/// Para reanudar hace falta el hash a medias, no el digest final: se siembra
/// con el `.part` y se sigue alimentando con lo que va llegando.
fn seed_hasher(path: &Path) -> std::io::Result<(Sha1, u64)> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        total += n as u64;
    }
    Ok((hasher, total))
}

/// Máximo de archivos recordados; al pasarse se descarta todo y se rehashea.
const VERIFY_CACHE_MAX: usize = 40_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifyEntry {
    len: u64,
    mtime: u64,
    sha1: String,
}

#[derive(Default)]
struct VerifyCache {
    entries: HashMap<String, VerifyEntry>,
    dirty: bool,
}

fn verify_cache_path() -> PathBuf {
    paths::data_dir().join("download-verify.json")
}

fn verify_cache() -> &'static Mutex<VerifyCache> {
    static CACHE: OnceLock<Mutex<VerifyCache>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let entries = std::fs::read_to_string(verify_cache_path())
            .ok()
            .and_then(|raw| serde_json::from_str::<HashMap<String, VerifyEntry>>(&raw).ok())
            .unwrap_or_default();
        Mutex::new(VerifyCache { entries, dirty: false })
    })
}

/// Vuelca la caché a disco. Se llama al cerrar cada grupo de descargas.
pub fn flush_verify_cache() {
    let Ok(mut cache) = verify_cache().lock() else {
        return;
    };
    if !cache.dirty {
        return;
    }
    cache.dirty = false;
    if cache.entries.len() > VERIFY_CACHE_MAX {
        cache.entries.clear();
    }
    let path = verify_cache_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(&cache.entries) {
        let _ = std::fs::write(path, json);
    }
}

/// Identidad barata de un archivo: si tamaño y mtime no cambiaron, el
/// contenido tampoco (y el SHA-1 guardado sigue valiendo).
fn file_signature(path: &Path) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some((meta.len(), mtime))
}

fn remember_verified(path: &Path, len: u64, mtime: u64, sha1: String) {
    if let Ok(mut cache) = verify_cache().lock() {
        cache
            .entries
            .insert(path.to_string_lossy().into_owned(), VerifyEntry { len, mtime, sha1 });
        cache.dirty = true;
    }
}

/// Verifica el SHA-1 de un archivo existente (para skip).
///
/// Bloquea: llamar siempre dentro de `spawn_blocking`.
fn file_matches_sha1(path: &Path, expected: &str) -> bool {
    let Some((len, mtime)) = file_signature(path) else {
        return false;
    };
    let key = path.to_string_lossy();
    if let Ok(cache) = verify_cache().lock() {
        if let Some(hit) = cache.entries.get(key.as_ref()) {
            if hit.len == len && hit.mtime == mtime {
                return hit.sha1.eq_ignore_ascii_case(expected);
            }
        }
    }
    let Ok(got) = sha1_file(path) else {
        return false;
    };
    remember_verified(path, len, mtime, got.clone());
    got.eq_ignore_ascii_case(expected)
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

fn http_retryable(e: &AppError) -> bool {
    match e {
        AppError::Http(http) => {
            http.is_timeout()
                || http.is_connect()
                || http.is_request()
                || http.status().map(is_retryable_status).unwrap_or(false)
        }
        AppError::Msg(s) => {
            let low = s.to_ascii_lowercase();
            low.contains("error sending request") || low.contains("error trying to connect")
        }
        _ => false,
    }
}

async fn with_retries_n<F, Fut, T>(mut op: F, max: u32) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    let mut last = None;
    for attempt in 0..max {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                let retry = http_retryable(&e);
                last = Some(e);
                if !retry || attempt + 1 >= max {
                    break;
                }
                let wait_ms = 400u64 * (attempt as u64 + 1).pow(2);
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
            }
        }
    }
    Err(last.unwrap_or_else(|| AppError::msg("Error de red")))
}

/// Espejos para CDN de Mojang (piston-meta a veces no responde en LATAM).
pub fn mirror_urls(url: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(4);
    let mut push = |u: String| {
        if !out.iter().any(|x| x == &u) {
            out.push(u);
        }
    };
    push(url.to_string());
    if let Some(rest) = url.strip_prefix("https://piston-meta.mojang.com/") {
        push(format!("https://launchermeta.mojang.com/{rest}"));
        push(format!("https://bmclapi2.bangbang93.com/{rest}"));
    }
    if let Some(rest) = url.strip_prefix("https://launchermeta.mojang.com/") {
        push(format!("https://piston-meta.mojang.com/{rest}"));
        push(format!("https://bmclapi2.bangbang93.com/{rest}"));
    }
    if let Some(rest) = url.strip_prefix("https://piston-data.mojang.com/") {
        push(format!("https://launcher.mojang.com/{rest}"));
        push(format!("https://bmclapi2.bangbang93.com/{rest}"));
    }
    if let Some(rest) = url.strip_prefix("https://resources.download.minecraft.net/") {
        push(format!("https://bmclapi2.bangbang93.com/assets/{rest}"));
    }
    if let Some(rest) = url.strip_prefix("https://libraries.minecraft.net/") {
        push(format!("https://bmclapi2.bangbang93.com/maven/{rest}"));
    }
    out
}

async fn get_bytes(client: &reqwest::Client, url: &str) -> AppResult<Vec<u8>> {
    let u = url.to_string();
    with_retries_n(
        || async {
            let resp = client.get(&u).send().await?.error_for_status()?;
            Ok(resp.bytes().await?.to_vec())
        },
        3,
    )
    .await
}

/// Descarga un archivo de forma atomica y verificada. Devuelve bytes escritos
/// (0 si se reuso por skip).
pub async fn download_one(client: &reqwest::Client, item: &DownloadItem) -> AppResult<u64> {
    if skip_existing(item).await {
        return Ok(0);
    }

    let dest = item.dest.clone();
    let sha1 = item.sha1.clone();
    let mut last_err = None;
    for url in mirror_urls(&item.url) {
        match with_retries_n(|| stream_download(client, &url, &dest, sha1.as_deref()), 3).await {
            Ok(n) => return Ok(n),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| AppError::msg("Error de red")))
}

/// Baja a `.part` escribiendo a medida que llegan los chunks, y reanuda si ya
/// había bytes de un intento anterior.
///
/// Antes el archivo entero se juntaba en RAM antes de tocar disco: con 28
/// descargas en paralelo y un `client.jar` de 100 MB el pico de memoria era
/// enorme, y cualquier corte obligaba a empezar de cero.
async fn stream_download(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
) -> AppResult<u64> {
    use tokio::io::AsyncWriteExt;

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = dest.with_extension("part");

    // Reanudar sin hash sería a ciegas: si el resto viene de otro espejo no hay
    // forma de detectar que el pegado quedó corrupto.
    let resumable = sha1.is_some();
    let have = if resumable {
        tokio::fs::metadata(&tmp).await.map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let mut req = client.get(url);
    if have > 0 {
        req = req.header(reqwest::header::RANGE, format!("bytes={have}-"));
    }
    let resp = req.send().await?.error_for_status()?;
    let resumed = have > 0 && resp.status() == reqwest::StatusCode::PARTIAL_CONTENT;

    // Si el servidor ignoró el Range hay que rehacer el archivo desde cero.
    let (mut hasher, mut written, mut file) = if resumed {
        let (seed, len) = hash_existing_part(tmp.clone()).await?;
        let handle = tokio::fs::OpenOptions::new().append(true).open(&tmp).await?;
        (seed, len, handle)
    } else {
        (Sha1::new(), 0u64, tokio::fs::File::create(&tmp).await?)
    };

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        written += chunk.len() as u64;
    }
    file.flush().await?;
    drop(file);

    if let Some(expected) = sha1 {
        let got = hex::encode(hasher.finalize());
        if !got.eq_ignore_ascii_case(expected)
            && !legacy_library_sha1_soft_ok(dest, written as usize)
        {
            // Sin borrarlo, el próximo intento reanudaría sobre bytes malos.
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err(AppError::msg(format!(
                "SHA-1 invalido para {} (esperado {expected}, obtenido {got})",
                dest.display()
            )));
        }
    }

    tokio::fs::rename(&tmp, dest).await?;
    if let Some(expected) = sha1 {
        if let Some((len, mtime)) = file_signature(dest) {
            remember_verified(dest, len, mtime, expected.to_string());
        }
    }
    Ok(written)
}

/// Rehace el SHA-1 de lo ya bajado, por bloques para no cargarlo en RAM.
async fn hash_existing_part(tmp: PathBuf) -> AppResult<(Sha1, u64)> {
    tokio::task::spawn_blocking(move || seed_hasher(&tmp))
        .await
    .unwrap_or_else(|e| Err(std::io::Error::other(format!("Lectura abortada: {e}"))))
    .map_err(AppError::from)
}

/// ¿Ya está en disco y verificado? `stat` + SHA-1 son sincrónicos, así que el
/// chequeo entero va a un hilo de bloqueo: con miles de assets, hacerlo sobre
/// el runtime congela el IPC hacia la UI.
async fn skip_existing(item: &DownloadItem) -> bool {
    let dest = item.dest.clone();
    let sha1 = item.sha1.clone();
    tokio::task::spawn_blocking(move || match sha1 {
        Some(expected) => dest.is_file() && file_matches_sha1(&dest, &expected),
        // Sin hash: si ya existe, lo damos por bueno (assets/libs ya presentes).
        None => dest.is_file(),
    })
    .await
    .unwrap_or(false)
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

    let mut outcome = Ok(());
    while let Some(res) = stream.next().await {
        if let Err(e) = res {
            outcome = Err(e);
            break;
        }
    }
    // Persistir lo verificado aunque el grupo haya fallado a la mitad.
    let _ = tokio::task::spawn_blocking(flush_verify_cache).await;
    outcome?;
    emit_simple(app, group_id, label, 100.0, "done");
    Ok(())
}

/// Comprueba si hay internet con un GET corto (sin reintentos largos).
/// Usar antes de instalar assets para poder lanzar 100% local si ya está descargado.
pub async fn is_online(client: &reqwest::Client) -> bool {
    const URLS: [&str; 3] = [
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
        "https://launchermeta.mojang.com/mc/game/version_manifest.json",
        "https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json",
    ];
    for url in URLS {
        let fut = client.get(url).send();
        match tokio::time::timeout(std::time::Duration::from_secs(3), fut).await {
            Ok(Ok(resp)) if resp.status().is_success() || resp.status().as_u16() == 304 => {
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Descarga un recurso a memoria (JSON/metadata). Prueba espejos de Mojang.
pub async fn fetch_bytes(client: &reqwest::Client, url: &str) -> AppResult<Vec<u8>> {
    let mut last = None;
    for candidate in mirror_urls(url) {
        match get_bytes(client, &candidate).await {
            Ok(b) => return Ok(b),
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| AppError::msg("Error de red")))
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
) -> AppResult<T> {
    let bytes = fetch_bytes(client, url).await?;
    let mut text = String::from_utf8_lossy(&bytes).into_owned();
    if text.starts_with('\u{FEFF}') {
        text = text.trim_start_matches('\u{FEFF}').to_string();
    }
    Ok(serde_json::from_str(&text)?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_file_matches_in_memory_hash() {
        let dir = std::env::temp_dir().join("pg-sha1-stream-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("blob.bin");
        // Más grande que el buffer de 64 KiB para cubrir el bucle de lectura.
        let data: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(&path, &data).unwrap();

        assert_eq!(sha1_file(&path).unwrap(), sha1_hex(&data));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn resumed_hash_equals_full_file_hash() {
        let dir = std::env::temp_dir().join("pg-resume-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("half.part");
        let whole: Vec<u8> = (0..150_000u32).map(|i| (i % 251) as u8).collect();
        // Corte en medio de un bloque de lectura, como un corte de red real.
        let (prefix, suffix) = whole.split_at(57_000);
        std::fs::write(&path, prefix).unwrap();

        let (mut hasher, seeded) = seed_hasher(&path).unwrap();
        assert_eq!(seeded, prefix.len() as u64, "el Range arranca donde quedó el .part");
        hasher.update(suffix);

        assert_eq!(
            hex::encode(hasher.finalize()),
            sha1_hex(&whole),
            "reanudar debe dar el mismo SHA-1 que bajar todo de una"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signature_changes_when_file_is_rewritten() {
        let dir = std::env::temp_dir().join("pg-sig-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sig.bin");
        std::fs::write(&path, b"uno").unwrap();
        let first = file_signature(&path).unwrap();
        std::fs::write(&path, b"uno y algo mas largo").unwrap();
        let second = file_signature(&path).unwrap();

        assert_ne!(first, second, "tamaño distinto debe invalidar la caché");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_has_no_signature() {
        assert!(file_signature(Path::new("no-existe-pg-test.bin")).is_none());
    }

    #[test]
    fn mirrors_piston_meta_asset_index() {
        let url = "https://piston-meta.mojang.com/v1/packages/412b37fe5b672961951430718c89127656b47ed5/29.json";
        let mirrors = mirror_urls(url);
        assert_eq!(mirrors[0], url);
        assert!(mirrors.iter().any(|u| u.contains("launchermeta.mojang.com")));
        assert!(mirrors.iter().any(|u| u.contains("bmclapi2.bangbang93.com")));
    }

    #[test]
    fn mirrors_leave_unrelated_urls() {
        let url = "https://cdn.modrinth.com/data/foo.jar";
        let mirrors = mirror_urls(url);
        assert_eq!(mirrors, vec![url.to_string()]);
    }
}
