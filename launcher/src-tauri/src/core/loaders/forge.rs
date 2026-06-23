//! Provider Forge (maven.minecraftforge.net).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::core::versions;
use crate::error::{AppError, AppResult};

const MAVEN: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";

/// Forge <= 1.12.x usa coordenadas Maven `{mc}-{forge}-{mc}` (ej. `1.8.9-11.15.1.2318-1.8.9`).
fn uses_legacy_maven_suffix(mc: &str) -> bool {
    let parts: Vec<&str> = mc.split('.').collect();
    let major: u32 = parts.first().and_then(|p| p.parse().ok()).unwrap_or(99);
    let minor: u32 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(99);
    major < 1 || (major == 1 && minor <= 12)
}

/// Convierte la version mostrada en la UI al id Maven completo.
pub fn resolve_maven_id(mc: &str, forge_version: &str) -> String {
    let v = forge_version.trim();
    if v.starts_with(&format!("{mc}-")) {
        return v.to_string();
    }
    if v.ends_with(&format!("-{mc}")) {
        return format!("{mc}-{v}");
    }
    if uses_legacy_maven_suffix(mc) {
        format!("{mc}-{v}-{mc}")
    } else {
        format!("{mc}-{v}")
    }
}

/// Version legible para la UI a partir del id Maven.
fn display_version(mc: &str, full: &str) -> String {
    let prefix = format!("{mc}-");
    let suffix = format!("-{mc}");
    let mut s = full.strip_prefix(&prefix).unwrap_or(full).to_string();
    if s.ends_with(&suffix) {
        s.truncate(s.len() - suffix.len());
    }
    s
}

pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    let url = format!("{MAVEN}/maven-metadata.xml");
    let xml = match net::fetch_bytes(client, &url).await {
        Ok(b) => String::from_utf8_lossy(&b).to_string(),
        Err(_) => return Ok(vec![]),
    };
    let prefix = format!("{mc}-");
    let mut matching: Vec<String> = super::parse_maven_versions(&xml)
        .into_iter()
        .filter(|v| v.starts_with(&prefix))
        .map(|v| display_version(mc, &v))
        .collect();
    matching.sort();
    matching.dedup();
    matching.reverse();
    Ok(matching)
}

fn legacy_library_item(lib: &Value) -> Option<DownloadItem> {
    if lib["clientreq"].as_bool() == Some(false) {
        return None;
    }
    let name = lib["name"].as_str()?;
    let rel = versions::maven_to_path(name)?;
    let dest = paths::default_minecraft_dir().join("libraries").join(&rel);
    let base = lib["url"].as_str().unwrap_or("https://libraries.minecraft.net/");
    let url = format!(
        "{}/{}",
        base.trim_end_matches('/'),
        rel.to_string_lossy().replace('\\', "/")
    );
    let sha1 = lib["checksums"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|x| x.as_str())
        .map(String::from);
    Some(DownloadItem::new(url, dest).with_sha1(sha1))
}

fn extract_zip_entry(archive: &mut zip::ZipArchive<File>, name: &str, dest: &Path) -> AppResult<()> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| AppError::msg(format!("Falta {name} en el instalador Forge")))?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf)?;
    std::fs::write(dest, buf)?;
    Ok(())
}

/// Instala Forge legacy (<=1.12) leyendo install_profile.json del installer (sin --installClient).
async fn install_legacy(
    app: &AppHandle,
    client: &reqwest::Client,
    full: &str,
    installer_path: &Path,
) -> AppResult<String> {
    let file = File::open(installer_path)
        .map_err(|e| AppError::msg(format!("No se pudo abrir instalador Forge: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::msg(format!("Instalador Forge invalido: {e}")))?;

    let mut profile_raw = String::new();
    {
        let mut f = archive
            .by_name("install_profile.json")
            .map_err(|_| AppError::msg("install_profile.json ausente en instalador Forge"))?;
        f.read_to_string(&mut profile_raw)?;
    }
    let profile: Value = serde_json::from_str(&profile_raw)?;
    let version_info = profile
        .get("versionInfo")
        .ok_or_else(|| AppError::msg("versionInfo ausente en install_profile.json"))?
        .clone();
    let forge_id = version_info["id"]
        .as_str()
        .ok_or_else(|| AppError::msg("versionInfo.id ausente"))?
        .to_string();

    let version_dir = versions::versions_dir().join(&forge_id);
    std::fs::create_dir_all(&version_dir)?;
    crate::config::write_json_atomic(&version_dir.join(format!("{forge_id}.json")), &version_info)?;

    let forge_lib_dir = paths::default_minecraft_dir()
        .join("libraries")
        .join("net")
        .join("minecraftforge")
        .join("forge")
        .join(full);
    std::fs::create_dir_all(&forge_lib_dir)?;

    let universal_name = format!("forge-{full}-universal.jar");
    let universal_dest = forge_lib_dir.join(&universal_name);
    extract_zip_entry(&mut archive, &universal_name, &universal_dest)?;

    // El classpath legacy busca forge-{full}.jar
    let standard_dest = forge_lib_dir.join(format!("forge-{full}.jar"));
    if !standard_dest.exists() {
        std::fs::copy(&universal_dest, &standard_dest)?;
    }

    let mut items = Vec::new();
    if let Some(libs) = version_info["libraries"].as_array() {
        for lib in libs {
            if let Some(item) = legacy_library_item(lib) {
                // Ya tenemos el universal/local de forge
                if item.dest == standard_dest || item.dest == universal_dest {
                    continue;
                }
                items.push(item);
            }
        }
    }

    if !items.is_empty() {
        net::download_all(
            client,
            items,
            8,
            app,
            "install-loader",
            &format!("Librerias Forge {forge_id}"),
        )
        .await?;
    }

    Ok(forge_id)
}

pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    forge_version: &str,
) -> AppResult<String> {
    let full = resolve_maven_id(mc, forge_version);
    let installer_url = format!("{MAVEN}/{full}/forge-{full}-installer.jar");
    let installers = paths::java_dir().join("installers");
    std::fs::create_dir_all(&installers)?;
    let jar = installers.join(format!("forge-{full}-installer.jar"));

    net::download_all(
        client,
        vec![DownloadItem::new(installer_url, jar.clone())],
        1,
        app,
        "install-loader",
        &format!("Instalador Forge {forge_version}"),
    )
    .await?;

    if uses_legacy_maven_suffix(mc) {
        return install_legacy(app, client, &full, &jar).await;
    }

    let state = app.state::<crate::state::AppState>();
    let java = super::installer_java(app, &state, mc).await?;
    let mc_dir = paths::default_minecraft_dir();
    std::fs::create_dir_all(&mc_dir)?;
    super::run_installer(
        java,
        vec![
            "-jar".into(),
            jar.to_string_lossy().to_string(),
            "--installClient".into(),
            mc_dir.to_string_lossy().to_string(),
        ],
    )
    .await?;

    super::find_installed_version_id(&[mc, "forge"])
        .or_else(|| super::find_installed_version_id(&["forge", forge_version]))
        .or_else(|| super::find_installed_version_id(&[&full]))
        .ok_or_else(|| AppError::msg("Forge se instalo pero no se encontro el perfil generado"))
}
