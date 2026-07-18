//! Tienda Modrinth (api.modrinth.com/v2).
//!
//! Búsqueda global sin mc/loader, o filtrada cuando se pasan (instalación / modal).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::error::{AppError, AppResult};
use crate::models::{StoreItem, StoreVersion};

const API: &str = "https://api.modrinth.com/v2";

/// Indica si un tipo de contenido depende del loader (para filtrar por loader).
pub fn loader_relevant(project_type: &str) -> bool {
    matches!(project_type, "mod" | "modpack" | "plugin")
}

const PLUGIN_LOADERS_PAPER: &[&str] = &["paper", "spigot", "bukkit", "purpur", "folia"];

fn loaders_query(project_type: &str, loader: &str) -> Option<String> {
    if project_type == "datapack" {
        return Some("[\"minecraft\"]".into());
    }
    if project_type == "plugin" {
        if loader == "fabric" {
            return Some("[\"fabric\"]".into());
        }
        let list: Vec<&str> = PLUGIN_LOADERS_PAPER.to_vec();
        let json: String = list
            .iter()
            .map(|l| format!("\"{l}\""))
            .collect::<Vec<_>>()
            .join(",");
        return Some(format!("[{json}]"));
    }
    if loader_relevant(project_type) && !loader.is_empty() && loader != "vanilla" {
        return Some(format!("[\"{loader}\"]"));
    }
    None
}

/// Construye las facetas de busqueda filtrando por tipo + version (+ loader).
fn build_facets(project_type: &str, mc: &str, loader: &str) -> String {
    let mut groups: Vec<String> = vec![format!("[\"project_type:{project_type}\"]")];
    if !mc.is_empty() {
        groups.push(format!("[\"versions:{mc}\"]"));
    }
    if loader_relevant(project_type) && !loader.is_empty() && loader != "vanilla" {
        if project_type == "plugin" && PLUGIN_LOADERS_PAPER.contains(&loader) {
            groups.push(format!(
                "[{}]",
                PLUGIN_LOADERS_PAPER
                    .iter()
                    .map(|l| format!("\"categories:{l}\""))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        } else {
            groups.push(format!("[\"categories:{loader}\"]"));
        }
    }
    format!("[{}]", groups.join(","))
}

fn hit_to_item(hit: &Value) -> StoreItem {
    StoreItem {
        id: hit["project_id"].as_str().unwrap_or_default().to_string(),
        slug: hit["slug"].as_str().unwrap_or_default().to_string(),
        title: hit["title"].as_str().unwrap_or_default().to_string(),
        author: hit["author"].as_str().unwrap_or_default().to_string(),
        description: hit["description"].as_str().unwrap_or_default().to_string(),
        icon_url: hit["icon_url"].as_str().unwrap_or_default().to_string(),
        downloads: hit["downloads"].as_u64().unwrap_or(0),
        follows: hit["follows"].as_u64().unwrap_or(0),
        project_type: hit["project_type"].as_str().unwrap_or(project_default()).to_string(),
        provider: "modrinth".into(),
        categories: hit["categories"]
            .as_array()
            .map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        project_url: None,
    }
}

fn project_default() -> &'static str {
    "mod"
}

pub async fn search(
    client: &reqwest::Client,
    query: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
) -> AppResult<Vec<StoreItem>> {
    let facets = build_facets(project_type, mc, loader);
    let url = format!(
        "{API}/search?query={}&limit=40&index=relevance&facets={}",
        net::url_encode(query),
        net::url_encode(&facets)
    );
    let resp: Value = net::fetch_json(client, &url).await?;
    let hits = resp["hits"].as_array().cloned().unwrap_or_default();
    Ok(hits.iter().map(hit_to_item).collect())
}

fn pick_file<'a>(files: &'a [Value], project_type: &str) -> Option<&'a Value> {
    if project_type == "modpack" {
        return files
            .iter()
            .find(|f| f["filename"].as_str().unwrap_or("").ends_with(".mrpack"));
    }
    files
        .iter()
        .find(|f| f["primary"].as_bool().unwrap_or(false))
        .or_else(|| files.first())
}

