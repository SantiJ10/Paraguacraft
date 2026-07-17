//! Metadatos Modrinth (nombre, icono, autor) para contenido instalado.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde_json::Value;

use super::content::InstanceContentItem;
use crate::core::instances;
use crate::core::loaders;
use crate::error::AppResult;

const API: &str = "https://api.modrinth.com/v2";

#[derive(Default)]
struct ProjectMeta {
    title: String,
    author: String,
    icon_url: String,
    description: String,
}

pub async fn enrich(
    client: &reqwest::Client,
    instance_id: &str,
    base: &Path,
    items: &mut [InstanceContentItem],
) -> AppResult<()> {
    let meta = instances::ensure_meta(instance_id)?;
    let mc = meta.mc_version.clone();
    let loader = loaders::normalize(&meta.loader);

    let mut hashes: Vec<String> = items.iter().filter_map(|i| i.sha1.clone()).collect();
    hashes.sort();
    hashes.dedup();

    let mut hash_to_project: HashMap<String, String> = HashMap::new();
    let mut hash_version: HashMap<String, Value> = HashMap::new();

    if !hashes.is_empty() {
        let body = serde_json::json!({
            "hashes": hashes,
            "algorithm": "sha1"
        });
        if let Ok(resp) = client
            .post(format!("{API}/version_files"))
            .json(&body)
            .send()
            .await
        {
            if let Ok(map) = resp.json::<HashMap<String, Value>>().await {
                for (hash, ver) in map {
                    if let Some(pid) = ver["project_id"].as_str() {
                        hash_to_project.insert(hash.clone(), pid.to_string());
                    }
                    hash_version.insert(hash, ver);
                }
            }
        }
    }

    let project_ids: Vec<String> = hash_to_project.values().cloned().collect::<HashSet<_>>().into_iter().collect();
    let mut projects: HashMap<String, ProjectMeta> = HashMap::new();
    if !project_ids.is_empty() {
        let ids_json = serde_json::to_string(&project_ids).unwrap_or_else(|_| "[]".into());
        if let Ok(resp) = client
            .get(format!("{API}/projects"))
            .query(&[("ids", ids_json.as_str())])
            .send()
            .await
        {
            if let Ok(arr) = resp.json::<Vec<Value>>().await {
                for p in arr {
                    let id = p["id"].as_str().unwrap_or_default().to_string();
                    projects.insert(
                        id,
                        ProjectMeta {
                            title: p["title"].as_str().unwrap_or_default().to_string(),
                            author: p["author"].as_str().unwrap_or("Modrinth").to_string(),
                            icon_url: p["icon_url"].as_str().unwrap_or_default().to_string(),
                            description: p["description"].as_str().unwrap_or_default().to_string(),
                        },
                    );
                }
            }
        }
    }

    for item in items.iter_mut() {
        enrich_local(base, item);

        if let Some(ref hash) = item.sha1 {
            if let Some(pid) = hash_to_project.get(hash) {
                if let Some(pm) = projects.get(pid) {
                    if !pm.title.is_empty() {
                        item.display_name = Some(pm.title.clone());
                    }
                    if !pm.author.is_empty() {
                        item.author = Some(pm.author.clone());
                    }
                    if !pm.icon_url.is_empty() {
                        item.icon_url = Some(pm.icon_url.clone());
                    }
                    if !pm.description.is_empty() {
                        item.description = Some(truncate(&pm.description, 120));
                    }
                }
            }
            if let Some(ver) = hash_version.get(hash) {
                let (ok, msg) = check_compat(ver, &mc, &loader, &item.folder);
                item.compatible = Some(ok);
                if let Some(m) = msg {
                    item.compat_message = Some(m);
                }
            }
        }
    }
    Ok(())
}

fn enrich_local(base: &Path, item: &mut InstanceContentItem) {
    let path = base.join(&item.path.replace('/', std::path::MAIN_SEPARATOR_STR));
    if path.is_dir() {
        let pack_png = path.join("pack.png");
        if pack_png.is_file() {
            item.local_icon_path = Some(pack_png.to_string_lossy().into());
        }
        let mcmeta = path.join("pack.mcmeta");
        if mcmeta.is_file() {
            if let Ok(raw) = std::fs::read_to_string(&mcmeta) {
                if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                    let desc = v["pack"]["description"]
                        .as_str()
                        .map(String::from)
                        .or_else(|| v["pack"]["description"]["text"].as_str().map(String::from));
                    if let Some(d) = desc.filter(|s| !s.is_empty()) {
                        item.description = Some(truncate(&d, 120));
                    }
                }
            }
        }
    } else if item.folder == "resourcepacks" || item.folder == "shaderpacks" {
        if let Ok(file) = std::fs::File::open(&path) {
            if let Ok(mut zip) = zip::ZipArchive::new(file) {
                if let Ok(mut mcmeta) = zip.by_name("pack.mcmeta") {
                    let mut buf = String::new();
                    if std::io::Read::read_to_string(&mut mcmeta, &mut buf).is_ok() {
                        if let Ok(v) = serde_json::from_str::<Value>(&buf) {
                            let desc = v["pack"]["description"]
                                .as_str()
                                .map(String::from)
                                .or_else(|| v["pack"]["description"]["text"].as_str().map(String::from));
                            if let Some(d) = desc.filter(|s| !s.is_empty()) {
                                item.description = Some(truncate(&d, 120));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_compat(ver: &Value, mc: &str, loader: &str, folder: &str) -> (bool, Option<String>) {
    let versions: Vec<String> = ver["game_versions"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let loaders: Vec<String> = ver["loaders"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    if !versions.is_empty() && !versions.iter().any(|v| v == mc) {
        return (
            false,
            Some(format!("Versión del mod: {} (instancia {mc})", versions.join(", "))),
        );
    }

    if folder == "mods" && loader != "vanilla" && !loaders.is_empty() {
        let norm = loaders::normalize(loader);
        let ok = loaders.iter().any(|l| {
            l == &norm
                || (norm == "fabric-iris" && l == "fabric")
                || (norm == "paraguacraft-pvp" && (l == "forge" || l == "neoforge"))
        });
        if !ok {
            return (
                false,
                Some(format!("Loader del mod: {} (instancia {norm})", loaders.join(", "))),
            );
        }
    }
    (true, None)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    format!("{}…", s.chars().take(max).collect::<String>())
}
