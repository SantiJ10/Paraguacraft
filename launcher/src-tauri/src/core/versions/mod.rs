//! Versiones de Minecraft: manifest oficial de Mojang + instalacion vanilla.
//!
//! - Lista todas las versiones (release / snapshot / old_beta / old_alpha) del
//!   `version_manifest_v2`, con cache en disco (6 h de frescura).
//! - Instala una version vanilla (client jar + libraries con reglas de SO +
//!   assets) de forma asincronica y verificada (SHA-1) usando `core::net`.

use std::path::PathBuf;
use std::time::SystemTime;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::java;
use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::MinecraftVersion;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_BASE: &str = "https://resources.download.minecraft.net";

fn cache_file() -> PathBuf {
    let dir = paths::data_dir().join("cache");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("version_manifest_v2.json")
}

/// Nombre de SO tal como lo usan los `rules` de Mojang.
pub fn os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

/// Descarga (o reusa de cache) el manifest completo.
pub async fn fetch_manifest(client: &reqwest::Client) -> AppResult<Value> {
    let cache = cache_file();
    if let Ok(meta) = std::fs::metadata(&cache) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = SystemTime::now().duration_since(modified) {
                if age.as_secs() < 6 * 3600 {
                    if let Some(v) = crate::config::read_json::<Value>(&cache) {
                        return Ok(v);
                    }
                }
            }
        }
    }
    let bytes = net::fetch_bytes(client, MANIFEST_URL).await?;
    let v: Value = serde_json::from_slice(&bytes)?;
    let _ = std::fs::write(&cache, &bytes);
    Ok(v)
}

fn version_json_path(id: &str) -> PathBuf {
    paths::default_minecraft_dir().join("versions").join(id).join(format!("{id}.json"))
}

/// Lista todas las versiones con su canal y si estan instaladas.
pub async fn list_versions(client: &reqwest::Client) -> AppResult<Vec<MinecraftVersion>> {
    let manifest = fetch_manifest(client).await?;
    let arr = manifest["versions"].as_array().cloned().unwrap_or_default();
    let mut out = Vec::with_capacity(arr.len());
    for v in arr {
        let id = v["id"].as_str().unwrap_or_default().to_string();
        if id.is_empty() {
            continue;
        }
        let channel = v["type"].as_str().unwrap_or("release").to_string();
        let release_date = v["releaseTime"].as_str().unwrap_or_default().to_string();
        let installed = version_json_path(&id).is_file();
        out.push(MinecraftVersion { id, channel, release_date, installed });
    }
    Ok(out)
}

/// Asegura el JSON de la version (lo baja del manifest si falta) y lo devuelve.
pub async fn ensure_version_json(client: &reqwest::Client, id: &str) -> AppResult<Value> {
    let path = version_json_path(id);
    if let Some(v) = crate::config::read_json::<Value>(&path) {
        return Ok(v);
    }
    let manifest = fetch_manifest(client).await?;
    let entry = manifest["versions"]
        .as_array()
        .and_then(|arr| arr.iter().find(|v| v["id"].as_str() == Some(id)))
        .ok_or_else(|| AppError::msg(format!("Version {id} no existe en el manifest")))?;
    let url = entry["url"].as_str().ok_or_else(|| AppError::msg("URL de version ausente"))?;
    let bytes = net::fetch_bytes(client, url).await?;
    let v: Value = serde_json::from_slice(&bytes)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &bytes)?;
    Ok(v)
}

/// Evalua las `rules` de una libreria/argumento para el SO actual.
pub fn rules_allow(rules: &Value) -> bool {
    let Some(arr) = rules.as_array() else {
        return true;
    };
    let mut allowed = arr.is_empty();
    for rule in arr {
        let action_allow = rule["action"].as_str() == Some("allow");
        // Filtro por SO si esta presente.
        if let Some(os) = rule.get("os").and_then(|o| o.get("name")).and_then(|n| n.as_str()) {
            if os != os_name() {
                continue; // la regla no aplica a este SO
            }
        }
        // (Ignoramos features para el caso comun.)
        allowed = action_allow;
    }
    allowed
}

