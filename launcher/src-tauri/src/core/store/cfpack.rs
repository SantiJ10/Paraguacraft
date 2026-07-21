//! Importador de modpacks CurseForge (.zip con manifest.json).

use std::io::Read;
use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::instances::{self, profiles};
use crate::core::loaders;
use crate::core::net::{self, DownloadItem};
use crate::core::store::curseforge;
use crate::error::{AppError, AppResult};
use crate::models::Instance;

fn read_manifest(bytes: &[u8]) -> AppResult<Value> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut file = zip
        .by_name("manifest.json")
        .map_err(|_| AppError::msg("El .zip no contiene manifest.json (formato CurseForge)"))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(serde_json::from_str(&buf)?)
}

fn parse_loader_id(id: &str) -> (String, String) {
    let id = id.trim();
    for (prefix, loader) in [
        ("forge-", "forge"),
        ("neoforge-", "neoforge"),
        ("fabric-", "fabric"),
        ("quilt-", "quilt"),
    ] {
        if let Some(v) = id.strip_prefix(prefix) {
            return (loader.to_string(), v.to_string());
        }
    }
    ("vanilla".into(), String::new())
}

fn detect_loader(manifest: &Value) -> (String, String, String) {
    let mc = manifest["minecraft"]["version"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let loaders_arr = manifest["minecraft"]["modLoaders"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let chosen = loaders_arr
        .iter()
        .find(|l| l["primary"].as_bool() == Some(true))
        .or_else(|| loaders_arr.first());
    if let Some(l) = chosen {
        if let Some(id) = l["id"].as_str() {
            let (loader, lv) = parse_loader_id(id);
            return (mc, loader, lv);
        }
    }
    (mc, "vanilla".into(), String::new())
}

fn apply_overrides(bytes: &[u8], dest: &PathBuf, overrides_folder: &str) -> AppResult<()> {
    let prefix = format!("{overrides_folder}/");
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(name) = entry.enclosed_name().map(|p| p.to_string_lossy().replace('\\', "/")) else {
            continue;
        };
        let Some(rel) = name.strip_prefix(&prefix) else {
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

async fn install_manifest_files(
    app: &AppHandle,
    client: &reqwest::Client,
    key: &str,
    manifest: &Value,
    dest: &PathBuf,
    label: &str,
) -> AppResult<()> {
    let files = manifest["files"].as_array().cloned().unwrap_or_default();
    if files.is_empty() {
        return Err(AppError::msg("El manifest no declara archivos del modpack"));
    }
    let mods_dir = dest.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let mut items = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let project_id = f["projectID"].as_u64().unwrap_or(0).to_string();
        let file_id = f["fileID"].as_u64().unwrap_or(0).to_string();
        if project_id == "0" || file_id == "0" {
            continue;
        }
        let file_resp = curseforge::get_file_metadata(client, key, &project_id, &file_id).await?;
        let filename = file_resp["fileName"]
            .as_str()
            .unwrap_or("mod.jar")
            .to_string();
        let dl = curseforge::file_download_url(&file_resp);
        let sha1 = curseforge::file_sha1(&file_resp);
        items.push(DownloadItem::new(dl, mods_dir.join(&filename)).with_sha1(sha1));
        if items.len() >= 24 {
            net::download_all(client, items, 8, app, &format!("cfpack-{i}"), label).await?;
            items = Vec::new();
        }
    }
    if !items.is_empty() {
        net::download_all(client, items, 8, app, "cfpack-final", label).await?;
    }
    Ok(())
}

async fn import_from_manifest_bytes(
    app: &AppHandle,
    client: &reqwest::Client,
    key: &str,
    zip_bytes: &[u8],
    name_override: Option<&str>,
) -> AppResult<Instance> {
    let manifest = {
        let b = zip_bytes.to_vec();
        super::run_blocking(move || read_manifest(&b)).await?
    };
    if manifest["manifestType"].as_str() != Some("minecraftModpack") {
        return Err(AppError::msg("No es un modpack de Minecraft (manifestType invalido)"));
    }
    let pack_name = name_override
        .map(String::from)
        .or_else(|| manifest["name"].as_str().map(String::from))
        .unwrap_or_else(|| "Modpack CurseForge".into());
    let (mc, loader, loader_version) = detect_loader(&manifest);
    if mc.is_empty() {
        return Err(AppError::msg("El modpack no declara version de Minecraft"));
    }

    let inst = profiles::create(&pack_name, &mc, &loader, &loader_version, "\u{1F4E6}", 4096)?;
    let dest = instances::instance_dir(&inst.id);

    let version_id =
        loaders::install_loader(app, client, &mc, &loader, &loader_version).await?;
    if let Some(mut meta) = instances::read_meta(&inst.id) {
        meta.version_id = Some(version_id);
        meta.source = "curseforge".into();
        let _ = instances::write_meta(&inst.id, &meta);
    }

    install_manifest_files(app, client, key, &manifest, &dest, &pack_name).await?;

    let overrides = manifest["overrides"]
        .as_str()
        .unwrap_or("overrides")
        .trim_end_matches('/')
        .to_string();
    super::run_blocking({
        let b = zip_bytes.to_vec();
        let d = dest.clone();
        move || apply_overrides(&b, &d, &overrides)
    })
    .await?;

    instances::read_meta(&inst.id)
        .map(|m| m.into_instance(&inst.id, &dest))
        .ok_or_else(|| AppError::msg("No se pudo leer la instancia importada"))
}

/// Importa un modpack CurseForge por id de proyecto + id de archivo (.zip).
pub async fn import_by_file_id(
    app: &AppHandle,
    client: &reqwest::Client,
    key: &str,
    mod_id: &str,
    file_id: &str,
) -> AppResult<Instance> {
    if key.trim().is_empty() {
        return Err(AppError::msg(
            "CurseForge requiere una API key (.env o Ajustes).",
        ));
    }
    let file_resp = curseforge::get_file_metadata(client, key, mod_id, file_id).await?;
    let filename = file_resp["fileName"].as_str().unwrap_or("modpack.zip");
    if !filename.ends_with(".zip") {
        return Err(AppError::msg(
            "Esta version no es un .zip de modpack CurseForge. Elegí otra version.",
        ));
    }
    let name = curseforge::mod_name(client, key, mod_id).await.ok();
    let url = curseforge::file_download_url(&file_resp);
    let bytes = net::fetch_bytes(client, &url).await?;
    import_from_manifest_bytes(app, client, key, &bytes, name.as_deref()).await
}

/// Importa desde un .zip local (manifest CurseForge).
pub async fn import_from_zip_path(
    app: &AppHandle,
    client: &reqwest::Client,
    key: &str,
    path: &std::path::Path,
) -> AppResult<Instance> {
    let bytes = super::run_blocking({
        let p = path.to_path_buf();
        move || Ok(std::fs::read(&p)?)
    })
    .await?;
    import_from_manifest_bytes(app, client, key, &bytes, None).await
}

/// Busca e importa el modpack CurseForge por slug/URL/id de proyecto.
pub async fn import_from_project(
    app: &AppHandle,
    client: &reqwest::Client,
    key: &str,
    source: &str,
    mc_filter: &str,
) -> AppResult<Instance> {
    if key.trim().is_empty() {
        return Err(AppError::msg(
            "CurseForge requiere una API key (.env o Ajustes).",
        ));
    }
    let mod_id = curseforge::resolve_modpack_id(client, key, source).await?;
    let files = curseforge::list_mod_files(client, key, &mod_id, mc_filter).await?;
    let file = files
        .iter()
        .find(|f| f["fileName"].as_str().unwrap_or("").ends_with(".zip"))
        .ok_or_else(|| AppError::msg("No se encontro un .zip de modpack compatible"))?;
    let file_id = file["id"].as_u64().unwrap_or(0).to_string();
    import_by_file_id(app, client, key, &mod_id, &file_id).await
}
