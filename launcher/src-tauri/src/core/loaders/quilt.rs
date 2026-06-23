//! Provider Quilt (meta.quiltmc.org). Reusa la instalacion de perfil de Fabric.

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net;
use crate::error::AppResult;

use super::fabric::install_profile;

const META: &str = "https://meta.quiltmc.org/v3";

pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    let url = format!("{META}/versions/loader/{mc}");
    let arr: Value = match net::fetch_json(client, &url).await {
        Ok(v) => v,
        Err(_) => return Ok(vec![]),
    };
    let Some(list) = arr.as_array() else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    for item in list {
        if let Some(v) = item["loader"]["version"].as_str() {
            out.push(v.to_string());
        }
    }
    Ok(out)
}

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