/// Resuelve la clave de natives (`natives-windows`, etc.) sustituyendo `${arch}`.
fn natives_key(lib: &Value) -> Option<String> {
    let key = lib.get("natives")?.get(os_name())?.as_str()?;
    let arch = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
    Some(key.replace("${arch}", arch))
}

/// Junta las libraries (artifact + natives) que aplican al SO actual.
/// Devuelve (items_de_descarga).
pub fn collect_library_items(version_json: &Value) -> Vec<DownloadItem> {
    let libs_root = paths::default_minecraft_dir().join("libraries");
    let mut items = Vec::new();
    let Some(libs) = version_json["libraries"].as_array() else {
        return items;
    };
    for lib in libs {
        if let Some(rules) = lib.get("rules") {
            if !rules_allow(rules) {
                continue;
            }
        }
        let downloads = &lib["downloads"];
        // Artefacto principal.
        if let Some(artifact) = downloads.get("artifact") {
            if let (Some(path), Some(url)) =
                (artifact["path"].as_str(), artifact["url"].as_str())
            {
                if !url.is_empty() {
                    items.push(
                        DownloadItem::new(url, libs_root.join(path))
                            .with_sha1(artifact["sha1"].as_str().map(String::from)),
                    );
                }
            }
        }
        // Natives del SO actual.
        if let Some(key) = natives_key(lib) {
            if let Some(classifier) = downloads.get("classifiers").and_then(|c| c.get(&key)) {
                if let (Some(path), Some(url)) =
                    (classifier["path"].as_str(), classifier["url"].as_str())
                {
                    items.push(
                        DownloadItem::new(url, libs_root.join(path))
                            .with_sha1(classifier["sha1"].as_str().map(String::from)),
                    );
                }
            }
        }
    }
    items
}

/// Construye los items de descarga de assets a partir del index.
async fn collect_asset_items(
    client: &reqwest::Client,
    version_json: &Value,
) -> AppResult<Vec<DownloadItem>> {
    let assets_root = paths::default_minecraft_dir().join("assets");
    let index = &version_json["assetIndex"];
    let (Some(asset_id), Some(url)) = (index["id"].as_str(), index["url"].as_str()) else {
        return Ok(Vec::new());
    };
    let index_path = assets_root.join("indexes").join(format!("{asset_id}.json"));
    let bytes = net::fetch_bytes(client, url).await?;
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&index_path, &bytes)?;

    let index_json: Value = serde_json::from_slice(&bytes)?;
    let mut items = Vec::new();
    if let Some(objects) = index_json["objects"].as_object() {
        for (_, obj) in objects {
            let Some(hash) = obj["hash"].as_str() else { continue };
            let sub = &hash[0..2];
            let dest = assets_root.join("objects").join(sub).join(hash);
            let url = format!("{RESOURCES_BASE}/{sub}/{hash}");
            items.push(DownloadItem::new(url, dest).with_sha1(Some(hash.to_string())));
        }
    }
    Ok(items)
}

fn client_jar_path(id: &str) -> PathBuf {
    paths::default_minecraft_dir().join("versions").join(id).join(format!("{id}.jar"))
}

