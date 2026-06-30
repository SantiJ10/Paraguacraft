//! Preset **Paraguacraft PvP** (solo Minecraft 1.8.9).
//!
//! ## Actualizar solo el cliente (sin recompilar el launcher)
//! La versión publicada vive en el manifest remoto:
//! `clientes/paraguacraft-pvp/manifest.json` + JAR en `bundled/pvp/`.
//! En cada lanzamiento se llama a `install_bundle`, que descarga el JAR si el SHA-1
//! no coincide. **No hace falta** subir un launcher nuevo para cada cliente.
//!
//! Las constantes `FALLBACK_*` son solo respaldo offline (sin internet o manifest caído).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::PvpClientStatus;

use super::forge;

pub const MC: &str = "1.8.9";
pub const FORGE_VERSION: &str = "11.15.1.2318";
const GITHUB_REPO: &str = "SantiJ10/Paraguacraft";
const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/clientes/paraguacraft-pvp/manifest.json";

const FALLBACK_CLIENT_VERSION: &str = "2.1.21";
const FALLBACK_RELEASE_TAG: &str = "pvp-client-2.1.21";

const FALLBACK_MODS: &[(&str, &str)] = &[
    (
        "ParaguacraftPvP-2.1.21.jar",
        "728a474464e80c2902dba6441663814c82880a27",
    ),
    (
        "Hytils-Reborn-1.8.9-forge-1.7.5.jar",
        "f34462f9072bb23b0c6dd313d38e80a8d6a3eda1",
    ),
    (
        "OptiFine_1.8.9_HD_U_M5.jar",
        "d362d58a28f5373b141b9e426e8e160638bfafcd",
    ),
];

