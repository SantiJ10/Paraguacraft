//! Tienda CurseForge (api.curseforge.com/v1). Requiere API key (Regla 1: filtra
//! por gameVersion + modLoaderType exactos de la instancia).
//!
//! Contingencia Project Distribution: si el autor bloquea descargas de terceros,
//! devolvemos un error estructurado (`cf_distribution_blocked`) con la URL del
//! proyecto para descarga manual — la cola no se aborta.

use serde_json::Value;

use crate::core::net::{self, DownloadItem};
use crate::error::{AppError, AppResult, StructuredError};
use crate::models::{StoreItem, StoreVersion};

const API: &str = "https://api.curseforge.com/v1";
const GAME_ID: &str = "432"; // Minecraft

fn class_id(project_type: &str) -> Option<&'static str> {
    match project_type {
        "mod" => Some("6"),
        "resourcepack" => Some("12"),
        "shader" => Some("6552"),
        "modpack" => Some("4471"),
        "plugin" => Some("5"),
        "datapack" => Some("6945"),
        _ => None,
    }
}

fn loader_type(loader: &str) -> Option<&'static str> {
    match loader {
        "forge" => Some("1"),
        "fabric" | "fabric-iris" => Some("4"),
        "quilt" => Some("5"),
        "neoforge" => Some("6"),
        _ => None,
    }
}

fn loader_relevant(project_type: &str) -> bool {
    matches!(project_type, "mod" | "modpack" | "plugin")
}

fn project_page_url(slug: &str, class_id: u64) -> String {
    let section = match class_id {
        12 => "texture-packs",
        6552 => "shaders",
        4471 => "modpacks",
        6945 => "data-packs",
        _ => "mc-mods",
    };
    format!("https://www.curseforge.com/minecraft/{section}/{slug}")
}

fn distribution_blocked(name: &str, url: &str) -> AppError {
    AppError::structured(StructuredError {
        code: "cf_distribution_blocked".into(),
        message: format!(
            "CurseForge bloqueo la descarga de \"{name}\": el autor no permite descargas de terceros (Project Distribution)."
        ),
        project_url: Some(url.to_string()),
        project_name: Some(name.to_string()),
    })
}

async fn get_json(client: &reqwest::Client, url: &str, key: &str) -> AppResult<Value> {
    let resp = client
        .get(url)
        .header("x-api-key", key)
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json().await?)
}

fn mod_to_item(m: &Value, project_type: &str) -> StoreItem {
    let author = m["authors"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|a| a["name"].as_str())
        .unwrap_or_default()
        .to_string();
    let slug = m["slug"].as_str().unwrap_or_default().to_string();
    let class_id = m["classId"].as_u64().unwrap_or(6);
    StoreItem {
        id: m["id"].as_u64().map(|n| n.to_string()).unwrap_or_default(),
        slug: slug.clone(),
        title: m["name"].as_str().unwrap_or_default().to_string(),
        author,
        description: m["summary"].as_str().unwrap_or_default().to_string(),
        icon_url: m["logo"]["url"].as_str().unwrap_or_default().to_string(),
        downloads: m["downloadCount"].as_u64().unwrap_or(0),
        follows: m["thumbsUpCount"].as_u64().unwrap_or(0),
        project_type: project_type.to_string(),
        provider: "curseforge".into(),
        categories: m["categories"]
            .as_array()
            .map(|a| a.iter().filter_map(|c| c["name"].as_str().map(String::from)).collect())
            .unwrap_or_default(),
        project_url: if slug.is_empty() {
            None
        } else {
            Some(project_page_url(&slug, class_id))
        },
    }
}

pub async fn search(
    client: &reqwest::Client,
    key: &str,
    query: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
) -> AppResult<Vec<StoreItem>> {
    if key.trim().is_empty() {
        return Err(AppError::msg(
            "CurseForge requiere una API key. Configurala en .env (CURSEFORGE_API_KEY) o en Ajustes.",
        ));
    }
    let mut url = format!(
        "{API}/mods/search?gameId={GAME_ID}&searchFilter={}&pageSize=40&sortField=2&sortOrder=desc",
        net::url_encode(query)
    );
    if let Some(cid) = class_id(project_type) {
        url.push_str(&format!("&classId={cid}"));
    }
    if !mc.is_empty() {
        url.push_str(&format!("&gameVersion={}", net::url_encode(mc)));
    }
    if loader_relevant(project_type) {
        if let Some(lt) = loader_type(loader) {
            url.push_str(&format!("&modLoaderType={lt}"));
        }
    }
    let resp = get_json(client, &url, key).await?;
    let data = resp["data"].as_array().cloned().unwrap_or_default();
    Ok(data.iter().map(|m| mod_to_item(m, project_type)).collect())
}