fn version_from_json(v: &Value) -> StoreVersion {
    version_from_json_typed(v, "")
}

fn version_from_json_typed(v: &Value, project_type: &str) -> StoreVersion {
    let files = v["files"].as_array().cloned().unwrap_or_default();
    let file = pick_file(&files, project_type);
    let filename = file
        .and_then(|f| f["filename"].as_str())
        .unwrap_or("download.jar")
        .to_string();
    StoreVersion {
        id: v["id"].as_str().unwrap_or_default().to_string(),
        name: v["name"].as_str().unwrap_or_default().to_string(),
        version_number: v["version_number"].as_str().unwrap_or_default().to_string(),
        filename,
        download_url: file.and_then(|f| f["url"].as_str().map(String::from)),
        sha1: file
            .and_then(|f| f["hashes"]["sha1"].as_str().map(String::from)),
        game_versions: v["game_versions"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        loaders: v["loaders"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        published_at: v["date_published"].as_str().unwrap_or("").to_string(),
    }
}

/// Lista versiones del proyecto filtradas por mc + loader.
pub async fn list_versions(
    client: &reqwest::Client,
    project_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
) -> AppResult<Vec<StoreVersion>> {
    let mut url = format!(
        "{API}/project/{project_id}/version?game_versions={}",
        net::url_encode(&format!("[\"{mc}\"]"))
    );
    if let Some(q) = loaders_query(project_type, loader) {
        url.push_str(&format!("&loaders={}", net::url_encode(&q)));
    }
    let versions: Value = net::fetch_json(client, &url).await?;
    let arr = versions.as_array().cloned().unwrap_or_default();
    Ok(arr.iter().map(version_from_json).collect())
}

/// Todas las versiones del proyecto (sin filtrar mc/loader). Para el asistente de modpacks.
pub async fn list_all_versions(
    client: &reqwest::Client,
    project_id: &str,
    project_type: &str,
) -> AppResult<Vec<StoreVersion>> {
    let url = format!("{API}/project/{project_id}/version");
    let versions: Value = net::fetch_json(client, &url).await?;
    let arr = versions.as_array().cloned().unwrap_or_default();
    Ok(arr
        .iter()
        .filter_map(|v| {
            if project_type == "modpack" && pick_file(v["files"].as_array()?.as_slice(), project_type).is_none()
            {
                return None;
            }
            Some(version_from_json_typed(v, project_type))
        })
        .collect())
}

/// Instala un archivo concreto por id de version Modrinth.
pub async fn install_version_id(
    app: &AppHandle,
    client: &reqwest::Client,
    version_id: &str,
    dest_dir: std::path::PathBuf,
    file_hint: Option<(String, String, Option<String>)>,
) -> AppResult<String> {
    let (filename, dl, sha1) = match fetch_version_file(client, version_id).await {
        Ok(info) => info,
        Err(e) => {
            if let Some((filename, url, sha1)) = file_hint {
                (filename, url, sha1)
            } else if e.is_transient_server() {
                return Err(AppError::msg(
                    "Modrinth no responde (servidor saturado). Esperá 1–2 minutos e intentá de nuevo.",
                ));
            } else {
                return Err(e);
            }
        }
    };
    std::fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(&filename);
    net::download_all(
        client,
        vec![DownloadItem::new(dl, dest).with_sha1(sha1)],
        1,
        app,
        &format!("store-v-{version_id}"),
        &filename,
    )
    .await?;
    Ok(filename)
}

async fn fetch_version_file(
    client: &reqwest::Client,
    version_id: &str,
) -> AppResult<(String, String, Option<String>)> {
    let v: Value = net::fetch_json(client, &format!("{API}/version/{version_id}")).await?;
    file_from_version(&v, "mod")
}

/// Devuelve (filename, url, sha1) de la mejor version compatible.
async fn best_file(
    client: &reqwest::Client,
    project_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
) -> AppResult<(String, String, Option<String>)> {
    let version = fetch_best_version(client, project_id, project_type, mc, loader).await?;
    file_from_version(&version, project_type)
}

async fn fetch_best_version(
    client: &reqwest::Client,
    project_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
) -> AppResult<Value> {
    let mut url = format!(
        "{API}/project/{project_id}/version?game_versions={}",
        net::url_encode(&format!("[\"{mc}\"]"))
    );
    if let Some(q) = loaders_query(project_type, loader) {
        url.push_str(&format!("&loaders={}", net::url_encode(&q)));
    }
    let versions: Value = net::fetch_json(client, &url).await?;
    versions
        .as_array()
        .and_then(|arr| arr.first().cloned())
        .ok_or_else(|| AppError::msg("No hay version compatible con esta instancia"))
}

fn file_from_version(version: &Value, project_type: &str) -> AppResult<(String, String, Option<String>)> {
    let files = version["files"].as_array().cloned().unwrap_or_default();
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool().unwrap_or(false))
        .or_else(|| files.first())
        .ok_or_else(|| AppError::msg("La version no tiene archivos"))?;

    let filename = file["filename"].as_str().unwrap_or("download.jar").to_string();
    let dl = file["url"].as_str().ok_or_else(|| AppError::msg("Sin URL de descarga"))?.to_string();
    let sha1 = file["hashes"]["sha1"].as_str().map(String::from);
    Ok((filename, dl, sha1))
}

fn jar_already_present(dest_dir: &Path, filename: &str) -> bool {
    let target = dest_dir.join(filename);
    if target.is_file() {
        return true;
    }
    let stem = filename
        .trim_end_matches(".jar")
        .trim_end_matches(".disabled")
        .to_lowercase();
    dest_dir.read_dir().into_iter().flatten().flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_lowercase();
        (n.ends_with(".jar") || n.ends_with(".jar.disabled")) && n.contains(&stem)
    })
}

