//! Provider Fabric (meta.fabricmc.net).

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::core::versions;
use crate::error::{AppError, AppResult};

const META: &str = "https://meta.fabricmc.net/v2";

/// Versiones de loader Fabric para `mc` (estables primero). Vacio = no soportado.
pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    let url = format!("{META}/versions/loader/{mc}");
    let arr: Value = net::fetch_json(client, &url).await?;
    let Some(list) = arr.as_array() else {
        return Ok(vec![]);
    };
    let mut stable = Vec::new();
    let mut unstable = Vec::new();
    for item in list {
        let loader = &item["loader"];
        if let Some(v) = loader["version"].as_str() {
            if loader["stable"].as_bool().unwrap_or(false) {
                stable.push(v.to_string());
            } else {
                unstable.push(v.to_string());
            }
        }
    }
    stable.extend(unstable);
    Ok(stable)
}

/// Instala el perfil Fabric y devuelve el version id a lanzar.
pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader_version: &str,
) -> AppResult<String> {
    let url = format!("{META}/versions/loader/{mc}/{loader_version}/profile/json");
    let profile: Value = net::fetch_json(client, &url).await?;
    install_profile(app, client, &profile).await
}

/// Guarda el JSON del perfil y descarga sus libraries (compartido con Quilt).
pub async fn install_profile(
    app: &AppHandle,
    client: &reqwest::Client,
    profile: &Value,
) -> AppResult<String> {
    let id = profile["id"]
        .as_str()
        .ok_or_else(|| AppError::msg("Perfil de loader sin id"))?
        .to_string();

    // 1) Guardar versions/<id>/<id>.json
    let json_path = versions::versions_dir().join(&id).join(format!("{id}.json"));
    if let Some(parent) = json_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&json_path, serde_json::to_vec_pretty(profile)?)?;

    // 2) Descargar libraries (estilo maven: name + url base).
    let libs_root = paths::default_minecraft_dir().join("libraries");
    let mut items = Vec::new();
    if let Some(libs) = profile["libraries"].as_array() {
        for lib in libs {
            let Some(name) = lib["name"].as_str() else { continue };
            let Some(rel) = versions::maven_to_path(name) else { continue };
            let rel_url = rel.to_string_lossy().replace('\\', "/");
            let base = lib["url"].as_str().unwrap_or("https://maven.fabricmc.net/");
            let base = if base.ends_with('/') { base.to_string() } else { format!("{base}/") };
            let full = format!("{base}{rel_url}");
            items.push(
                DownloadItem::new(full, libs_root.join(&rel))
                    .with_sha1(lib["sha1"].as_str().map(String::from)),
            );
        }
    }
    net::download_all(client, items, 8, app, "install-loader", "Librerias del loader").await?;

    Ok(id)
}