fn file_to_version(f: &Value) -> StoreVersion {
    let game_versions: Vec<String> = f["gameVersions"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let loaders: Vec<String> = f["modLoaders"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    StoreVersion {
        id: f["id"].as_u64().map(|n| n.to_string()).unwrap_or_default(),
        name: f["displayName"].as_str().unwrap_or_default().to_string(),
        version_number: f["displayName"].as_str().unwrap_or_default().to_string(),
        filename: f["fileName"].as_str().unwrap_or("download.jar").to_string(),
        game_versions,
        loaders,
        published_at: f["fileDate"].as_str().unwrap_or("").to_string(),
    }
}

/// Lista archivos/versiones del mod compatibles con mc + loader.
pub async fn list_versions(
    client: &reqwest::Client,
    key: &str,
    mod_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
) -> AppResult<Vec<StoreVersion>> {
    if key.trim().is_empty() {
        return Err(AppError::msg(
            "CurseForge requiere una API key (.env o Ajustes).",
        ));
    }
    let mut url = format!("{API}/mods/{mod_id}/files?pageSize=50");
    if !mc.is_empty() {
        url.push_str(&format!("&gameVersion={}", net::url_encode(mc)));
    }
    if loader_relevant(project_type) {
        if let Some(lt) = loader_type(loader) {
            url.push_str(&format!("&modLoaderType={lt}"));
        }
    }
    let resp = get_json(client, &url, key).await?;
    let files = resp["data"].as_array().cloned().unwrap_or_default();
    Ok(files.iter().map(file_to_version).collect())
}

/// Todas las versiones/archivos del proyecto (sin filtro mc/loader).
pub async fn list_all_versions(
    client: &reqwest::Client,
    key: &str,
    mod_id: &str,
    project_type: &str,
) -> AppResult<Vec<StoreVersion>> {
    if key.trim().is_empty() {
        return Err(AppError::msg(
            "CurseForge requiere una API key (.env o Ajustes).",
        ));
    }
    let url = format!("{API}/mods/{mod_id}/files?pageSize=50");
    let resp = get_json(client, &url, key).await?;
    let files = resp["data"].as_array().cloned().unwrap_or_default();
    Ok(files
        .iter()
        .map(file_to_version)
        .filter(|v| {
            if project_type == "modpack" {
                v.filename.ends_with(".mrpack") || v.filename.ends_with(".zip")
            } else {
                true
            }
        })
        .collect())
}

/// Instala un archivo concreto de CurseForge por id.
pub async fn install_file_id(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    key: &str,
    mod_id: &str,
    file_id: &str,
    dest_dir: std::path::PathBuf,
) -> AppResult<String> {
    if key.trim().is_empty() {
        return Err(AppError::msg(
            "CurseForge requiere una API key (.env o Ajustes).",
        ));
    }

    let mod_resp = get_json(client, &format!("{API}/mods/{mod_id}"), key).await?;
    let meta = &mod_resp["data"];
    let slug = meta["slug"].as_str().unwrap_or_default();
    let name = meta["name"].as_str().unwrap_or("Proyecto").to_string();
    let class_id = meta["classId"].as_u64().unwrap_or(6);
    let project_url = project_page_url(slug, class_id);
    let allow_dist = meta["allowModDistribution"].as_bool().unwrap_or(true);

    if !allow_dist {
        return Err(distribution_blocked(&name, &project_url));
    }

    let file_resp = get_json(client, &format!("{API}/mods/{mod_id}/files/{file_id}"), key).await?;
    let file = &file_resp["data"];

    if file["isAvailable"].as_bool() == Some(false) {
        return Err(distribution_blocked(&name, &project_url));
    }

    let filename = file["fileName"].as_str().unwrap_or("download.jar").to_string();
    let fid = file["id"].as_u64().unwrap_or(0);
    let dl = match file["downloadUrl"].as_str() {
        Some(u) if !u.is_empty() => u.to_string(),
        _ => edge_url(fid, &filename),
    };

    if file["downloadUrl"].as_str().filter(|u| !u.is_empty()).is_none() {
        if let Err(e) = probe_download_url(client, &dl).await {
            let msg = e.to_string();
            if msg.contains("distribution_blocked") || msg.contains("403") {
                return Err(distribution_blocked(&name, &project_url));
            }
        }
    }

    let sha1 = file["hashes"]
        .as_array()
        .and_then(|a| a.iter().find(|h| h["algo"].as_u64() == Some(1)))
        .and_then(|h| h["value"].as_str())
        .map(String::from);

    std::fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(&filename);

    match net::download_all(
        client,
        vec![DownloadItem::new(dl.clone(), dest).with_sha1(sha1)],
        1,
        app,
        &format!("store-{mod_id}-{file_id}"),
        &filename,
    )
    .await
    {
        Ok(()) => Ok(filename),
        Err(e) => {
            let low = e.to_string().to_lowercase();
            if low.contains("403") || low.contains("forbidden") || low.contains("401") {
                Err(distribution_blocked(&name, &project_url))
            } else {
                Err(e)
            }
        }
    }
}

fn edge_url(file_id: u64, filename: &str) -> String {
    let s = file_id.to_string();
    let (a, b) = if s.len() >= 4 {
        (&s[..s.len() - 3], &s[s.len() - 3..])
    } else {
        (s.as_str(), "")
    };
    let b = b.trim_start_matches('0');
    let b = if b.is_empty() { "0" } else { b };
    format!("https://edge.forgecdn.net/files/{a}/{b}/{}", net::url_encode(filename))
}

/// Comprueba si la URL de descarga responde (403 = distribution blocked).
async fn probe_download_url(client: &reqwest::Client, url: &str) -> AppResult<()> {
    let resp = client.head(url).send().await?;
    let status = resp.status();
    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::msg("distribution_blocked"));
    }
    if status.is_client_error() {
        return Err(AppError::msg(format!("HTTP {status} al verificar descarga")));
    }
    Ok(())
}

