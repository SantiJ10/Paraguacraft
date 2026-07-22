//! Cliente **Paraguacraft PvP Modern** (Fabric 1.21.11) — loader dedicado + mod HUD propio.
//!
//! Separado de `fabric-iris` (solo optimización). Espeja `loaders/pvp.rs` en 1.8.9.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

use super::fabric_iris;

pub const MC: &str = "1.21.11";
const GITHUB_REPO: &str = "SantiJ10/Paraguacraft";
const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/clientes/paraguacraft-pvp-modern/manifest.json";

const MANIFEST_MIRROR_URLS: &[&str] = &[
    "https://github.com/SantiJ10/Paraguacraft/raw/main/clientes/paraguacraft-pvp-modern/manifest.json",
    MANIFEST_URL,
    "https://cdn.jsdelivr.net/gh/SantiJ10/Paraguacraft@main/clientes/paraguacraft-pvp-modern/manifest.json",
];

const FALLBACK_CLIENT_VERSION: &str = "0.8.4";
const FALLBACK_RELEASE_TAG: &str = "pvp-modern-0.8.4";

const FALLBACK_MODS: &[(&str, &str)] = &[(
    "ParaguacraftPvP-Modern-0.8.4.jar",
    "d2f516eae7bd548118d71438b267637139faee2d",
)];

#[derive(Debug, Clone, Deserialize)]
struct ModernManifest {
    #[serde(default = "default_client_version")]
    client_version: String,
    #[serde(default)]
    release_tag: Option<String>,
    #[serde(default)]
    mods: Vec<ModernModEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModernModEntry {
    filename: String,
    sha1: String,
}

fn default_client_version() -> String {
    FALLBACK_CLIENT_VERSION.into()
}

impl Default for ModernManifest {
    fn default() -> Self {
        Self {
            client_version: FALLBACK_CLIENT_VERSION.into(),
            release_tag: Some(FALLBACK_RELEASE_TAG.into()),
            mods: FALLBACK_MODS
                .iter()
                .map(|(f, s)| ModernModEntry {
                    filename: (*f).into(),
                    sha1: (*s).into(),
                })
                .collect(),
        }
    }
}

impl ModernManifest {
    fn release_tag(&self) -> String {
        self.release_tag.clone().unwrap_or_else(|| {
            format!("pvp-modern-{}", self.client_version.trim())
        })
    }
}

async fn fetch_manifest_with_source(
    client: &reqwest::Client,
    app: Option<&AppHandle>,
) -> (ModernManifest, &'static str) {
    for url in MANIFEST_MIRROR_URLS {
        if let Ok(m) = net::fetch_json::<ModernManifest>(client, url).await {
            if !m.mods.is_empty() {
                return (m, "remote");
            }
        }
    }
    if let Some(app) = app {
        if let Some(m) = read_bundled_manifest(app) {
            if !m.mods.is_empty() {
                return (m, "bundled");
            }
        }
    }
    (ModernManifest::default(), "fallback")
}

async fn fetch_manifest(client: &reqwest::Client, app: Option<&AppHandle>) -> ModernManifest {
    fetch_manifest_with_source(client, app).await.0
}

fn version_from_modern_jar(filename: &str) -> Option<String> {
    let lower = filename.to_lowercase();
    const PREFIX: &str = "paraguacraftpvp-modern-";
    if !lower.starts_with(PREFIX) || !lower.ends_with(".jar") {
        return None;
    }
    let ver = &filename[PREFIX.len()..filename.len().saturating_sub(4)];
    if ver.is_empty() {
        None
    } else {
        Some(ver.to_string())
    }
}

fn detect_installed_client(
    app: &AppHandle,
    instance_dir: Option<&Path>,
    manifest: &ModernManifest,
) -> (Option<String>, Option<String>, bool) {
    let Some(dir) = instance_dir else {
        return (None, None, false);
    };
    let mods_dir = dir.join("mods");
    let main_mod = manifest.mods.iter().find(|m| {
        m.filename
            .to_lowercase()
            .starts_with("paraguacraftpvp-modern")
    });

    if let Some(entry) = main_mod {
        let dest = mods_dir.join(&entry.filename);
        if sha_matches(&dest, &entry.sha1) {
            return (
                Some(entry.filename.clone()),
                Some(manifest.client_version.clone()),
                true,
            );
        }
    }

    let Ok(rd) = std::fs::read_dir(&mods_dir) else {
        return (None, None, false);
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.to_lowercase().starts_with("paraguacraftpvp-modern") {
            continue;
        }
        let path = e.path();
        let up_to_date = main_mod.is_some_and(|exp| {
            exp.filename == name && sha_matches(&path, &exp.sha1)
        });
        return (Some(name.clone()), version_from_modern_jar(&name), up_to_date);
    }
    (None, None, false)
}

/// Estado del cliente PvP 1.21.11 (manifest remoto vs JAR instalado).
pub async fn client_status(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_dir: Option<&Path>,
) -> crate::models::PvpClientStatus {
    let (manifest, source) = fetch_manifest_with_source(client, Some(app)).await;
    let main_mod = manifest.mods.iter().find(|m| {
        m.filename
            .to_lowercase()
            .starts_with("paraguacraftpvp-modern")
    });
    let remote_filename = main_mod.map(|m| m.filename.clone()).unwrap_or_default();
    let (installed_filename, installed_version, up_to_date) =
        detect_installed_client(app, instance_dir, &manifest);

    crate::models::PvpClientStatus {
        remote_version: manifest.client_version.clone(),
        remote_filename,
        installed_version,
        installed_filename,
        up_to_date,
        auto_updates_on_launch: true,
        manifest_source: source.into(),
        manifest_url: MANIFEST_URL.into(),
    }
}

fn read_bundled_manifest(app: &AppHandle) -> Option<ModernManifest> {
    for dir in local_bundled_dirs(app) {
        let path = dir.join("manifest.json");
        if !path.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&path).ok()?;
        let text = text.trim_start_matches('\u{FEFF}');
        let m: ModernManifest = serde_json::from_str(text).ok()?;
        if !m.mods.is_empty() {
            return Some(m);
        }
    }
    None
}

