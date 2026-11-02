//! Metadatos Modrinth y manifiestos locales (.jar/.zip) para contenido instalado.

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

    let project_ids: Vec<String> = hash_to_project
        .values()
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
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
                if let Some(vn) = ver["version_number"].as_str().filter(|s| !s.is_empty()) {
                    item.version = Some(vn.to_string());
                }
                if item.mod_loader.is_none() {
                    if let Some(arr) = ver["loaders"].as_array() {
                        let ls: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                        if !ls.is_empty() {
                            item.mod_loader = Some(ls.join(" / "));
                        }
                    }
                }
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
            if item.icon_url.is_none() {
                item.icon_url = crate::core::instances::icons::file_as_data_url(&pack_png);
            }
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
    } else if item.folder == "mods" {
        enrich_local_mod_jar(&path, item);
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
                if item.icon_url.is_none() {
                    if let Some(data) = extract_zip_png_data(&mut zip, "pack.png") {
                        item.icon_url = Some(data);
                    }
                }
            }
        }
    }
}

fn enrich_local_mod_jar(path: &Path, item: &mut InstanceContentItem) {
    let Ok(file) = std::fs::File::open(path) else {
        return;
    };
    let Ok(mut zip) = zip::ZipArchive::new(file) else {
        return;
    };

    let mut icon_candidate: Option<String> = None;

    // 1. Fabric (fabric.mod.json)
    if let Some(raw) = read_zip_text(&mut zip, "fabric.mod.json") {
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            item.mod_loader = Some("Fabric".into());
            if let Some(n) = v["name"].as_str().filter(|s| !s.trim().is_empty()) {
                item.display_name = Some(n.trim().to_string());
            }
            if let Some(ver) = v["version"].as_str().filter(|s| !s.trim().is_empty()) {
                item.version = Some(ver.trim().to_string());
            }
            if let Some(desc) = v["description"].as_str().filter(|s| !s.trim().is_empty()) {
                item.description = Some(truncate(desc.trim(), 120));
            }
            if let Some(arr) = v["authors"].as_array() {
                let names: Vec<String> = arr
                    .iter()
                    .filter_map(|a| {
                        a.as_str()
                            .map(String::from)
                            .or_else(|| a["name"].as_str().map(String::from))
                    })
                    .collect();
                if !names.is_empty() {
                    item.author = Some(names.join(", "));
                }
            }
            icon_candidate = v["icon"]
                .as_str()
                .map(String::from)
                .or_else(|| {
                    v["icon"]
                        .as_object()
                        .and_then(|obj| obj.values().filter_map(|x| x.as_str()).last().map(String::from))
                });
        }
    }

    // 2. Quilt (quilt.mod.json)
    if item.display_name.is_none() {
        if let Some(raw) = read_zip_text(&mut zip, "quilt.mod.json") {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                item.mod_loader = Some("Quilt".into());
                let ql = &v["quilt_loader"];
                let md = &ql["metadata"];
                if let Some(n) = md["name"].as_str().filter(|s| !s.trim().is_empty()) {
                    item.display_name = Some(n.trim().to_string());
                }
                if let Some(ver) = ql["version"].as_str().filter(|s| !s.trim().is_empty()) {
                    item.version = Some(ver.trim().to_string());
                }
                if let Some(desc) = md["description"].as_str().filter(|s| !s.trim().is_empty()) {
                    item.description = Some(truncate(desc.trim(), 120));
                }
                if let Some(obj) = md["contributors"].as_object() {
                    let names: Vec<String> = obj.keys().cloned().collect();
                    if !names.is_empty() {
                        item.author = Some(names.join(", "));
                    }
                }
                if icon_candidate.is_none() {
                    icon_candidate = md["icon"].as_str().map(String::from);
                }
            }
        }
    }

    // 3. NeoForge / Forge moderno (META-INF/neoforge.mods.toml | META-INF/mods.toml)
    if item.display_name.is_none() {
        for (entry_name, loader_label) in [
            ("META-INF/neoforge.mods.toml", "NeoForge"),
            ("META-INF/mods.toml", "Forge"),
        ] {
            if let Some(raw) = read_zip_text(&mut zip, entry_name) {
                item.mod_loader = Some(loader_label.into());
                if let Some(n) = extract_toml_field(&raw, "displayName") {
                    item.display_name = Some(n);
                }
                if let Some(ver) = extract_toml_field(&raw, "version") {
                    if ver.contains("${file.jarVersion}") {
                        if let Some(mf_ver) = read_manifest_version(&mut zip) {
                            item.version = Some(mf_ver);
                        }
                    } else if !ver.starts_with("${") {
                        item.version = Some(ver);
                    }
                }
                if let Some(auth) = extract_toml_field(&raw, "authors") {
                    item.author = Some(auth);
                }
                if let Some(desc) = extract_toml_field(&raw, "description") {
                    let clean = desc
                        .lines()
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if !clean.is_empty() {
                        item.description = Some(truncate(&clean, 120));
                    }
                }
                if icon_candidate.is_none() {
                    icon_candidate = extract_toml_field(&raw, "logoFile");
                }
                break;
            }
        }
    }

    // 4. Legacy Forge 1.7.10 / 1.8.9 / 1.12.2 (mcmod.info)
    if item.display_name.is_none() {
        if let Some(raw) = read_zip_text(&mut zip, "mcmod.info") {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                let first = v
                    .as_array()
                    .and_then(|a| a.first())
                    .or_else(|| v["modList"].as_array().and_then(|a| a.first()));
                if let Some(m) = first {
                    item.mod_loader = Some("Forge".into());
                    if let Some(n) = m["name"].as_str().filter(|s| !s.trim().is_empty()) {
                        item.display_name = Some(n.trim().to_string());
                    }
                    if let Some(ver) = m["version"]
                        .as_str()
                        .filter(|s| !s.trim().is_empty() && !s.starts_with("${"))
                    {
                        item.version = Some(ver.trim().to_string());
                    }
                    if let Some(desc) = m["description"].as_str().filter(|s| !s.trim().is_empty()) {
                        item.description = Some(truncate(desc.trim(), 120));
                    }
                    let authors = m["authorList"]
                        .as_array()
                        .or_else(|| m["authors"].as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(String::from))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if !authors.is_empty() {
                        item.author = Some(authors.join(", "));
                    }
                    if icon_candidate.is_none() {
                        icon_candidate = m["logoFile"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(String::from);
                    }
                }
            }
        }
    }

    if item.icon_url.is_none() {
        let mut cands = Vec::new();
        if let Some(c) = icon_candidate {
            let trimmed = c.trim_start_matches('/').to_string();
            if !trimmed.is_empty() {
                cands.push(trimmed);
            }
        }
        cands.push("icon.png".into());
        cands.push("logo.png".into());
        cands.push("pack.png".into());
        for cand in cands {
            if let Some(data) = extract_zip_png_data(&mut zip, &cand) {
                item.icon_url = Some(data);
                break;
            }
        }
    }
}

