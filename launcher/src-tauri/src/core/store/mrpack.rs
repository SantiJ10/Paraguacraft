//! Importador de modpacks Modrinth (`.mrpack`). Espeja `importar_modpack_modrinth`.
//!
//! Descarga el `.mrpack`, lee `modrinth.index.json`, crea una instancia con la
//! version + loader exactos, baja todos los archivos (verificados por SHA-1) a
//! sus rutas y aplica los `overrides`.

use std::io::Read;
use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::instances::{self, profiles};
use crate::core::net::{self, DownloadItem};
use crate::error::{AppError, AppResult};
use crate::models::Instance;

const API: &str = "https://api.modrinth.com/v2";

fn parse_slug(input: &str) -> String {
    let s = input.trim();
    if let Some(idx) = s.find("modrinth.com/modpack/") {
        let rest = &s[idx + "modrinth.com/modpack/".len()..];
        return rest.split(['/', '?', '#']).next().unwrap_or(rest).to_string();
    }
    s.to_string()
}

/// Detecta (loader, loader_version) desde las dependencies del index.
fn detect_loader(deps: &Value) -> (String, String) {
    for (key, loader) in [
        ("fabric-loader", "fabric"),
        ("quilt-loader", "quilt"),
        ("neoforge", "neoforge"),
        ("forge", "forge"),
    ] {
        if let Some(v) = deps.get(key).and_then(|x| x.as_str()) {
            return (loader.to_string(), v.to_string());
        }
    }
    ("vanilla".to_string(), String::new())
}

/// Lee `modrinth.index.json` desde los bytes del `.mrpack`.
fn read_index(bytes: &[u8]) -> AppResult<Value> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut file = zip
        .by_name("modrinth.index.json")
        .map_err(|_| AppError::msg("El .mrpack no contiene modrinth.index.json"))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(serde_json::from_str(&buf)?)
}

/// Extrae los `overrides/` (y `client-overrides/`) del `.mrpack` a la instancia.
fn apply_overrides(bytes: &[u8], dest: &PathBuf) -> AppResult<()> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(name) = entry.enclosed_name().map(|p| p.to_string_lossy().replace('\\', "/")) else {
            continue;
        };
        let rel = if let Some(r) = name.strip_prefix("overrides/") {
            r
        } else if let Some(r) = name.strip_prefix("client-overrides/") {
            r
        } else {
            continue;
        };
        if rel.is_empty() || entry.is_dir() {
            continue;
        }
        let out = dest.join(rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        std::fs::write(&out, &buf)?;
    }
    Ok(())
}

pub async fn import_modrinth(
    app: &AppHandle,
    client: &reqwest::Client,
    source: &str,
    mc_filter: &str,
) -> AppResult<Instance> {
    let slug = parse_slug(source);

    // 1) Buscar una version del modpack con archivo .mrpack.
    let versions: Value = net::fetch_json(client, &format!("{API}/project/{slug}/version")).await?;
    let arr = versions.as_array().cloned().unwrap_or_default();
    let mut chosen: Option<Value> = None;
    for v in &arr {
        let ok_mc = mc_filter.is_empty()
            || v["game_versions"]
                .as_array()
                .map(|gv| gv.iter().any(|g| g.as_str() == Some(mc_filter)))
                .unwrap_or(false);
        let has_mrpack = v["files"]
            .as_array()
            .map(|fs| fs.iter().any(|f| f["filename"].as_str().unwrap_or("").ends_with(".mrpack")))
            .unwrap_or(false);
        if ok_mc && has_mrpack {
            chosen = Some(v.clone());
            break;
        }
    }
    let version = chosen.ok_or_else(|| AppError::msg("No se encontro un .mrpack compatible"))?;
    let files = version["files"].as_array().cloned().unwrap_or_default();
    let mrpack = files
        .iter()
        .find(|f| f["filename"].as_str().unwrap_or("").ends_with(".mrpack"))
        .ok_or_else(|| AppError::msg("Sin archivo .mrpack"))?;
    let mrpack_url = mrpack["url"].as_str().ok_or_else(|| AppError::msg("Sin URL .mrpack"))?;

    // 2) Descargar y leer el index.
    let bytes = net::fetch_bytes(client, mrpack_url).await?;
    let index = read_index(&bytes)?;
    let name = index["name"].as_str().unwrap_or(&slug).to_string();
    let deps = &index["dependencies"];
    let mc = deps["minecraft"].as_str().unwrap_or_default().to_string();
    if mc.is_empty() {
        return Err(AppError::msg("El modpack no declara version de Minecraft"));
    }
    let (loader, loader_version) = detect_loader(deps);

    // 3) Crear instancia local.
    let inst = profiles::create(&name, &mc, &loader, &loader_version, "\u{1F4E6}", 4096)?;
    let dest = instances::instance_dir(&inst.id);

    // 4) Descargar archivos del index a sus rutas.
    let mut items = Vec::new();
    if let Some(file_list) = index["files"].as_array() {
        for f in file_list {
            let Some(path) = f["path"].as_str() else { continue };
            let url = f["downloads"].as_array().and_then(|d| d.first()).and_then(|u| u.as_str());
            let Some(url) = url else { continue };
            let sha1 = f["hashes"]["sha1"].as_str().map(String::from);
            items.push(DownloadItem::new(url, dest.join(path)).with_sha1(sha1));
        }
    }
    net::download_all(client, items, 12, app, "mrpack-import", &format!("Modpack {name}")).await?;

    // 5) Aplicar overrides.
    apply_overrides(&bytes, &dest)?;

    // 6) Instalar el loader para dejarla jugable y fijar version_id.
    let version_id =
        crate::core::loaders::install_loader(app, client, &mc, &loader, &loader_version).await?;
    if let Some(mut meta) = instances::read_meta(&inst.id) {
        meta.version_id = Some(version_id);
        let _ = instances::write_meta(&inst.id, &meta);
    }

    instances::read_meta(&inst.id)
        .map(|m| m.into_instance(&inst.id, &dest))
        .ok_or_else(|| AppError::msg("No se pudo leer la instancia importada"))
}

