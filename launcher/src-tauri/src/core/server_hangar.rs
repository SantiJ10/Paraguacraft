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

    let platform = if is_fabric { "FABRIC" } else { "PAPER" };
    let mut matched_ver: Option<String> = None;

    if !mc_version.is_empty() {
        let mc_candidates: Vec<&str> = match mc_version {
            "1.8.9" => vec!["1.8.9", "1.8.8", "1.8"],
            "1.12.2" => vec!["1.12.2", "1.12"],
            other => vec![other],
        };
        let vc_url = format!("https://hangar.papermc.io/api/v1/projects/{owner}/{slug}/versions");
        for mc_cand in mc_candidates {
            let Ok(vc) = client
                .get(&vc_url)
                .query(&[("platform", platform), ("platformVersion", mc_cand)])
                .header("User-Agent", UA)
                .send()
                .await
            else {
                continue;
            };
            if vc.status().is_success() {
                if let Ok(body) = vc.json::<serde_json::Value>().await {
                    if let Some(first) = body
                        .get("result")
                        .and_then(|r| r.as_array())
                        .and_then(|a| a.first())
                    {
                        if let Some(name) = first.get("name").and_then(|n| n.as_str()) {
                            matched_ver = Some(name.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    if matched_ver.is_none() && mc_version.is_empty() {
        let latest_url =
            format!("https://hangar.papermc.io/api/v1/projects/{owner}/{slug}/latestrelease");
        if let Ok(ver_resp) = client.get(&latest_url).header("User-Agent", UA).send().await {
            if ver_resp.status().is_success() {
                if let Ok(txt) = ver_resp.text().await {
                    let v = txt.trim().trim_matches('"').to_string();
                    if !v.is_empty() {
                        matched_ver = Some(v);
                    }
                }
            }
        }
    }

    if let Some(ver_name) = matched_ver {
        let dl_url = format!(
            "https://hangar.papermc.io/api/v1/projects/{owner}/{slug}/versions/{ver_name}/{platform}/download"
        );
        if let Ok(resp) = client.get(&dl_url).header("User-Agent", UA).send().await {
            if resp.status().is_success() {
                if let Ok(bytes) = resp.bytes().await {
                    if bytes.len() > 1000 {
                        let fname = format!("{slug}.jar");
                        let out = target.join(&fname);
                        std::fs::write(&out, &bytes)?;
                        return Ok(fname);
                    }
                }
            }
        }
    }

    // Fallback a Modrinth si Hangar no publica versión compatible para este MC (p. ej. 1.8.9 / 1.12.2)
    let loaders: &[&str] = if is_fabric {
        &["fabric"]
    } else {
        &["paper", "spigot", "bukkit"]
    };
    let fname = format!("{slug}.jar");
    let out = target.join(&fname);
    crate::core::server_setup::download_modrinth_plugin_direct(
        client,
        &slug.to_lowercase(),
        loaders,
        mc_version,
        &out,
    )
    .await?;
    Ok(fname)
}