fn cache_dir() -> PathBuf {
    paths::default_minecraft_dir()
        .join("Paraguacraft_cache")
        .join("pvp-modern")
}

fn file_sha1(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(net::sha1_hex(&bytes))
}

fn resource_roots(app: &AppHandle) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rel in ["bundled/pvp-modern", "resources/bundled/pvp-modern"] {
        if let Ok(p) = app.path().resolve(rel, BaseDirectory::Resource) {
            roots.push(p);
        }
    }
    if let Ok(dir) = app.path().resource_dir() {
        roots.push(dir.join("bundled").join("pvp-modern"));
        roots.push(dir.join("resources").join("bundled").join("pvp-modern"));
    }
    roots
}

fn local_bundled_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs = resource_roots(app);
    if let Ok(custom) = std::env::var("PARAGUACRAFT_BUNDLED_PVP_MODERN") {
        dirs.push(PathBuf::from(custom));
    }
    dirs.push(paths::data_dir().join("bundled").join("pvp-modern"));
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        for _ in 0..8 {
            let Some(mut d) = dir else { break };
            dirs.push(d.join("bundled").join("pvp-modern"));
            if !d.pop() {
                break;
            }
            dir = Some(d);
        }
    }
    dirs
}

fn find_local_file(app: &AppHandle, filename: &str) -> Option<PathBuf> {
    for dir in local_bundled_dirs(app) {
        let p = dir.join(filename);
        if p.is_file() && p.metadata().map(|m| m.len()).unwrap_or(0) > 1024 {
            return Some(p);
        }
    }
    None
}

fn sha_matches(path: &Path, expected: &str) -> bool {
    if expected.chars().all(|c| c == '0') {
        return path.is_file();
    }
    let Some(got) = file_sha1(path) else {
        return false;
    };
    got.eq_ignore_ascii_case(expected)
}

fn copy_to_dest(src: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        let _ = std::fs::remove_file(dest);
    }
    std::fs::copy(src, dest)
        .map_err(|e| AppError::msg(format!("No se pudo copiar {}: {e}", dest.display())))?;
    Ok(())
}

fn sha_matches_or_fallback(path: &Path, expected: &str, filename: &str) -> bool {
    if sha_matches(path, expected) {
        return true;
    }
    if let Some((_, fb)) = FALLBACK_MODS.iter().find(|(f, _)| *f == filename) {
        if let Some(got) = file_sha1(path) {
            return got.eq_ignore_ascii_case(fb);
        }
    }
    false
}

fn try_local_bundled(
    app: &AppHandle,
    filename: &str,
    sha_expected: &str,
    dest: &Path,
) -> AppResult<bool> {
    let Some(src) = find_local_file(app, filename) else {
        return Ok(false);
    };
    if !sha_matches_or_fallback(&src, sha_expected, filename) {
        return Ok(false);
    }
    copy_to_dest(&src, dest)?;
    Ok(true)
}

