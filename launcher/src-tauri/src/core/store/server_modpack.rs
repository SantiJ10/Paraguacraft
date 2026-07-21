//! Importación de modpacks (.mrpack / CurseForge .zip) a servidores locales Fabric/Forge.

use std::io::Read;
use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::loaders;
use crate::core::net::{self, DownloadItem};
use crate::core::servers::{self, ServerProfile};
use crate::core::store::curseforge;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

const MR_API: &str = "https://api.modrinth.com/v2";

fn loader_to_server_type(loader: &str) -> AppResult<&'static str> {
    match loaders::normalize(loader).as_str() {
        // Quilt es compatible binariamente con mods Fabric server-side; NeoForge es un
        // loader propio (no corre sobre un server jar Forge) y necesita su propio
        // instalador Maven, así que ya no se colapsa dentro de "forge".
        "fabric" | "quilt" => Ok("fabric"),
        "forge" => Ok("forge"),
        "neoforge" => Ok("neoforge"),
        other => Err(AppError::msg(format!(
            "El modpack usa {other}; solo servidores Fabric, Forge o NeoForge están soportados."
        ))),
    }
}

fn include_for_server(env: &Value) -> bool {
    if env.is_null() {
        return true;
    }
    match env["server"].as_str() {
        Some("unsupported") => false,
        Some("required") | Some("optional") => true,
        _ => env["client"].as_str() != Some("required"),
    }
}

fn read_mr_index(bytes: &[u8]) -> AppResult<Value> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut file = zip
        .by_name("modrinth.index.json")
        .map_err(|_| AppError::msg("El .mrpack no contiene modrinth.index.json"))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(serde_json::from_str(&buf)?)
}

fn detect_mr_loader(deps: &Value) -> (String, String) {
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
    ("vanilla".into(), String::new())
}

fn apply_zip_prefixes(bytes: &[u8], dest: &PathBuf, prefixes: &[&str]) -> AppResult<()> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(name) = entry.enclosed_name().map(|p| p.to_string_lossy().replace('\\', "/")) else {
            continue;
        };
        let rel = prefixes
            .iter()
            .find_map(|pfx| name.strip_prefix(&format!("{pfx}/")));
        let Some(rel) = rel else {
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

async fn bootstrap_server(
    app: &AppHandle,
    state: &AppState,
    client: &reqwest::Client,
    name: &str,
    mc: &str,
    loader: &str,
    ram_mb: u32,
) -> AppResult<(ServerProfile, PathBuf)> {
    let server_type = loader_to_server_type(loader)?;
    let prof = servers::create(name, mc, server_type, ram_mb)?;
    let dest = servers::folder_for_id(&prof.id)?;
    crate::core::java::resolve::ensure_installer_java(app, state, mc).await?;
    servers::ensure_server_jar(app, client, &prof.id).await?;
    Ok((prof, dest))
}

/// Instala un .mrpack de Modrinth en un servidor local nuevo.
pub async fn import_mrpack_version_to_server(
    app: &AppHandle,
    state: &AppState,
    client: &reqwest::Client,
    version_id: &str,
    ram_mb: u32,
) -> AppResult<ServerProfile> {
    let version: Value = net::fetch_json(client, &format!("{MR_API}/version/{version_id}")).await?;
    let files = version["files"].as_array().cloned().unwrap_or_default();
    let mrpack = files
        .iter()
        .find(|f| f["filename"].as_str().unwrap_or("").ends_with(".mrpack"))
        .ok_or_else(|| AppError::msg("Esta versión no incluye archivo .mrpack"))?;
    let mrpack_url = mrpack["url"].as_str().ok_or_else(|| AppError::msg("Sin URL .mrpack"))?;

    let bytes = net::fetch_bytes(client, mrpack_url).await?;
    let index = {
        let b = bytes.clone();
        super::run_blocking(move || read_mr_index(&b)).await?
    };
    let name = index["name"].as_str().unwrap_or("Modpack").to_string();
    let deps = &index["dependencies"];
    let mc = deps["minecraft"]
        .as_str()
        .ok_or_else(|| AppError::msg("El modpack no declara versión de Minecraft"))?
        .to_string();
    let (loader, _) = detect_mr_loader(deps);

    let (prof, dest) = bootstrap_server(app, state, client, &name, &mc, &loader, ram_mb).await?;

    let mut items = Vec::new();
    if let Some(file_list) = index["files"].as_array() {
        for f in file_list {
            if !include_for_server(&f["env"]) {
                continue;
            }
            let Some(path) = f["path"].as_str() else {
                continue;
            };
            let url = f["downloads"]
                .as_array()
                .and_then(|d| d.first())
                .and_then(|u| u.as_str());
            let Some(url) = url else {
                continue;
            };
            let sha1 = f["hashes"]["sha1"].as_str().map(String::from);
            items.push(DownloadItem::new(url, dest.join(path)).with_sha1(sha1));
        }
    }
    if !items.is_empty() {
        net::download_all(
            client,
            items,
            12,
            app,
            "server-modpack",
            &format!("Servidor {name}"),
        )
        .await?;
    }
    super::run_blocking({
        let b = bytes.clone();
        let d = dest.clone();
        move || apply_zip_prefixes(&b, &d, &["overrides", "server-overrides"])
    })
    .await?;

    Ok(prof)
}

fn read_cf_manifest(bytes: &[u8]) -> AppResult<Value> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut file = zip
        .by_name("manifest.json")
        .map_err(|_| AppError::msg("El .zip no contiene manifest.json"))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(serde_json::from_str(&buf)?)
}

