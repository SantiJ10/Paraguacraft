//! Preset **Paraguacraft PvP** (solo Minecraft 1.8.9).
//!
//! Instala Forge 11.15.1.2318 y descarga en red:
//! - ParaguacraftPvP-1.0.0.jar desde GitHub (raw + release)
//! - OptiFine HD U M5 desde optifine.net (mirror oficial)

use std::path::{Path, PathBuf};

use serde_json::Value;

use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

use super::{forge, optifine};

pub const MC: &str = "1.8.9";
pub const FORGE_VERSION: &str = "11.15.1.2318";
const GITHUB_REPO: &str = "SantiJ10/Paraguacraft";
const RELEASE_TAG: &str = "pvp-client-1.0.0";
const PVP_CLIENT_JAR: &str = "ParaguacraftPvP-1.0.0.jar";
const PVP_CLIENT_SHA1: &str = "c0bbee47a923afa2b90af9eb4e08c417d519a19b";
const OPTIFINE_SHA1: &str = "d362d58a28f5373b141b9e426e8e160638bfafcd";

struct PvpMod {
    filename: &'static str,
    sha1: &'static str,
    optifine_type: Option<&'static str>,
    optifine_patch: Option<&'static str>,
}

const MODS: &[PvpMod] = &[
    PvpMod {
        filename: PVP_CLIENT_JAR,
        sha1: PVP_CLIENT_SHA1,
        optifine_type: None,
        optifine_patch: None,
    },
    PvpMod {
        filename: "OptiFine_1.8.9_HD_U_M5.jar",
        sha1: OPTIFINE_SHA1,
        optifine_type: Some("HD_U"),
        optifine_patch: Some("M5"),
    },
];

/// Solo 1.8.9: devuelve la version fija de Forge del cliente PvP.
pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    if mc != MC {
        return Ok(vec![]);
    }
    let forge_list = forge::versions(client, mc).await?;
    if forge_list.iter().any(|v| v == FORGE_VERSION) {
        return Ok(vec![FORGE_VERSION.into()]);
    }
    Ok(vec![FORGE_VERSION.into()])
}

/// Instala vanilla + Forge para el perfil PvP.
pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader_version: &str,
) -> AppResult<String> {
    if mc != MC {
        return Err(AppError::msg("Paraguacraft PvP solo esta disponible para Minecraft 1.8.9"));
    }
    let forge_ver = if loader_version.is_empty() {
        FORGE_VERSION
    } else {
        loader_version
    };
    forge::install(app, client, mc, forge_ver).await
}

fn cache_dir() -> PathBuf {
    paths::default_minecraft_dir()
        .join("Paraguacraft_cache")
        .join("pvp")
}

fn file_sha1(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(net::sha1_hex(&bytes))
}

/// URLs publicas del cliente PvP (todos los usuarios descargan desde GitHub).
fn pvp_client_urls(filename: &str) -> Vec<String> {
    vec![
        format!("https://raw.githubusercontent.com/{GITHUB_REPO}/main/bundled/pvp/{filename}"),
        format!("https://github.com/{GITHUB_REPO}/releases/download/{RELEASE_TAG}/{filename}"),
    ]
}

async fn github_release_asset_url(
    client: &reqwest::Client,
    filename: &str,
) -> Option<String> {
    for endpoint in [
        format!("https://api.github.com/repos/{GITHUB_REPO}/releases/tags/{RELEASE_TAG}"),
        format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest"),
    ] {
        let Ok(release): Result<Value, _> = net::fetch_json(client, &endpoint).await else {
            continue;
        };
        if let Some(assets) = release["assets"].as_array() {
            for asset in assets {
                if asset["name"].as_str() == Some(filename) {
                    return asset["browser_download_url"].as_str().map(String::from);
                }
            }
        }
    }
    None
}

async fn download_verified(
    client: &reqwest::Client,
    urls: Vec<String>,
    dest: &Path,
    sha1_expected: &str,
    label: &str,
) -> AppResult<()> {
    let tmp = dest.with_extension("part");
    for url in urls {
        if url.is_empty() {
            continue;
        }
        if net::download_one(client, &DownloadItem::new(url, tmp.clone()))
            .await
            .is_err()
        {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        if file_sha1(&tmp).as_deref() != Some(sha1_expected) {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if dest.exists() {
            let _ = std::fs::remove_file(dest);
        }
        std::fs::rename(&tmp, dest)
            .map_err(|e| AppError::msg(format!("No se pudo guardar {label}: {e}")))?;
        return Ok(());
    }
    Err(AppError::msg(format!("No se pudo descargar {label}")))
}

async fn ensure_pvp_client(client: &reqwest::Client, dest: &Path) -> AppResult<()> {
    let mut urls = pvp_client_urls(PVP_CLIENT_JAR);
    if let Some(u) = github_release_asset_url(client, PVP_CLIENT_JAR).await {
        urls.insert(0, u);
    }
    download_verified(client, urls, dest, PVP_CLIENT_SHA1, PVP_CLIENT_JAR)
        .await
        .map_err(|_| {
            AppError::msg(format!(
                "No se pudo descargar {PVP_CLIENT_JAR} desde GitHub. \
                 Verifica tu conexion o descargalo manualmente desde \
                 https://github.com/{GITHUB_REPO}/tree/main/bundled/pvp"
            ))
        })
}

async fn ensure_mod(
    client: &reqwest::Client,
    mod_def: &PvpMod,
    mods_dir: &Path,
) -> AppResult<()> {
    let dest = mods_dir.join(mod_def.filename);
    if file_sha1(&dest).as_deref() == Some(mod_def.sha1) {
        return Ok(());
    }

    let cache = cache_dir();
    std::fs::create_dir_all(&cache)?;
    let cache_path = cache.join(mod_def.filename);
    if file_sha1(&cache_path).as_deref() == Some(mod_def.sha1) {
        let _ = std::fs::copy(&cache_path, &dest);
        return Ok(());
    }

    if mod_def.filename.starts_with("ParaguacraftPvP") {
        ensure_pvp_client(client, &dest).await?;
        let _ = std::fs::copy(&dest, &cache_path);
        return Ok(());
    }

    if let (Some(of_type), Some(of_patch)) = (mod_def.optifine_type, mod_def.optifine_patch) {
        optifine::download_mod_jar_official(client, MC, of_type, of_patch, &dest)
            .await
            .map_err(|e| {
                AppError::msg(format!(
                    "No se pudo descargar OptiFine desde optifine.net: {e}. \
                     Descargalo manualmente en https://optifine.net/downloads (Mirror HD U M5 para 1.8.9)."
                ))
            })?;
        if file_sha1(&dest).as_deref() != Some(mod_def.sha1) {
            let _ = std::fs::remove_file(&dest);
            return Err(AppError::msg(
                "OptiFine descargado desde optifine.net no coincide con el hash esperado (HD U M5)",
            ));
        }
        let _ = std::fs::copy(&dest, &cache_path);
        return Ok(());
    }

    Ok(())
}

/// Descarga ParaguacraftPvP + OptiFine en la instancia (solo fuentes remotas).
pub async fn install_bundle(
    _app: &AppHandle,
    client: &reqwest::Client,
    instance_dir: &Path,
) -> AppResult<()> {
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    for mod_def in MODS {
        ensure_mod(client, mod_def, &mods_dir).await?;
    }
    Ok(())
}