pub async fn install(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    key: &str,
    mod_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
    dest_dir: std::path::PathBuf,
) -> AppResult<String> {
    if key.trim().is_empty() {
        return Err(AppError::msg(
            "CurseForge requiere una API key (.env o Ajustes).",
        ));
    }

    // Metadata del proyecto (slug, nombre, allowModDistribution).
    let mod_resp = get_json(client, &format!("{API}/mods/{mod_id}"), key).await?;
    let meta = &mod_resp["data"];
    let slug = meta["slug"].as_str().unwrap_or_default();
    let name = meta["name"].as_str().unwrap_or("Proyecto").to_string();
    let class_id = meta["classId"].as_u64().unwrap_or(6);
    let project_url = project_page_url(slug, class_id);
    let allow_dist = meta["allowModDistribution"].as_bool().unwrap_or(true);

    if !allow_dist {
        return Err(distribution_blocked(&name, &project_url));
    }

    let mut url = format!("{API}/mods/{mod_id}/files?pageSize=20");
    if !mc.is_empty() {
        url.push_str(&format!("&gameVersion={}", net::url_encode(mc)));
    }
    if loader_relevant(project_type) {
        if let Some(lt) = loader_type(loader) {
            url.push_str(&format!("&modLoaderType={lt}"));
        }
    }
    let resp = get_json(client, &url, key).await?;
    let files = resp["data"].as_array().cloned().unwrap_or_default();
    let file = files.first().ok_or_else(|| {
        AppError::msg("CurseForge: no hay archivos compatibles con esta instancia")
    })?;

    if file["isAvailable"].as_bool() == Some(false) {
        return Err(distribution_blocked(&name, &project_url));
    }

    let filename = file["fileName"].as_str().unwrap_or("download.jar").to_string();
    let file_id = file["id"].as_u64().unwrap_or(0);
    let dl = match file["downloadUrl"].as_str() {
        Some(u) if !u.is_empty() => u.to_string(),
        _ => edge_url(file_id, &filename),
    };

    // Si no hay downloadUrl oficial, el edge CDN suele bloquear distribution.
    if file["downloadUrl"].as_str().filter(|u| !u.is_empty()).is_none() {
        if let Err(e) = probe_download_url(client, &dl).await {
            let msg = e.to_string();
            if msg.contains("distribution_blocked") || msg.contains("403") {
                return Err(distribution_blocked(&name, &project_url));
            }
        }
    }

    let sha1 = file["hashes"]
        .as_array()
        .and_then(|a| a.iter().find(|h| h["algo"].as_u64() == Some(1)))
        .and_then(|h| h["value"].as_str())
        .map(String::from);

    std::fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(&filename);

    match net::download_all(
        client,
        vec![DownloadItem::new(dl.clone(), dest).with_sha1(sha1)],
        1,
        app,
        &format!("store-{mod_id}"),
        &filename,
    )
    .await
    {
        Ok(()) => Ok(filename),
        Err(e) => {
            let low = e.to_string().to_lowercase();
            if low.contains("403") || low.contains("forbidden") || low.contains("401") {
                Err(distribution_blocked(&name, &project_url))
            } else {
                Err(e)
            }
        }
    }
}
