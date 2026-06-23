//! Preset **Fabric + Iris** (loader separado de Fabric).
//!
//! Instala el perfil Fabric (via API oficial) y luego el bundle de optimizacion
//! (Sodium, Iris, etc.) desde Modrinth, con cache global por version de MC
//! (espejo de `instalar_bundle_fabric_iris_cache` en core.py).

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

use super::fabric;

const BUNDLE_SLUGS: &[&str] = &[
    "fabric-api",
    "sodium",
    "iris",
    "lithium",
    "ferrite-core",
    "entityculling",
    "immediatelyfast",
    "modmenu",
];

const CACHE_DAYS: u64 = 30;
const MODRINTH: &str = "https://api.modrinth.com/v2";

fn cache_root(mc: &str) -> PathBuf {
    paths::default_minecraft_dir()
        .join("Paraguacraft_cache")
        .join("fabric_iris")
        .join(mc.replace('/', "_"))
}

fn cache_valid(marker: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(marker) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return false;
    };
    age.as_secs() / 86400 < CACHE_DAYS
}

/// Versiones de Fabric disponibles (mismo criterio que el loader Fabric puro).
pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    fabric::versions(client, mc).await
}

/// Instala perfil Fabric y devuelve el version id.
pub async fn install_fabric_profile(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader_version: &str,
) -> AppResult<String> {
    fabric::install(app, client, mc, loader_version).await
}

/// Descarga/actualiza el bundle en cache y copia a `instance_dir/mods`.
pub async fn install_bundle(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    instance_dir: &Path,
) -> AppResult<()> {
    let cache = cache_root(mc);
    std::fs::create_dir_all(&cache)?;
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let marker = cache.join(".cache_ok");

    if !cache_valid(&marker) {
        if marker.exists() {
            for e in std::fs::read_dir(&cache).into_iter().flatten().flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("jar") {
                    let _ = std::fs::remove_file(p);
                }
            }
        }
        let mut items = Vec::new();
        for slug in BUNDLE_SLUGS {
            if let Some(item) = modrinth_jar_item(client, slug, mc).await? {
                items.push(
                    DownloadItem::new(item.url, cache.join(&item.filename)).with_sha1(item.sha1),
                );
            }
        }
        if !items.is_empty() {
            net::download_all(
                client,
                items,
                6,
                app,
                "fabric-iris-bundle",
                &format!("Mods Fabric + Iris ({mc})"),
            )
            .await?;
        }
        let _ = std::fs::write(&marker, b"ok");
    }

    let inst_lower: std::collections::HashSet<String> = std::fs::read_dir(&mods_dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_lowercase())
        .collect();

    for e in std::fs::read_dir(&cache).into_iter().flatten().flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("jar") {
            continue;
        }
        let fname = p.file_name().unwrap_or_default().to_string_lossy().to_string();
        let dest = mods_dir.join(&fname);
        if dest.exists() || dest.with_extension("jar.disabled").exists() {
            continue;
        }
        let slug_hit = BUNDLE_SLUGS
            .iter()
            .any(|s| fname.to_lowercase().contains(&s.to_lowercase()));
        if slug_hit
            && BUNDLE_SLUGS
                .iter()
                .any(|s| inst_lower.iter().any(|f| f.contains(&s.to_lowercase())))
        {
            continue;
        }
        let _ = std::fs::copy(&p, &dest);
    }
    Ok(())
}

struct JarItem {
    url: String,
    filename: String,
    sha1: Option<String>,
}

async fn modrinth_jar_item(client: &reqwest::Client, slug: &str, mc: &str) -> AppResult<Option<JarItem>> {
    let url = format!(
        "{MODRINTH}/project/{slug}/version?loaders={}&game_versions={}",
        net::url_encode(r#"["fabric"]"#),
        net::url_encode(&format!(r#"["{mc}"]"#))
    );
    let versions: Value = net::fetch_json(client, &url).await?;
    let arr = versions.as_array().cloned().unwrap_or_default();
    let Some(version) = arr.first() else {
        return Ok(None);
    };
    let files = version["files"].as_array().cloned().unwrap_or_default();
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool().unwrap_or(false))
        .or_else(|| files.first())
        .ok_or_else(|| AppError::msg(format!("Modrinth: {slug} sin archivos")))?;
    Ok(Some(JarItem {
        url: file["url"].as_str().unwrap_or_default().to_string(),
        filename: file["filename"].as_str().unwrap_or("mod.jar").to_string(),
        sha1: file["hashes"]["sha1"].as_str().map(String::from),
    }))
}
