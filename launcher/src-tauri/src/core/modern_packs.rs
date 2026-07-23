//! Resource packs PvP 1.21.11 — descarga global vía GitHub Releases (como 1.8.9).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

const GITHUB_REPO: &str = "SantiJ10/Paraguacraft";
const CATALOG_URL: &str =
    "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/clientes/paraguacraft-pvp-modern/packs/catalog.json";

const CATALOG_MIRRORS: &[&str] = &[
    CATALOG_URL,
    "https://cdn.jsdelivr.net/gh/SantiJ10/Paraguacraft@main/clientes/paraguacraft-pvp-modern/packs/catalog.json",
];

#[derive(Debug, Clone, Deserialize)]
struct PackCatalog {
    #[serde(default = "default_release_tag", rename = "releaseTag")]
    release_tag: String,
    #[serde(default, rename = "baseUrl")]
    base_url: String,
    #[serde(default)]
    packs: Vec<PackEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct PackEntry {
    #[serde(rename = "fileName")]
    file_name: String,
    sha1: String,
}

fn default_release_tag() -> String {
    "pvp-packs-modern-1.0".into()
}

impl Default for PackCatalog {
    fn default() -> Self {
        Self {
            release_tag: default_release_tag(),
            base_url: format!(
                "https://github.com/{GITHUB_REPO}/releases/download/pvp-packs-modern-1.0"
            ),
            packs: vec![],
        }
    }
}

fn bundled_roots(app: &AppHandle) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rel in ["bundled/pvp-modern", "resources/bundled/pvp-modern"] {
        if let Ok(p) = app.path().resolve(rel, BaseDirectory::Resource) {
            roots.push(p);
        }
    }
    if let Ok(dir) = app.path().resource_dir() {
        roots.push(dir.join("bundled").join("pvp-modern"));
    }
    roots.push(paths::data_dir().join("bundled").join("pvp-modern"));
    roots
}

fn read_bundled_catalog(app: &AppHandle) -> Option<PackCatalog> {
    for root in bundled_roots(app) {
        let path = root.join("packs").join("catalog.json");
        if !path.is_file() {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(c) = serde_json::from_str::<PackCatalog>(&text) {
                if !c.packs.is_empty() {
                    return Some(c);
                }
            }
        }
    }
    None
}

async fn fetch_catalog(app: &AppHandle, client: &reqwest::Client) -> PackCatalog {
    for url in CATALOG_MIRRORS {
        if let Ok(c) = net::fetch_json::<PackCatalog>(client, url).await {
            if !c.packs.is_empty() {
                return c;
            }
        }
    }
    read_bundled_catalog(app).unwrap_or_default()
}

fn file_sha1(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(net::sha1_hex(&bytes))
}

fn sha_matches(path: &Path, expected: &str) -> bool {
    if expected.chars().all(|c| c == '0') {
        return path.is_file();
    }
    file_sha1(path)
        .map(|h| h.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn find_local_pack(app: &AppHandle, filename: &str) -> Option<PathBuf> {
    for root in bundled_roots(app) {
        let p = root.join("resourcepacks").join(filename);
        if p.is_file() && p.metadata().map(|m| m.len()).unwrap_or(0) > 1024 {
            return Some(p);
        }
    }
    None
}

fn pack_urls(base_url: &str, release_tag: &str, filename: &str) -> Vec<String> {
    let base = if base_url.is_empty() {
        format!("https://github.com/{GITHUB_REPO}/releases/download/{release_tag}")
    } else {
        base_url.trim_end_matches('/').to_string()
    };
    vec![
        format!("{base}/{filename}"),
        format!("https://github.com/{GITHUB_REPO}/releases/download/{release_tag}/{filename}"),
    ]
}

async fn ensure_pack(
    app: &AppHandle,
    client: &reqwest::Client,
    base_url: &str,
    release_tag: &str,
    entry: &PackEntry,
    dest_dir: &Path,
) -> AppResult<bool> {
    let dest = dest_dir.join(&entry.file_name);
    if sha_matches(&dest, &entry.sha1) {
        return Ok(false);
    }

    if let Some(src) = find_local_pack(app, &entry.file_name) {
        if sha_matches(&src, &entry.sha1) {
            std::fs::create_dir_all(dest_dir)?;
            std::fs::copy(&src, &dest)?;
            return Ok(true);
        }
    }

    let tmp = dest.with_extension("part");
    for url in pack_urls(base_url, release_tag, &entry.file_name) {
        if net::download_one(client, &DownloadItem::new(url, tmp.clone()))
            .await
            .is_err()
        {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        if !sha_matches(&tmp, &entry.sha1) {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        std::fs::create_dir_all(dest_dir)?;
        if dest.exists() {
            let _ = std::fs::remove_file(&dest);
        }
        std::fs::rename(&tmp, &dest)?;
        return Ok(true);
    }

    if let Some(src) = find_local_pack(app, &entry.file_name) {
        std::fs::create_dir_all(dest_dir)?;
        std::fs::copy(&src, &dest)?;
        return Ok(true);
    }

    Err(AppError::msg(format!(
        "No se pudo obtener el pack {}",
        entry.file_name
    )))
}

const OFFICIAL_PACK: &str = "paraguacraft-pvp-modern.zip";
const BRAND_PACK: &str = "ParaguacraftBrandPack";

/// Elimina packs PvP de terceros dejando solo el oficial y el brand del launcher.
pub fn purge_non_official_packs(instance_dir: &Path) {
    let dir = instance_dir.join("resourcepacks");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.eq_ignore_ascii_case(OFFICIAL_PACK) || name.eq_ignore_ascii_case(BRAND_PACK) {
            continue;
        }
        if path.is_dir() {
            continue;
        }
        if name.ends_with(".zip") {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn official_entry(catalog: &PackCatalog) -> Option<PackEntry> {
    catalog
        .packs
        .iter()
        .find(|p| p.file_name.eq_ignore_ascii_case(OFFICIAL_PACK))
        .cloned()
        .or_else(|| {
            catalog.packs.first().filter(|p| {
                p.file_name.eq_ignore_ascii_case(OFFICIAL_PACK)
                    || p.file_name.to_lowercase().starts_with("paraguacraft-pvp")
            }).cloned()
        })
}

/// Descarga solo el pack oficial PvP 1.21.11 al directorio `resourcepacks/` de la instancia.
pub async fn sync_instance_packs(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_dir: &Path,
) -> AppResult<u32> {
    let catalog = fetch_catalog(app, client).await;
    let dest = instance_dir.join("resourcepacks");
    std::fs::create_dir_all(&dest)?;
    purge_non_official_packs(instance_dir);
    let Some(entry) = official_entry(&catalog) else {
        return Err(AppError::msg(format!(
            "Catálogo sin pack oficial {OFFICIAL_PACK}"
        )));
    };
    let installed = if ensure_pack(
        app,
        client,
        &catalog.base_url,
        &catalog.release_tag,
        &entry,
        &dest,
    )
    .await?
    {
        1
    } else {
        0
    };
    Ok(installed)
}
