//! Catálogo, instalación y cambio de versiones Bedrock (releases públicas).

use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::DownloadProgress;

#[cfg(windows)]
use super::fe3;

const VERSION_DB_URL: &str = "https://mrarm.io/r/w10-vdb";
const MCAPPX_URL: &str = "https://data.mcappx.com/v2/bedrock.json";
const MCAPPX_UA: &str = "mcappx_developer";
const CACHE_TTL_SECS: u64 = 24 * 60 * 60;
const META_FILE: &str = "bedrock-version.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockVersion {
    pub version: String,
    pub update_identity: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub installable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockInstalledVersion {
    pub version: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub path: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockVersionStatus {
    pub store_installed: bool,
    pub managed_active: bool,
    pub active_version: Option<String>,
    pub developer_mode: bool,
    pub conflict_store: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionMeta {
    version: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionDbCache {
    time: u64,
    versions: Vec<BedrockVersion>,
}

pub fn versions_root() -> PathBuf {
    let dir = paths::data_dir().join("bedrock-versions");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn cache_path() -> PathBuf {
    versions_root().join("version-db.json")
}

pub fn sanitize_version_dir(version: &str) -> String {
    version
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn version_dir(version: &str) -> PathBuf {
    versions_root().join(sanitize_version_dir(version))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn type_from_raw(raw: i64) -> Option<&'static str> {
    match raw {
        0 => Some("release"),
        1 => Some("beta"),
        2 => Some("preview"),
        _ => None,
    }
}

fn type_from_str(raw: &str) -> &'static str {
    match raw.to_ascii_lowercase().as_str() {
        "beta" => "beta",
        "preview" => "preview",
        _ => "release",
    }
}

fn make_version(version: String, update_identity: String, kind: &str) -> BedrockVersion {
    BedrockVersion {
        installable: kind == "release" && !update_identity.is_empty(),
        version,
        update_identity,
        kind: kind.to_string(),
    }
}

fn parse_legacy_db(data: &[serde_json::Value]) -> Vec<BedrockVersion> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for entry in data.iter().rev() {
        let arr = match entry.as_array() {
            Some(a) if a.len() >= 3 => a,
            _ => continue,
        };
        let version = arr[0].as_str().unwrap_or("").trim().to_string();
        let identity = if arr[1].is_string() {
            arr[1].as_str().unwrap_or("").trim().to_string()
        } else {
            arr[1].to_string()
        };
        let kind = match arr[2].as_i64().and_then(type_from_raw) {
            Some(k) => k,
            None => continue,
        };
        if version.is_empty() || identity.is_empty() {
            continue;
        }
        let key = format!("{version}:{kind}");
        if !seen.insert(key) {
            continue;
        }
        out.push(make_version(version, identity, kind));
    }
    out
}

fn pick_mcappx_metadata(build: &serde_json::Value) -> Option<String> {
    let variations = build.get("Variations")?.as_array()?;
    let mut ordered: Vec<&serde_json::Value> = Vec::new();
    for want in ["x64", "neutral"] {
        for v in variations {
            let arch = v
                .get("Arch")
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if arch == want {
                ordered.push(v);
            }
        }
    }
    for v in variations {
        if !ordered.iter().any(|x| std::ptr::eq(*x, v)) {
            ordered.push(v);
        }
    }
    for variation in ordered {
        let Some(meta) = variation.get("MetaData").and_then(|m| m.as_array()) else {
            continue;
        };
        for item in meta.iter().rev() {
            if let Some(s) = item.as_str() {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}

fn parse_mcappx_db(data: &serde_json::Value) -> AppResult<Vec<BedrockVersion>> {
    let obj = data.as_object().ok_or_else(|| AppError::msg("Catálogo Bedrock inválido"))?;
    let versions_obj = obj
        .get("From_mcappx_com")
        .or_else(|| obj.get("From_mcappx.com"))
        .and_then(|v| v.as_object())
        .or_else(|| {
            obj.iter().find_map(|(k, v)| {
                if k == "CreationTime" {
                    None
                } else {
                    v.as_object()
                }
            })
        })
        .ok_or_else(|| AppError::msg("Catálogo MCAPPX inválido"))?;

    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (key, value) in versions_obj {
        let Some(build) = value.as_object() else { continue };
        let version = build
            .get("ID")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(key)
            .to_string();
        let Some(identity) = pick_mcappx_metadata(value) else {
            continue;
        };
        let kind = type_from_str(
            build
                .get("Type")
                .and_then(|v| v.as_str())
                .unwrap_or("release"),
        );
        let k = format!("{version}:{kind}");
        if !seen.insert(k) {
            continue;
        }
        out.push(make_version(version, identity, kind));
    }
    sort_versions(&mut out);
    Ok(out)
}

fn sort_versions(versions: &mut [BedrockVersion]) {
    versions.sort_by(|a, b| {
        cmp_version(&b.version, &a.version).then_with(|| type_weight(&a.kind).cmp(&type_weight(&b.kind)))
    });
}

fn type_weight(kind: &str) -> u8 {
    match kind {
        "release" => 0,
        "beta" => 1,
        _ => 2,
    }
}

fn cmp_version(a: &str, b: &str) -> std::cmp::Ordering {
    let pa: Vec<u64> = a.split('.').filter_map(|p| p.parse().ok()).collect();
    let pb: Vec<u64> = b.split('.').filter_map(|p| p.parse().ok()).collect();
    let n = pa.len().max(pb.len());
    for i in 0..n {
        let x = pa.get(i).copied().unwrap_or(0);
        let y = pb.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            std::cmp::Ordering::Equal => {}
            other => return other,
        }
    }
    a.cmp(b)
}

pub fn parse_version_db(text: &str) -> AppResult<Vec<BedrockVersion>> {
    let data: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{FEFF}'))?;
    if let Some(arr) = data.as_array() {
        let mut versions = parse_legacy_db(arr);
        sort_versions(&mut versions);
        return Ok(versions);
    }
    if data.is_object() {
        return parse_mcappx_db(&data);
    }
    Err(AppError::msg("Formato de catálogo Bedrock desconocido"))
}

async fn fetch_text(client: &reqwest::Client, url: &str, ua: Option<&str>) -> AppResult<String> {
    let mut req = client.get(url);
    if let Some(ua) = ua {
        req = req.header("user-agent", ua);
    }
    let resp = req.send().await?.error_for_status()?;
    Ok(resp.text().await?)
}

pub async fn list_catalog(client: &reqwest::Client, force: bool) -> AppResult<Vec<BedrockVersion>> {
    let cache = cache_path();
    if !force {
        if let Ok(raw) = std::fs::read_to_string(&cache) {
            if let Ok(cached) = serde_json::from_str::<VersionDbCache>(&raw) {
                if now_secs().saturating_sub(cached.time) < CACHE_TTL_SECS && !cached.versions.is_empty() {
                    return Ok(cached.versions);
                }
            }
        }
    }

    let mut last_err = None;
    let attempts = [
        (VERSION_DB_URL, None),
        (MCAPPX_URL, Some(MCAPPX_UA)),
    ];
    for (url, ua) in attempts {
        match fetch_text(client, url, ua).await {
            Ok(text) => match parse_version_db(&text) {
                Ok(versions) if !versions.is_empty() => {
                    let payload = VersionDbCache {
                        time: now_secs(),
                        versions: versions.clone(),
                    };
                    if let Ok(json) = serde_json::to_string_pretty(&payload) {
                        let _ = std::fs::write(&cache, json);
                    }
                    return Ok(versions);
                }
                Ok(_) => last_err = Some(AppError::msg("El catálogo Bedrock vino vacío")),
                Err(e) => last_err = Some(e),
            },
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| AppError::msg("No se pudo leer el catálogo de versiones Bedrock")))
}

fn extra_http() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("SantiJ10/Paraguacraft (https://paraguacraft.gg)")
        .timeout(std::time::Duration::from_secs(4 * 3600))
        .connect_timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn emit_progress(app: &AppHandle, id: &str, label: &str, progress: f64, status: &str) {
    let _ = app.emit(
        "download://progress",
        DownloadProgress {
            id: id.to_string(),
            label: label.to_string(),
            progress,
            status: status.to_string(),
            speed: String::new(),
            error: None,
            failed_file: None,
        },
    );
}

fn emit_error(app: &AppHandle, id: &str, label: &str, err: &str) {
    let _ = app.emit(
        "download://progress",
        DownloadProgress {
            id: id.to_string(),
            label: label.to_string(),
            progress: 0.0,
            status: "error".into(),
            speed: String::new(),
            error: Some(err.to_string()),
            failed_file: None,
        },
    );
}

pub fn list_installed() -> Vec<BedrockInstalledVersion> {
    let root = versions_root();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let active_loc = registered_install_location();
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let meta_path = path.join(META_FILE);
        let Ok(raw) = std::fs::read_to_string(&meta_path) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<VersionMeta>(&raw) else {
            continue;
        };
        if meta.version.is_empty() {
            continue;
        }
        let active = active_loc
            .as_deref()
            .map(|loc| paths_match(&path, loc))
            .unwrap_or(false);
        out.push(BedrockInstalledVersion {
            version: meta.version,
            kind: meta.kind,
            path: path.to_string_lossy().to_string(),
            active,
        });
    }
    out.sort_by(|a, b| cmp_version(&b.version, &a.version));
    out
}

fn paths_match(dir: &Path, other: &str) -> bool {
    let a = dir
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .replace('/', "\\")
        .to_ascii_lowercase();
    let b = other
        .trim_end_matches(['\\', '/'])
        .replace('/', "\\")
        .to_ascii_lowercase();
    a == b
}

fn registered_install_location() -> Option<String> {
    #[cfg(windows)]
    {
        super::appx::query_registered().map(|p| p.install_location)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

pub fn extra_status() -> BedrockVersionStatus {
    #[cfg(windows)]
    {
        let pkg = super::appx::query_registered();
        let installed = list_installed();
        let loc = pkg.as_ref().map(|p| p.install_location.clone());
        let managed_active = loc
            .as_deref()
            .map(|l| {
                installed.iter().any(|v| paths_match(Path::new(&v.path), l))
            })
            .unwrap_or(false);
        let conflict_store = loc
            .as_deref()
            .map(super::appx::is_store_location)
            .unwrap_or(false);
        let active_version = if managed_active {
            installed.into_iter().find(|v| v.active).map(|v| v.version)
        } else {
            pkg.as_ref()
                .map(|p| p.version.clone())
                .filter(|s| !s.is_empty())
        };
        BedrockVersionStatus {
            store_installed: pkg.is_some() && conflict_store,
            managed_active,
            active_version,
            developer_mode: super::appx::is_developer_mode_enabled(),
            conflict_store,
        }
    }
    #[cfg(not(windows))]
    {
        BedrockVersionStatus {
            store_installed: false,
            managed_active: false,
            active_version: None,
            developer_mode: false,
            conflict_store: false,
        }
    }
}

async fn stream_download(app: &AppHandle, url: &str, dest: &Path, id: &str, label: &str) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("appx.part");
    let _ = std::fs::remove_file(&tmp);

    let client = extra_http();
    let resp = client.get(url).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let mut file = std::fs::File::create(&tmp)?;
    let mut downloaded: u64 = 0;
    let mut last_pct: u32 = 0;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        if total > 0 {
            let pct = ((downloaded as f64 / total as f64) * 80.0).floor() as u32;
            if pct != last_pct {
                last_pct = pct;
                emit_progress(app, id, label, pct as f64, "downloading");
            }
        }
    }
    file.flush()?;
    drop(file);
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

pub async fn install_version(app: &AppHandle, catalog_client: &reqwest::Client, version: &str) -> AppResult<()> {
    #[cfg(not(windows))]
    {
        let _ = (app, catalog_client, version);
        return Err(AppError::msg("Bedrock solo está disponible en Windows"));
    }
    #[cfg(windows)]
    {
        let catalog = list_catalog(catalog_client, false).await?;
        let entry = catalog
            .iter()
            .find(|v| v.version == version)
            .ok_or_else(|| AppError::msg(format!("No está en el catálogo la versión {version}")))?;
        if entry.kind != "release" {
            return Err(AppError::msg(
                "Las betas y Preview no se pueden bajar todavía (hace falta el programa Insider).",
            ));
        }
        let id = format!("bedrock-{version}");
        let label = format!("Bedrock {version}");
        emit_progress(app, &id, &label, 0.0, "downloading");

        let soap_client = extra_http();
        let url = match fe3::resolve_download_url(&soap_client, &entry.update_identity, "1").await {
            Ok(u) => u,
            Err(e) => {
                emit_error(app, &id, &label, &e.to_string());
                return Err(e);
            }
        };

        let cache_dir = versions_root().join(".cache");
        std::fs::create_dir_all(&cache_dir)?;
        let appx_path = cache_dir.join(format!("{}.appx", sanitize_version_dir(version)));
        if let Err(e) = stream_download(app, &url, &appx_path, &id, &label).await {
            emit_error(app, &id, &label, &e.to_string());
            let _ = std::fs::remove_file(&appx_path);
            let _ = std::fs::remove_file(appx_path.with_extension("appx.part"));
            return Err(e);
        }

        emit_progress(app, &id, &format!("Extrayendo Bedrock {version}"), 85.0, "downloading");
        let dest = version_dir(version);
        let dest_clone = dest.clone();
        let appx_clone = appx_path.clone();
        let extract = tokio::task::spawn_blocking(move || super::appx::extract_appx(&appx_clone, &dest_clone));
        if let Err(e) = extract.await.unwrap_or_else(|e| Err(AppError::msg(format!("Extract abortado: {e}")))) {
            emit_error(app, &id, &label, &e.to_string());
            let _ = std::fs::remove_file(&appx_path);
            return Err(e);
        }
        let meta = VersionMeta {
            version: version.to_string(),
            kind: entry.kind.clone(),
        };
        let _ = std::fs::write(dest.join(META_FILE), serde_json::to_string_pretty(&meta)?);
        let _ = std::fs::remove_file(&appx_path);
        emit_progress(app, &id, &format!("Bedrock {version} listo"), 100.0, "done");
        Ok(())
    }
}

pub fn switch_version(version: &str) -> AppResult<()> {
    #[cfg(not(windows))]
    {
        let _ = version;
        return Err(AppError::msg("Bedrock solo está disponible en Windows"));
    }
    #[cfg(windows)]
    {
        let installed = list_installed();
        let found = installed
            .iter()
            .find(|v| v.version == version)
            .ok_or_else(|| AppError::msg(format!("Bedrock {version} no está instalada")))?;
        if found.active {
            return Ok(());
        }
        super::windows::kill_bedrock();
        std::thread::sleep(std::time::Duration::from_millis(400));
        super::appx::register_package(Path::new(&found.path))?;
        Ok(())
    }
}

pub fn remove_version(version: &str) -> AppResult<()> {
    #[cfg(not(windows))]
    {
        let _ = version;
        return Err(AppError::msg("Bedrock solo está disponible en Windows"));
    }
    #[cfg(windows)]
    {
        let installed = list_installed();
        let found = installed
            .iter()
            .find(|v| v.version == version)
            .ok_or_else(|| AppError::msg(format!("Bedrock {version} no está instalada")))?;
        if found.active {
            super::windows::kill_bedrock();
            let _ = super::appx::unregister_package(Path::new(&found.path));
        }
        let _ = std::fs::remove_dir_all(&found.path);
        Ok(())
    }
}

pub fn enable_developer_mode() -> AppResult<()> {
    #[cfg(windows)]
    {
        super::appx::enable_developer_mode()
    }
    #[cfg(not(windows))]
    {
        Err(AppError::msg("Bedrock solo está disponible en Windows"))
    }
}

pub fn developer_mode() -> bool {
    extra_status().developer_mode
}

pub fn open_microsoft_store() -> AppResult<()> {
    #[cfg(windows)]
    {
        super::appx::open_store_product()
    }
    #[cfg(not(windows))]
    {
        Err(AppError::msg("Bedrock solo está disponible en Windows"))
    }
}

pub fn backup_saves() -> AppResult<String> {
    let src = super::com_mojang_dir().ok_or_else(|| {
        AppError::msg("No hay carpeta com.mojang todavía. Abrí Bedrock una vez o instalalo desde la Store.")
    })?;
    let stamp = now_secs();
    let dest = paths::data_dir()
        .join("bedrock-backup")
        .join(format!("com.mojang-{stamp}"));
    std::fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")))?;
    #[cfg(windows)]
    {
        super::appx::copy_dir(&src, &dest)?;
    }
    #[cfg(not(windows))]
    {
        let _ = src;
        return Err(AppError::msg("Bedrock solo está disponible en Windows"));
    }
    Ok(dest.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_legacy_array() {
        let json = r#"[["1.16.201.2","5754a03d-d8d5-489f-b24d-efc31b3fd32d",0],["1.21.30.3","aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",0],["1.21.40.1","bbbbbbbb-bbbb-cccc-dddd-eeeeeeeeeeee",1]]"#;
        let list = parse_version_db(json).unwrap();
        assert!(list.iter().any(|v| v.version == "1.21.30.3" && v.installable));
        assert!(list.iter().any(|v| v.version == "1.21.40.1" && !v.installable && v.kind == "beta"));
        assert_eq!(list[0].version, "1.21.40.1");
    }

    #[test]
    fn sanitize_dir() {
        assert_eq!(sanitize_version_dir("1.21.30.3"), "1.21.30.3");
        assert_eq!(sanitize_version_dir("1.21 foo/bar"), "1.21_foo_bar");
    }
}