/// Instala una version vanilla completa (idempotente; reusa lo ya descargado).
pub async fn install_vanilla(
    app: &AppHandle,
    client: &reqwest::Client,
    id: &str,
) -> AppResult<()> {
    let group = format!("install-{id}");
    let version_json = ensure_version_json(client, id).await?;
    let conc = net::concurrency_from_settings();

    let mut jar_items = Vec::new();
    if let Some(cl) = version_json["downloads"].get("client") {
        if let Some(url) = cl["url"].as_str() {
            jar_items.push(
                DownloadItem::new(url, client_jar_path(id))
                    .with_sha1(cl["sha1"].as_str().map(String::from)),
            );
        }
    }

    let libs = collect_library_items(&version_json);
    let assets = collect_asset_items(client, &version_json).await?;

    let client_j = client.clone();
    let app_j = app.clone();
    let group_j = group.clone();
    let id_j = id.to_string();
    let jar_f = async move {
        if jar_items.is_empty() {
            return Ok(());
        }
        net::download_all(
            &client_j,
            jar_items,
            4,
            &app_j,
            &group_j,
            &format!("Minecraft {id_j}"),
        )
        .await
    };

    let client_l = client.clone();
    let app_l = app.clone();
    let group_l = group.clone();
    let id_l = id.to_string();
    let libs_f = async move {
        net::download_all(
            &client_l,
            libs,
            conc,
            &app_l,
            &group_l,
            &format!("Librerias {id_l}"),
        )
        .await
    };

    let client_a = client.clone();
    let app_a = app.clone();
    let id_a = id.to_string();
    let assets_f = async move {
        net::download_all(
            &client_a,
            assets,
            conc,
            &app_a,
            &group,
            &format!("Assets {id_a}"),
        )
        .await
    };

    tokio::try_join!(jar_f, libs_f, assets_f)?;

    if let Some(comp) = java::mojang::component_from_version_id(id) {
        let _ = java::mojang::ensure_runtime(app, client, &comp).await;
    } else if java::required_for_mc(id) == 8 {
        // 1.8.x no trae javaVersion en el JSON; precargar jre-legacy oficial.
        let _ = java::mojang::ensure_runtime(app, client, "jre-legacy").await;
    }

    Ok(())
}

/// Ruta del jar de una version (para classpath del launcher).
pub fn jar_path(id: &str) -> PathBuf {
    client_jar_path(id)
}

/// Ruta del JSON de una version (para el launcher).
#[allow(dead_code)]
pub fn json_path(id: &str) -> PathBuf {
    version_json_path(id)
}

/// Lee el JSON de una version ya instalada del disco.
pub fn read_local_json(id: &str) -> Option<Value> {
    crate::config::read_json::<Value>(&version_json_path(id))
}

/// Une la lista de libraries de un perfil (para classpath), devolviendo rutas absolutas.
#[allow(dead_code)]
pub fn library_paths(version_json: &Value) -> Vec<PathBuf> {
    let libs_root = paths::default_minecraft_dir().join("libraries");
    let mut out = Vec::new();
    if let Some(libs) = version_json["libraries"].as_array() {
        for lib in libs {
            if let Some(rules) = lib.get("rules") {
                if !rules_allow(rules) {
                    continue;
                }
            }
            if let Some(path) = lib["downloads"]["artifact"]["path"].as_str() {
                out.push(libs_root.join(path));
            } else if let Some(name) = lib["name"].as_str() {
                // Libreria estilo maven (fabric/forge): derivar ruta del coordinate.
                if let Some(rel) = maven_to_path(name) {
                    out.push(libs_root.join(rel));
                }
            }
        }
    }
    out
}

/// `group:artifact:version[:classifier]` -> `group/artifact/version/artifact-version[-classifier].jar`.
pub fn maven_to_path(coord: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = coord.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = parts.get(3);
    let file = match classifier {
        Some(c) => format!("{artifact}-{version}-{c}.jar"),
        None => format!("{artifact}-{version}.jar"),
    };
    let mut p = PathBuf::from(group);
    p.push(artifact);
    p.push(version);
    p.push(file);
    Some(p)
}

/// Directorio `versions/` de `.minecraft`.
pub fn versions_dir() -> PathBuf {
    paths::default_minecraft_dir().join("versions")
}

/// `.minecraft` base.
#[allow(dead_code)]
pub fn mc_dir() -> PathBuf {
    paths::default_minecraft_dir()
}