fn read_zip_text(zip: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Option<String> {
    let mut entry = zip.by_name(name).ok()?;
    if entry.size() > 512_000 {
        return None;
    }
    let mut buf = String::new();
    std::io::Read::read_to_string(&mut entry, &mut buf).ok()?;
    Some(buf)
}

fn read_manifest_version(zip: &mut zip::ZipArchive<std::fs::File>) -> Option<String> {
    let raw = read_zip_text(zip, "META-INF/MANIFEST.MF")?;
    for line in raw.lines() {
        let t = line.trim();
        if let Some(rest) = t
            .strip_prefix("Implementation-Version:")
            .or_else(|| t.strip_prefix("Specification-Version:"))
        {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn extract_toml_field(raw: &str, key: &str) -> Option<String> {
    let scope = raw.find("[[mods]]").map(|idx| &raw[idx..]).unwrap_or(raw);
    parse_toml_key_in_scope(scope, key).or_else(|| parse_toml_key_in_scope(raw, key))
}

fn parse_toml_key_in_scope(scope: &str, key: &str) -> Option<String> {
    let mut rest = scope;
    while !rest.is_empty() {
        let (line, next) = match rest.split_once('\n') {
            Some((l, n)) => (l, n),
            None => (rest, ""),
        };
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            if let Some(after_key) = trimmed.strip_prefix(key) {
                let after_ws = after_key.trim_start();
                if let Some(rhs) = after_ws.strip_prefix('=') {
                    let rhs = rhs.trim();
                    if let Some(multi) = rhs.strip_prefix("'''") {
                        if let Some((inside, _)) = multi.split_once("'''") {
                            let v = inside.trim();
                            if !v.is_empty() {
                                return Some(v.to_string());
                            }
                        } else if let Some((inside, _)) = next.split_once("'''") {
                            let combined = format!("{multi}\n{inside}");
                            let v = combined.trim();
                            if !v.is_empty() {
                                return Some(v.to_string());
                            }
                        }
                    } else if let Some(multi) = rhs.strip_prefix("\"\"\"") {
                        if let Some((inside, _)) = multi.split_once("\"\"\"") {
                            let v = inside.trim();
                            if !v.is_empty() {
                                return Some(v.to_string());
                            }
                        } else if let Some((inside, _)) = next.split_once("\"\"\"") {
                            let combined = format!("{multi}\n{inside}");
                            let v = combined.trim();
                            if !v.is_empty() {
                                return Some(v.to_string());
                            }
                        }
                    } else if let Some(q) = rhs.strip_prefix('"') {
                        if let Some((val, _)) = q.split_once('"') {
                            let v = val.trim();
                            if !v.is_empty() {
                                return Some(v.to_string());
                            }
                        }
                    } else if let Some(q) = rhs.strip_prefix('\'') {
                        if let Some((val, _)) = q.split_once('\'') {
                            let v = val.trim();
                            if !v.is_empty() {
                                return Some(v.to_string());
                            }
                        }
                    }
                }
            }
        }
        rest = next;
    }
    None
}

fn extract_zip_png_data(zip: &mut zip::ZipArchive<std::fs::File>, entry_path: &str) -> Option<String> {
    let mut entry = zip.by_name(entry_path).ok()?;
    if entry.size() > 1_500_000 {
        return None;
    }
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut buf).ok()?;
    if buf.len() < 32 {
        return None;
    }
    use base64::Engine;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(buf)
    ))
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

    let norm = loaders::normalize(loader);
    let store = loaders::store_loader_for(loader, mc);

    // Shaders: Iris (Fabric) vs OptiFine (Forge legacy) según el backend real del pack.
    if folder == "shaderpacks" {
        let shader_ok = loaders.is_empty()
            || loaders.iter().any(|l| {
                l == "vanilla"
                    || l == "canvas"
                    || l == &store
                    || (store == "fabric" && (l == "iris" || l == "fabric"))
                    || (store == "forge" && (l == "optifine" || l == "forge"))
                    || (store == "neoforge" && (l == "iris" || l == "neoforge" || l == "optifine"))
                    || (store == "optifine" && (l == "optifine" || l == "forge"))
            });
        if !shader_ok {
            return (
                false,
                Some(format!(
                    "Shader para {} (esta instancia usa {store})",
                    loaders.join(", ")
                )),
            );
        }
        // Iris/OptiFine suelen publicar builds multi-MC; no exigir match exacto de versión.
        let soft_mc = loaders
            .iter()
            .any(|l| l == "iris" || l == "optifine" || l == "canvas");
        if !versions.is_empty() && !versions.iter().any(|v| v == mc) && !soft_mc {
            return (
                false,
                Some(format!("Versión del shader: {} (instancia {mc})", versions.join(", "))),
            );
        }
        return (true, None);
    }

    // Resource packs: solo versión MC si Modrinth la declara.
    if folder == "resourcepacks" {
        if !versions.is_empty() && !versions.iter().any(|v| v == mc) {
            return (
                false,
                Some(format!("Versión del pack: {} (instancia {mc})", versions.join(", "))),
            );
        }
        return (true, None);
    }

    if !versions.is_empty() && !versions.iter().any(|v| v == mc) {
        return (
            false,
            Some(format!("Versión del mod: {} (instancia {mc})", versions.join(", "))),
        );
    }

    if folder == "mods" && loader != "vanilla" && !loaders.is_empty() {
        let ok = loaders.iter().any(|l| {
            l == &norm
                || l == &store
                || (norm == "fabric-iris" && l == "fabric")
                || (norm == "paraguacraft-pvp-modern" && l == "fabric")
                || (norm == "paraguacraft-pvp" && (l == "forge" || l == "neoforge"))
                || (norm == "paraguacraft-optimized"
                    && (l == &store || (store == "forge" && l == "optifine")))
                || (norm == "paraguacraft-optimized-forge" && l == "forge")
                || (norm == "paraguacraft-optimized-neoforge"
                    && (l == "neoforge" || l == "forge"))
        });
        if !ok {
            return (
                false,
                Some(format!(
                    "Loader del mod: {} (esta instancia usa {store})",
                    loaders.join(", ")
                )),
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