/// Importa un modpack desde un id de version Modrinth (tienda → modpacks).
pub async fn import_by_version_id(
    app: &AppHandle,
    client: &reqwest::Client,
    version_id: &str,
) -> AppResult<Instance> {
    let version: Value = net::fetch_json(client, &format!("{API}/version/{version_id}")).await?;
    let files = version["files"].as_array().cloned().unwrap_or_default();
    let mrpack = files
        .iter()
        .find(|f| f["filename"].as_str().unwrap_or("").ends_with(".mrpack"))
        .ok_or_else(|| AppError::msg("Esta version no incluye archivo .mrpack"))?;
    let mrpack_url = mrpack["url"].as_str().ok_or_else(|| AppError::msg("Sin URL .mrpack"))?;
    let slug = version["project_id"]
        .as_str()
        .unwrap_or("modpack")
        .to_string();

    let bytes = net::fetch_bytes(client, mrpack_url).await?;
    let index = read_index(&bytes)?;
    let name = index["name"].as_str().unwrap_or(&slug).to_string();
    let deps = &index["dependencies"];
    let mc = deps["minecraft"].as_str().unwrap_or_default().to_string();
    if mc.is_empty() {
        return Err(AppError::msg("El modpack no declara version de Minecraft"));
    }
    let (loader, loader_version) = detect_loader(deps);

    let inst = profiles::create(&name, &mc, &loader, &loader_version, "\u{1F4E6}", 4096)?;
    let dest = instances::instance_dir(&inst.id);

    let mut items = Vec::new();
    if let Some(file_list) = index["files"].as_array() {
        for f in file_list {
            let Some(path) = f["path"].as_str() else { continue };
            let url = f["downloads"].as_array().and_then(|d| d.first()).and_then(|u| u.as_str());
            let Some(url) = url else { continue };
            let sha1 = f["hashes"]["sha1"].as_str().map(String::from);
            items.push(DownloadItem::new(url, dest.join(path)).with_sha1(sha1));
        }
    }
    net::download_all(client, items, 12, app, "mrpack-import", &format!("Modpack {name}")).await?;
    apply_overrides(&bytes, &dest)?;

    let version_id_installed =
        crate::core::loaders::install_loader(app, client, &mc, &loader, &loader_version).await?;
    if let Some(mut meta) = instances::read_meta(&inst.id) {
        meta.version_id = Some(version_id_installed);
        let _ = instances::write_meta(&inst.id, &meta);
    }

    instances::read_meta(&inst.id)
        .map(|m| m.into_instance(&inst.id, &dest))
        .ok_or_else(|| AppError::msg("No se pudo leer la instancia importada"))
}