fn remote_urls(filename: &str) -> Vec<String> {
    vec![
        format!("https://raw.githubusercontent.com/{GITHUB_REPO}/main/bundled/pvp-modern/{filename}"),
        format!("https://github.com/{GITHUB_REPO}/raw/main/bundled/pvp-modern/{filename}"),
        format!(
            "https://raw.githubusercontent.com/{GITHUB_REPO}/main/clientes/paraguacraft-pvp-modern/{filename}"
        ),
        format!(
            "https://github.com/{GITHUB_REPO}/raw/main/clientes/paraguacraft-pvp-modern/{filename}"
        ),
    ]
}

async fn download_verified(
    client: &reqwest::Client,
    urls: Vec<String>,
    dest: &Path,
    sha_expected: &str,
    filename: &str,
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
        if !sha_matches(&tmp, sha_expected) {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        if dest.exists() {
            let _ = std::fs::remove_file(dest);
        }
        std::fs::rename(&tmp, dest)
            .map_err(|e| AppError::msg(format!("No se pudo guardar {filename}: {e}")))?;
        return Ok(());
    }
    Err(AppError::msg(format!("No se pudo descargar {filename}")))
}

fn prune_stale_mods(mods_dir: &Path, keep: &[String]) {
    let Ok(rd) = std::fs::read_dir(mods_dir) else {
        return;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_lowercase();
        if !name.starts_with("paraguacraftpvp-modern") || !name.ends_with(".jar") {
            continue;
        }
        let keep_this = keep.iter().any(|k| k.to_lowercase() == name);
        if !keep_this {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

async fn ensure_mod(
    app: &AppHandle,
    client: &reqwest::Client,
    filename: &str,
    sha_expected: &str,
    _release_tag: &str,
    mods_dir: &Path,
) -> AppResult<()> {
    let dest = mods_dir.join(filename);
    if sha_matches(&dest, sha_expected) {
        return Ok(());
    }

    let cache = cache_dir();
    std::fs::create_dir_all(&cache)?;
    let cache_path = cache.join(filename);
    if sha_matches(&cache_path, sha_expected) {
        copy_to_dest(&cache_path, &dest)?;
        return Ok(());
    }

    if try_local_bundled(app, filename, sha_expected, &dest)? {
        let _ = std::fs::copy(&dest, &cache_path);
        return Ok(());
    }

    if let Err(e) = download_verified(
        client,
        remote_urls(filename),
        &dest,
        sha_expected,
        filename,
    )
    .await
    {
        if try_local_bundled(app, filename, sha_expected, &dest)? {
            let _ = std::fs::copy(&dest, &cache_path);
            return Ok(());
        }
        return Err(AppError::msg(format!(
            "No se pudo obtener {filename} (Paraguacraft PvP 1.21.11). Actualizá el launcher o usá Reparar instancia. ({e})"
        )));
    }
    let _ = std::fs::copy(&dest, &cache_path);
    Ok(())
}

/// Instala el mod HUD Paraguacraft PvP Modern segun manifest remoto.
pub async fn install_bundle(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_dir: &Path,
) -> AppResult<()> {
    let manifest = fetch_manifest(client, Some(app)).await;
    let release_tag = manifest.release_tag();
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;

    let filenames: Vec<String> = manifest.mods.iter().map(|m| m.filename.clone()).collect();
    prune_stale_mods(&mods_dir, &filenames);

    for entry in &manifest.mods {
        ensure_mod(
            app,
            client,
            &entry.filename,
            &entry.sha1,
            &release_tag,
            &mods_dir,
        )
        .await?;
    }
    Ok(())
}

/// Perfil Fabric (base para Iris + cliente PvP modern).
pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    if mc != MC {
        return Ok(vec![]);
    }
    fabric_iris::versions(client, mc).await
}

pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader_version: &str,
) -> AppResult<String> {
    if mc != MC {
        return Err(AppError::msg(
            "Paraguacraft PvP 1.21.11 solo esta disponible para Minecraft 1.21.11",
        ));
    }
    fabric_iris::install_fabric_profile(app, client, mc, loader_version).await
}

/// Iris bundle + JAR ParaguacraftPvP-Modern (solo instancias `paraguacraft-pvp-modern`).
pub async fn sync_instance(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    instance_dir: &Path,
) -> AppResult<()> {
    if mc != MC {
        return Err(AppError::msg("Versión MC incompatible con Paraguacraft PvP 1.21.11"));
    }
    fabric_iris::install_bundle(app, client, mc, instance_dir).await?;
    install_bundle(app, client, instance_dir).await
}