fn parse_cf_loader(id: &str) -> (String, String) {
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

fn detect_cf_loader(manifest: &Value) -> (String, String) {
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
            let (loader, _) = parse_cf_loader(id);
            return (mc, loader);
        }
    }
    (mc, "vanilla".into())
}

async fn install_cf_manifest_to_server(
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
            net::download_all(client, items, 8, app, &format!("srv-cfpack-{i}"), label).await?;
            items = Vec::new();
        }
    }
    if !items.is_empty() {
        net::download_all(client, items, 8, app, "srv-cfpack-final", label).await?;
    }
    Ok(())
}

/// Instala un modpack CurseForge (.zip) en un servidor local nuevo.
pub async fn import_cfpack_version_to_server(
    app: &AppHandle,
    state: &AppState,
    client: &reqwest::Client,
    cf_key: &str,
    mod_id: &str,
    file_id: &str,
    ram_mb: u32,
) -> AppResult<ServerProfile> {
    if cf_key.trim().is_empty() {
        return Err(AppError::msg(
            "CurseForge requiere una API key (.env o Ajustes).",
        ));
    }
    let file_resp = curseforge::get_file_metadata(client, cf_key, mod_id, file_id).await?;
    let filename = file_resp["fileName"].as_str().unwrap_or("modpack.zip");
    if !filename.ends_with(".zip") {
        return Err(AppError::msg(
            "Esta versión no es un .zip de modpack CurseForge.",
        ));
    }
    let name = curseforge::mod_name(client, cf_key, mod_id)
        .await
        .unwrap_or_else(|_| "Modpack CurseForge".into());
    let url = curseforge::file_download_url(&file_resp);
    let bytes = net::fetch_bytes(client, &url).await?;
    let manifest = {
        let b = bytes.clone();
        super::run_blocking(move || read_cf_manifest(&b)).await?
    };
    if manifest["manifestType"].as_str() != Some("minecraftModpack") {
        return Err(AppError::msg("No es un modpack de Minecraft válido"));
    }
    let pack_name = manifest["name"].as_str().unwrap_or(&name).to_string();
    let (mc, loader) = detect_cf_loader(&manifest);
    if mc.is_empty() {
        return Err(AppError::msg("El modpack no declara versión de Minecraft"));
    }

    let (prof, dest) =
        bootstrap_server(app, state, client, &pack_name, &mc, &loader, ram_mb).await?;

    install_cf_manifest_to_server(app, client, cf_key, &manifest, &dest, &pack_name).await?;

    let overrides = manifest["overrides"]
        .as_str()
        .unwrap_or("overrides")
        .trim_end_matches('/')
        .to_string();
    super::run_blocking({
        let b = bytes.clone();
        let d = dest.clone();
        move || apply_zip_prefixes(&b, &d, &[&overrides])
    })
    .await?;

    Ok(prof)
}
