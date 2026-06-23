//! Hangar (PaperMC) — búsqueda e instalación de plugins.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const UA: &str = "ParaguacraftLauncher/2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HangarPlugin {
    pub slug: String,
    pub owner: String,
    pub name: String,
    pub description: String,
    pub downloads: u64,
    pub stars: u64,
}

pub async fn search(client: &reqwest::Client, query: &str) -> AppResult<Vec<HangarPlugin>> {
    let url = "https://hangar.papermc.io/api/v1/projects";
    let resp = client
        .get(url)
        .query(&[("q", query), ("limit", "16"), ("sort", "-stars")])
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Hangar: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::msg(format!("Hangar HTTP {}", resp.status())));
    }
    let data: serde_json::Value = resp.json().await?;
    let mut out = Vec::new();
    for p in data.get("result").and_then(|r| r.as_array()).into_iter().flatten() {
        let ns = p.get("namespace").unwrap_or(&serde_json::Value::Null);
        out.push(HangarPlugin {
            slug: ns.get("slug").and_then(|x| x.as_str()).unwrap_or("").into(),
            owner: ns.get("owner").and_then(|x| x.as_str()).unwrap_or("").into(),
            name: p.get("name").and_then(|x| x.as_str()).unwrap_or("").into(),
            description: p
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .chars()
                .take(140)
                .collect(),
            downloads: p
                .get("stats")
                .and_then(|s| s.get("downloads"))
                .and_then(|x| x.as_u64())
                .unwrap_or(0),
            stars: p
                .get("stats")
                .and_then(|s| s.get("stars"))
                .and_then(|x| x.as_u64())
                .unwrap_or(0),
        });
    }
    Ok(out)
}

pub async fn install_plugin(
    client: &reqwest::Client,
    dir: &Path,
    owner: &str,
    slug: &str,
    mc_version: &str,
    is_fabric: bool,
) -> AppResult<String> {
    let target = if is_fabric {
        dir.join("mods")
    } else {
        dir.join("plugins")
    };
    std::fs::create_dir_all(&target)?;

    if !mc_version.is_empty() && !is_fabric {
        let vc_url = format!("https://hangar.papermc.io/api/v1/projects/{owner}/{slug}/versions");
        let vc = client
            .get(&vc_url)
            .query(&[("platform", "PAPER"), ("platformVersion", mc_version)])
            .header("User-Agent", UA)
            .send()
            .await?;
        if vc.status().is_success() {
            let body: serde_json::Value = vc.json().await?;
            if body.get("result").and_then(|r| r.as_array()).map(|a| a.is_empty()).unwrap_or(true) {
                return Err(AppError::msg(format!(
                    "'{slug}' no tiene versión para MC {mc_version} en Hangar."
                )));
            }
        }
    }

    let latest_url = format!("https://hangar.papermc.io/api/v1/projects/{owner}/{slug}/latestrelease");
    let ver_resp = client
        .get(&latest_url)
        .header("User-Agent", UA)
        .send()
        .await?;
    if !ver_resp.status().is_success() {
        return Err(AppError::msg(format!("No se pudo obtener versión de {slug}")));
    }
    let ver_name = ver_resp.text().await?.trim().trim_matches('"').to_string();
    let platform = if is_fabric { "FABRIC" } else { "PAPER" };
    let dl_url = format!(
        "https://hangar.papermc.io/api/v1/projects/{owner}/{slug}/versions/{ver_name}/{platform}/download"
    );
    let bytes = client
        .get(&dl_url)
        .header("User-Agent", UA)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let fname = format!("{slug}.jar");
    let out = target.join(&fname);
    std::fs::write(&out, &bytes)?;
    Ok(fname)
}
