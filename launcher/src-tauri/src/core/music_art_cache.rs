//! Cache de carátulas de música en disco (Fase 2.2 — carátula robusta).
//!
//! El launcher descarga la imagen al detectar una URL nueva en `set_music()`
//! y la guarda en `{data}/music-art/{sha1(url)}.jpg`. El cliente Modern
//! prioriza leer este archivo local (via IPC, ruta escrita en el campo de
//! imagen cuando ya está lista) antes de golpear la red él mismo — evita
//! duplicar descargas y falla menos por Accept/DNS/proxy en el juego.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::core::net;
use crate::core::paths;

static IN_FLIGHT: Mutex<Option<String>> = Mutex::new(None);

fn cache_dir() -> PathBuf {
    paths::data_dir().join("music-art")
}

pub fn path_for_url(url: &str) -> PathBuf {
    let hash = net::sha1_hex(url.as_bytes());
    cache_dir().join(format!("{hash}.jpg"))
}

/// Devuelve la ruta local si ya está cacheada. Si no, encola la descarga en
/// background (no bloquea) y devuelve `None` — la próxima consulta (o el
/// cliente por HTTP mientras tanto) la encontrará lista.
pub fn ensure_cached(client: reqwest::Client, url: &str) -> Option<PathBuf> {
    let url = url.trim();
    if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
        return None;
    }
    let path = path_for_url(url);
    if path.is_file() {
        return Some(path);
    }
    {
        let mut guard = IN_FLIGHT.lock().unwrap();
        if guard.as_deref() == Some(url) {
            return None;
        }
        *guard = Some(url.to_string());
    }
    let url_owned = url.to_string();
    let path_owned = path.clone();
    tauri::async_runtime::spawn(async move {
        let _ = download(&client, &url_owned, &path_owned).await;
        if let Ok(mut guard) = IN_FLIGHT.lock() {
            if guard.as_deref() == Some(url_owned.as_str()) {
                *guard = None;
            }
        }
    });
    None
}

async fn download(client: &reqwest::Client, url: &str, path: &PathBuf) -> Result<(), String> {
    let resp = client
        .get(url)
        .header("User-Agent", "ParaguacraftLauncher/1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() < 64 {
        return Err("Respuesta demasiado corta para ser una imagen".into());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("jpg.tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Limpieza opcional: evita que la carpeta crezca sin límite (mantiene las
/// ~200 más recientes).
pub fn prune(max_files: usize) {
    let dir = cache_dir();
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return;
    };
    let mut entries: Vec<_> = rd
        .flatten()
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let modified = meta.modified().ok()?;
            Some((e.path(), modified))
        })
        .collect();
    if entries.len() <= max_files {
        return;
    }
    entries.sort_by_key(|(_, m)| *m);
    let excess = entries.len() - max_files;
    for (path, _) in entries.into_iter().take(excess) {
        let _ = std::fs::remove_file(path);
    }
}