#[derive(Debug, Clone, Deserialize)]
struct PvpManifest {
    #[serde(default = "default_client_version")]
    client_version: String,
    #[serde(default)]
    release_tag: Option<String>,
    #[serde(default)]
    mods: Vec<PvpModEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct PvpModEntry {
    filename: String,
    sha1: String,
}

fn default_client_version() -> String {
    FALLBACK_CLIENT_VERSION.into()
}

impl Default for PvpManifest {
    fn default() -> Self {
        Self {
            client_version: FALLBACK_CLIENT_VERSION.into(),
            release_tag: Some(FALLBACK_RELEASE_TAG.into()),
            mods: FALLBACK_MODS
                .iter()
                .map(|(f, s)| PvpModEntry {
                    filename: (*f).into(),
                    sha1: (*s).into(),
                })
                .collect(),
        }
    }
}

impl PvpManifest {
    fn release_tag(&self) -> String {
        self.release_tag.clone().unwrap_or_else(|| {
            format!("pvp-client-{}", self.client_version.trim())
        })
    }
}

async fn fetch_manifest_with_source(client: &reqwest::Client) -> (PvpManifest, &'static str) {
    match net::fetch_json::<PvpManifest>(client, MANIFEST_URL).await {
        Ok(m) if !m.mods.is_empty() => (m, "remote"),
        _ => (PvpManifest::default(), "fallback"),
    }
}

async fn fetch_manifest(client: &reqwest::Client) -> PvpManifest {
    fetch_manifest_with_source(client).await.0
}

fn version_from_pvp_jar(filename: &str) -> Option<String> {
    let lower = filename.to_lowercase();
    if !lower.starts_with("paraguacraftpvp-") || !lower.ends_with(".jar") {
        return None;
    }
    let ver = &filename[16..filename.len().saturating_sub(4)];
    if ver.is_empty() {
        None
    } else {
        Some(ver.to_string())
    }
}

fn detect_installed_client(
    app: &AppHandle,
    instance_dir: Option<&Path>,
    manifest: &PvpManifest,
) -> (Option<String>, Option<String>, bool) {
    let Some(dir) = instance_dir else {
        return (None, None, false);
    };
    let mods_dir = dir.join("mods");
    let main_mod = manifest
        .mods
        .iter()
        .find(|m| m.filename.to_lowercase().starts_with("paraguacraftpvp"));

    if let Some(entry) = main_mod {
        let dest = mods_dir.join(&entry.filename);
        let sha = resolve_sha(app, &entry.filename, &entry.sha1);
        if sha_matches(&dest, &sha, &entry.filename) {
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
        if !name.to_lowercase().starts_with("paraguacraftpvp") {
            continue;
        }
        let path = e.path();
        let up_to_date = main_mod.is_some_and(|exp| {
            exp.filename == name && sha_matches(&path, &resolve_sha(app, &exp.filename, &exp.sha1), &exp.filename)
        });
        return (Some(name.clone()), version_from_pvp_jar(&name), up_to_date);
    }
    (None, None, false)
}

/// Estado del cliente PvP (remoto vs instalado en una instancia).
pub async fn client_status(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_dir: Option<&Path>,
) -> PvpClientStatus {
    let (manifest, source) = fetch_manifest_with_source(client).await;
    let main_mod = manifest
        .mods
        .iter()
        .find(|m| m.filename.to_lowercase().starts_with("paraguacraftpvp"));
    let remote_filename = main_mod
        .map(|m| m.filename.clone())
        .unwrap_or_default();
    let (installed_filename, installed_version, up_to_date) =
        detect_installed_client(app, instance_dir, &manifest);

    PvpClientStatus {
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

/// Versión publicada del cliente PvP (manifest remoto).
pub async fn remote_client_version(client: &reqwest::Client) -> String {
    fetch_manifest(client).await.client_version
}

pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    if mc != MC {
        return Ok(vec![]);
    }
    let _ = forge::versions(client, mc).await?;
    Ok(vec![FORGE_VERSION.into()])
}

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

fn resource_roots(app: &AppHandle) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rel in ["bundled/pvp", "resources/bundled/pvp"] {
        if let Ok(p) = app.path().resolve(rel, BaseDirectory::Resource) {
            roots.push(p);
        }
    }
    if let Ok(dir) = app.path().resource_dir() {
        roots.push(dir.join("bundled").join("pvp"));
        roots.push(dir.join("resources").join("bundled").join("pvp"));
    }
    roots
}

fn local_bundled_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs = resource_roots(app);
    if let Ok(custom) = std::env::var("PARAGUACRAFT_BUNDLED_PVP") {
        dirs.push(PathBuf::from(custom));
    }
    dirs.push(paths::data_dir().join("bundled").join("pvp"));
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        for _ in 0..8 {
            let Some(mut d) = dir else { break };
            dirs.push(d.join("bundled").join("pvp"));
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
        if p.is_file() && p.metadata().map(|m| m.len()).unwrap_or(0) > 10_000 {
            return Some(p);
        }
    }
    None
}

fn resolve_sha(app: &AppHandle, filename: &str, remote_sha: &str) -> String {
    // El manifest remoto define la versión publicada; el embebido solo sirve como fuente de copia.
    if !remote_sha.is_empty() {
        return remote_sha.to_string();
    }
    if let Some((_, fb)) = FALLBACK_MODS.iter().find(|(f, _)| *f == filename) {
        return (*fb).to_string();
    }
    if let Some(p) = find_local_file(app, filename) {
        if let Some(s) = file_sha1(&p) {
            return s;
        }
    }
    remote_sha.to_string()
}

fn is_pvp_client_jar(filename: &str) -> bool {
    filename.to_lowercase().starts_with("paraguacraftpvp-")
}

fn has_any_pvp_client_jar(mods_dir: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(mods_dir) else {
        return false;
    };
    rd.flatten().any(|e| {
        let name = e.file_name().to_string_lossy().to_lowercase();
        name.starts_with("paraguacraftpvp-") && name.ends_with(".jar")
    })
}

fn sha_matches(path: &Path, expected: &str, _filename: &str) -> bool {
    let Some(got) = file_sha1(path) else {
        return false;
    };
    got.eq_ignore_ascii_case(expected)
}

/// Igual que `sha_matches` pero acepta el SHA embebido de respaldo (solo para copias locales).
fn sha_matches_or_fallback(path: &Path, expected: &str, filename: &str) -> bool {
    if sha_matches(path, expected, filename) {
        return true;
    }
    if let Some((_, fb)) = FALLBACK_MODS.iter().find(|(f, _)| *f == filename) {
        if let Some(got) = file_sha1(path) {
            return got.eq_ignore_ascii_case(fb);
        }
    }
    false
}

fn copy_to_dest(src: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        let _ = std::fs::remove_file(dest);
    }
    std::fs::copy(src, dest).map_err(|e| AppError::msg(format!("No se pudo copiar {}: {e}", dest.display())))?;
    Ok(())
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

fn remote_urls(filename: &str, _release_tag: &str) -> Vec<String> {
    // El cliente PvP vive en la carpeta del repo (bundled/pvp), NO en GitHub Releases.
    // Solo el instalador del launcher se publica como release.
    vec![
        format!("https://raw.githubusercontent.com/{GITHUB_REPO}/main/bundled/pvp/{filename}"),
        format!(
            "https://raw.githubusercontent.com/{GITHUB_REPO}/main/clientes/paraguacraft-pvp/{filename}"
        ),
    ]
}

async fn download_verified(
    client: &reqwest::Client,
    urls: Vec<String>,
    dest: &Path,
    sha_expected: &str,
    filename: &str,
    _release_tag: &str,
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
        if !sha_matches(&tmp, sha_expected, filename) {
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

/// Elimina JARs PvP viejos cuando cambia el nombre (ej. 1.0.0 → 2.0.0).
fn prune_stale_pvp_mods(mods_dir: &Path, keep: &[String]) {
    let Ok(rd) = std::fs::read_dir(mods_dir) else {
        return;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_lowercase();
        if !name.ends_with(".jar") {
            continue;
        }
        let is_pvp = name.starts_with("paraguacraftpvp")
            || name.starts_with("optifine_1.8.9")
            || name.starts_with("hytils")
            || name.starts_with("patcher")
            || name.starts_with("essential");
        if !is_pvp {
            continue;
        }
        let keep_this = keep.iter().any(|k| k.to_lowercase() == name);
        if !keep_this {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

/// Pre-configura OneConfig/Hytils en la primera instalación:
/// - OneConfig sin tecla (RShift queda para el Mod Menu de Paraguacraft).
/// - Hytils: solo camas coloreadas; resto desactivado.
const ONECONFIG_SEED_MARKER: &str = ".paraguacraft-hytils-v1";

fn oneconfig_defaults_dir(app: &AppHandle) -> Option<PathBuf> {
    for dir in local_bundled_dirs(app) {
        let p = dir.join("defaults").join("oneconfig");
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

fn copy_template_file(src: &Path, dest: &Path, only_if_missing: bool) -> std::io::Result<()> {
    if only_if_missing && dest.is_file() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, dest)?;
    Ok(())
}

fn seed_oneconfig_defaults(app: &AppHandle, instance_dir: &Path) {
    let oneconfig_dir = instance_dir.join("OneConfig");
    let marker = oneconfig_dir.join(ONECONFIG_SEED_MARKER);
    if marker.is_file() {
        return;
    }
    let Some(templates) = oneconfig_defaults_dir(app) else {
        return;
    };
    let _ = std::fs::create_dir_all(&oneconfig_dir);
    let files: [(&str, bool); 3] = [
        ("OneConfig.json", true),
        ("Preferences.json", false),
        ("profiles/Default Profile/hytilsreborn.json", false),
    ];
    for (rel, only_if_missing) in files {
        let src = templates.join(rel);
        if !src.is_file() {
            continue;
        }
        let dest = oneconfig_dir.join(rel);
        let _ = copy_template_file(&src, &dest, only_if_missing);
    }
    let _ = std::fs::File::create(&marker);
}

/// Borra restos de Essential/Patcher: carpetas de datos y configs que quedaron
/// de versiones anteriores del cliente (Essential pedía login y rompía ajustes).
fn cleanup_essential_leftovers(instance_dir: &Path, mods_dir: &Path) {
    // Carpetas de datos en la raíz de la instancia.
    for dir in ["essential", "ModCoreOSS"] {
        let path = instance_dir.join(dir);
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
    // Config de Patcher en la carpeta config/.
    for cfg in ["patcher.toml", "patcher", "essential"] {
        let path = instance_dir.join("config").join(cfg);
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(&path);
        } else if path.is_file() {
            let _ = std::fs::remove_file(&path);
        }
    }
    // Essential también deja una carpeta "essential" dentro de mods/.
    let mods_essential = mods_dir.join("essential");
    if mods_essential.is_dir() {
        let _ = std::fs::remove_dir_all(&mods_essential);
    }
}

async fn ensure_mod(
    app: &AppHandle,
    client: &reqwest::Client,
    filename: &str,
    sha_expected: &str,
    release_tag: &str,
    mods_dir: &Path,
    offline_lenient: bool,
) -> AppResult<()> {
    let dest = mods_dir.join(filename);
    if sha_matches(&dest, sha_expected, filename) {
        return Ok(());
    }

    let cache = cache_dir();
    std::fs::create_dir_all(&cache)?;
    let cache_path = cache.join(filename);
    if sha_matches(&cache_path, sha_expected, filename) {
        copy_to_dest(&cache_path, &dest)?;
        return Ok(());
    }

    if try_local_bundled(app, filename, sha_expected, &dest)? {
        let _ = std::fs::copy(&dest, &cache_path);
        return Ok(());
    }

    if let Err(e) = download_verified(
        client,
        remote_urls(filename, release_tag),
        &dest,
        sha_expected,
        filename,
        release_tag,
    )
    .await
    {
        if offline_lenient && is_pvp_client_jar(filename) && has_any_pvp_client_jar(mods_dir) {
            return Ok(());
        }
        return Err(AppError::msg(format!(
            "No se pudo obtener {filename} (cliente PvP). Actualizá el launcher o usá Reparar instancia. ({e})"
        )));
    }
    let _ = std::fs::copy(&dest, &cache_path);
    Ok(())
}

/// Sincroniza mods PvP según el manifest remoto (versión + SHA-1).
pub async fn install_bundle(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_dir: &Path,
) -> AppResult<()> {
    let (manifest, source) = fetch_manifest_with_source(client).await;
    let offline_lenient = source == "fallback";
    let release_tag = manifest.release_tag();
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;

    let filenames: Vec<String> = manifest.mods.iter().map(|m| m.filename.clone()).collect();
    prune_stale_pvp_mods(&mods_dir, &filenames);
    cleanup_essential_leftovers(instance_dir, &mods_dir);
    seed_oneconfig_defaults(app, instance_dir);

    for entry in &manifest.mods {
        let sha = resolve_sha(app, &entry.filename, &entry.sha1);
        ensure_mod(
            app,
            client,
            &entry.filename,
            &sha,
            &release_tag,
            &mods_dir,
            offline_lenient,
        )
        .await?;
    }
    Ok(())
}