async fn install_recursive(
    app: &AppHandle,
    client: &reqwest::Client,
    project_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
    dest_dir: PathBuf,
    visiting: &mut HashSet<String>,
) -> AppResult<String> {
    let key = project_id.to_lowercase();
    if !visiting.insert(key) {
        return Ok(String::new());
    }

    let version = fetch_best_version(client, project_id, project_type, mc, loader).await?;

    if let Some(deps) = version["dependencies"].as_array() {
        for dep in deps {
            let dep_type = dep["dependency_type"].as_str().unwrap_or("");
            if dep_type != "required" && dep_type != "embedded" {
                continue;
            }
            if let Some(dep_project) = dep["project_id"].as_str() {
                let _ = Box::pin(install_recursive(
                    app,
                    client,
                    dep_project,
                    project_type,
                    mc,
                    loader,
                    dest_dir.clone(),
                    visiting,
                ))
                .await;
            }
        }
    }

    let (filename, url, sha1) = file_from_version(&version, project_type)?;
    std::fs::create_dir_all(&dest_dir)?;
    if jar_already_present(&dest_dir, &filename) {
        return Ok(filename);
    }
    let dest = dest_dir.join(&filename);
    net::download_all(
        client,
        vec![DownloadItem::new(url, dest).with_sha1(sha1)],
        1,
        app,
        &format!("store-{project_id}"),
        &filename,
    )
    .await?;
    Ok(filename)
}

/// Instala el contenido en `dest_dir`. Devuelve el filename instalado.
pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    project_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
    dest_dir: PathBuf,
) -> AppResult<String> {
    install_recursive(
        app,
        client,
        project_id,
        project_type,
        mc,
        loader,
        dest_dir,
        &mut HashSet::new(),
    )
    .await
}
