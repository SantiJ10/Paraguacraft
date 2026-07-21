//! Resource pack oficial Paraguacraft PvP — descarga y activación automática al lanzar
//! (mismo flujo que `ParaguacraftBrandPack`).

use std::path::{Path, PathBuf};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::core::instances::resource_packs;
use crate::core::loaders;
use crate::core::modern_packs;
use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

pub const PACK_189: &str = "paraguacraft-pvp-189.zip";
pub const PACK_189_SHA1: &str = "79599f36fe957d6443c9388c452bc93d67291bdd";
pub const PACK_MODERN: &str = "paraguacraft-pvp-modern.zip";

const RELEASE_189: &str = "pvp-packs-1.0";
const BASE_URL_189: &str =
    "https://github.com/SantiJ10/Paraguacraft/releases/download/pvp-packs-1.0";

fn bundled_roots_189(app: &AppHandle) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rel in ["bundled/pvp", "resources/bundled/pvp"] {
        if let Ok(p) = app.path().resolve(rel, BaseDirectory::Resource) {
            roots.push(p);
        }
    }
    if let Ok(dir) = app.path().resource_dir() {
        roots.push(dir.join("bundled").join("pvp"));
    }
    roots.push(paths::data_dir().join("bundled").join("pvp"));
    roots
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
    for root in bundled_roots_189(app) {
        let p = root.join("resourcepacks").join(filename);
        if p.is_file() && p.metadata().map(|m| m.len()).unwrap_or(0) > 1024 {
            return Some(p);
        }
    }
    // Desarrollo: zip generado en el repo.
    if let Ok(cwd) = std::env::current_dir() {
        for base in [
            cwd.join("clientes/paraguacraft-pvp/packs"),
            cwd.join("../clientes/paraguacraft-pvp/packs"),
        ] {
            let p = base.join(filename);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn pack_urls_189(filename: &str) -> Vec<String> {
    vec![
        format!("{BASE_URL_189}/{filename}"),
        format!(
            "https://github.com/SantiJ10/Paraguacraft/releases/download/{RELEASE_189}/{filename}"
        ),
    ]
}

async fn ensure_primary_189(
    app: &AppHandle,
    client: &reqwest::Client,
    dest_dir: &Path,
) -> AppResult<()> {
    let dest = dest_dir.join(PACK_189);
    if sha_matches(&dest, PACK_189_SHA1) {
        return Ok(());
    }

    if let Some(src) = find_local_pack(app, PACK_189) {
        if sha_matches(&src, PACK_189_SHA1) {
            std::fs::create_dir_all(dest_dir)?;
            std::fs::copy(&src, &dest)?;
            return Ok(());
        }
    }

    let tmp = dest.with_extension("part");
    for url in pack_urls_189(PACK_189) {
        if net::download_one(client, &DownloadItem::new(url, tmp.clone()))
            .await
            .is_err()
        {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        if !sha_matches(&tmp, PACK_189_SHA1) {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        std::fs::create_dir_all(dest_dir)?;
        if dest.exists() {
            let _ = std::fs::remove_file(&dest);
        }
        std::fs::rename(&tmp, &dest)?;
        return Ok(());
    }

    if let Some(src) = find_local_pack(app, PACK_189) {
        std::fs::create_dir_all(dest_dir)?;
        std::fs::copy(&src, &dest)?;
        return Ok(());
    }

    Err(AppError::msg(format!(
        "No se pudo obtener el pack {PACK_189}"
    )))
}

fn enable_primary_pack(game_dir: &Path, mc_version: &str, pack_name: &str) -> AppResult<()> {
    resource_packs::set_enabled(game_dir, mc_version, pack_name, false, true)
}

/// Descarga el pack oficial (si falta) y lo activa en `options.txt` antes del lanzamiento.
pub async fn prepare_launch(
    app: &AppHandle,
    client: &reqwest::Client,
    game_dir: &Path,
    loader: &str,
    mc_version: &str,
) -> AppResult<()> {
    let norm = loaders::normalize(loader);
    let packs_dir = game_dir.join("resourcepacks");
    std::fs::create_dir_all(&packs_dir)?;

    match norm.as_str() {
        "paraguacraft-pvp-modern" => {
            let _ = modern_packs::sync_instance_packs(app, client, game_dir).await?;
            enable_primary_pack(game_dir, mc_version, PACK_MODERN)?;
        }
        "paraguacraft-pvp" => {
            ensure_primary_189(app, client, &packs_dir).await?;
            enable_primary_pack(game_dir, mc_version, PACK_189)?;
        }
        _ => {}
    }
    Ok(())
}
