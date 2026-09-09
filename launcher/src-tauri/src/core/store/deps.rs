//! Resolución recursiva de dependencias REQUIRED (Modrinth + CurseForge).
//! Tope de profundidad, ciclos, remap Quilt↔Fabric.

use std::collections::HashSet;
use std::path::Path;

use serde_json::Value;

use super::curseforge;
use super::mod_index;
use super::modrinth;
use super::jar_already_present;
use crate::error::AppResult;
use crate::models::StoreDependency;

const MAX_DEPTH: u32 = 8;

/// Modrinth: Quilted Fabric API ↔ Fabric API; Quilt Kotlin ↔ Fabric Language Kotlin.
const MR_QFAPI: &str = "qvIfYCYJ";
const MR_FAPI: &str = "P7dR8mSH";
const MR_QKL: &str = "lwVhp9o5";
const MR_FLK: &str = "Ha28R6CL";

/// CurseForge numeric ids (strings).
const CF_QFAPI: &str = "634179";
const CF_FAPI: &str = "306612";
const CF_QKL: &str = "720410";
const CF_FLK: &str = "308769";

/// Candidatos de proyecto en orden (override + fallback).
pub fn remap_project_ids(provider: &str, loader: &str, project_id: &str) -> Vec<String> {
    let l = loader.trim().to_lowercase();
    let quilt = l == "quilt";
    let fabricish = l == "fabric"
        || l == "fabric-iris"
        || l.contains("pvp-modern")
        || (l.contains("optimized") && !l.contains("neoforge"));
    let pid = project_id.trim();
    if provider == "modrinth" {
        if quilt {
            if pid.eq_ignore_ascii_case(MR_FAPI) {
                return vec![MR_QFAPI.into(), MR_FAPI.into()];
            }
            if pid.eq_ignore_ascii_case(MR_FLK) {
                return vec![MR_QKL.into(), MR_FLK.into()];
            }
        }
        if fabricish {
            if pid.eq_ignore_ascii_case(MR_QFAPI) {
                return vec![MR_FAPI.into()];
            }
            if pid.eq_ignore_ascii_case(MR_QKL) {
                return vec![MR_FLK.into()];
            }
        }
    }
    if provider == "curseforge" {
        if quilt {
            if pid == CF_FAPI {
                return vec![CF_QFAPI.into(), CF_FAPI.into()];
            }
            if pid == CF_FLK {
                return vec![CF_QKL.into(), CF_FLK.into()];
            }
        }
        if fabricish {
            if pid == CF_QFAPI {
                return vec![CF_FAPI.into()];
            }
            if pid == CF_QKL {
                return vec![CF_FLK.into()];
            }
        }
    }
    vec![pid.to_string()]
}

fn already_in_dest(dest: Option<&Path>, filename: Option<&str>, project_id: &str) -> bool {
    let Some(dir) = dest else {
        return false;
    };
    if mod_index::project_installed(dir, project_id) {
        return true;
    }
    if let Some(f) = filename {
        if jar_already_present(dir, f) {
            return true;
        }
    }
    false
}

fn push_dep(out: &mut Vec<StoreDependency>, dep: StoreDependency) {
    if let Some(existing) = out.iter_mut().find(|d| d.project_id == dep.project_id) {
        for p in dep.required_by {
            if !existing.required_by.contains(&p) {
                existing.required_by.push(p);
            }
        }
        if existing.version_id.is_none() {
            existing.version_id = dep.version_id;
        }
        if existing.filename.is_none() {
            existing.filename = dep.filename;
        }
        existing.already_installed = existing.already_installed || dep.already_installed;
        return;
    }
    out.push(dep);
}

/// Árbol REQUIRED (sin incluir el proyecto raíz).
pub async fn resolve_required(
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    project_id: &str,
    version_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
    dest_dir: Option<&Path>,
) -> AppResult<Vec<StoreDependency>> {
    if project_type != "mod" && project_type != "plugin" {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut visiting = HashSet::new();
    visiting.insert(project_id.to_lowercase());
    match provider {
        "modrinth" => {
            walk_modrinth(
                client,
                version_id,
                mc,
                loader,
                dest_dir,
                0,
                "",
                &mut visiting,
                &mut out,
            )
            .await?;
        }
        "curseforge" => {
            walk_curseforge(
                client,
                cf_key,
                project_id,
                version_id,
                project_type,
                mc,
                loader,
                dest_dir,
                0,
                "",
                &mut visiting,
                &mut out,
            )
            .await?;
        }
        _ => {}
    }
    Ok(out)
}

async fn walk_modrinth(
    client: &reqwest::Client,
    version_id: &str,
    mc: &str,
    loader: &str,
    dest_dir: Option<&Path>,
    depth: u32,
    required_by: &str,
    visiting: &mut HashSet<String>,
    out: &mut Vec<StoreDependency>,
) -> AppResult<()> {
    if depth > MAX_DEPTH || version_id.is_empty() {
        return Ok(());
    }
    let version: Value = match crate::core::net::fetch_json(
        client,
        &format!("https://api.modrinth.com/v2/version/{version_id}"),
    )
    .await
    {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    let deps = version["dependencies"].as_array().cloned().unwrap_or_default();
    let parent_title = version["name"].as_str().unwrap_or(required_by).to_string();
    let parent_label = if required_by.is_empty() {
        parent_title
    } else {
        required_by.to_string()
    };

    for dep in deps {
        let dep_type = dep["dependency_type"].as_str().unwrap_or("");
        if dep_type != "required" && dep_type != "embedded" {
            continue;
        }
        let Some(raw_pid) = dep["project_id"].as_str() else {
            continue;
        };
        let candidates = remap_project_ids("modrinth", loader, raw_pid);
        let mut resolved: Option<(String, String, Option<String>, String, String)> = None;
        for cand in &candidates {
            if visiting.contains(&cand.to_lowercase()) {
                continue;
            }
            let pinned = dep["version_id"].as_str().filter(|s| !s.is_empty());
            let version_json = if let Some(vid) = pinned {
                crate::core::net::fetch_json::<Value>(
                    client,
                    &format!("https://api.modrinth.com/v2/version/{vid}"),
                )
                .await
                .ok()
            } else {
                None
            };
            let version_json = match version_json {
                Some(v) if version_matches_instance(&v, mc, loader) => Some(v),
                _ => modrinth::fetch_best_version(client, cand, "mod", mc, loader)
                    .await
                    .ok(),
            };
            let Some(v) = version_json else {
                continue;
            };
            let vid = v["id"].as_str().unwrap_or_default().to_string();
            let filename = modrinth::file_from_version(&v, "mod")
                .ok()
                .map(|(f, _, _)| f);
            let (title, icon) = modrinth::fetch_project_brief(client, cand)
                .await
                .unwrap_or_else(|_| (cand.clone(), String::new()));
            resolved = Some((
                cand.clone(),
                vid,
                filename,
                title,
                icon,
            ));
            break;
        }
        let Some((pid, vid, filename, title, icon)) = resolved else {
            continue;
        };
        if !visiting.insert(pid.to_lowercase()) {
            continue;
        }
        let already = already_in_dest(dest_dir, filename.as_deref(), &pid);
        push_dep(
            out,
            StoreDependency {
                project_id: pid.clone(),
                version_id: Some(vid.clone()),
                title: title.clone(),
                icon_url: icon,
                dependency_type: dep_type.to_string(),
                already_installed: already,
                required_by: if parent_label.is_empty() {
                    Vec::new()
                } else {
                    vec![parent_label.clone()]
                },
                filename: filename.clone(),
            },
        );
        Box::pin(walk_modrinth(
            client,
            &vid,
            mc,
            loader,
            dest_dir,
            depth + 1,
            &title,
            visiting,
            out,
        ))
        .await?;
    }
    Ok(())
}

fn version_matches_instance(v: &Value, mc: &str, loader: &str) -> bool {
    let gvs = v["game_versions"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .any(|x| x == mc)
        })
        .unwrap_or(true);
    if !mc.is_empty() && !gvs {
        return false;
    }
    if loader.is_empty() || loader == "vanilla" {
        return true;
    }
    v["loaders"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .any(|l| l.eq_ignore_ascii_case(loader))
        })
        .unwrap_or(true)
}

async fn walk_curseforge(
    client: &reqwest::Client,
    key: &str,
    project_id: &str,
    file_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
    dest_dir: Option<&Path>,
    depth: u32,
    required_by: &str,
    visiting: &mut HashSet<String>,
    out: &mut Vec<StoreDependency>,
) -> AppResult<()> {
    if depth > MAX_DEPTH || key.trim().is_empty() {
        return Ok(());
    }
    let file = match curseforge::get_file_metadata(client, key, project_id, file_id).await {
        Ok(f) => f,
        Err(_) => return Ok(()),
    };
    let parent_label = if required_by.is_empty() {
        file["displayName"]
            .as_str()
            .or(file["fileName"].as_str())
            .unwrap_or(project_id)
            .to_string()
    } else {
        required_by.to_string()
    };
    let deps = file["dependencies"].as_array().cloned().unwrap_or_default();
    for dep in deps {
        if dep["relationType"].as_u64().unwrap_or(0) != 3 {
            continue;
        }
        let Some(raw) = dep["modId"].as_u64().map(|n| n.to_string()) else {
            continue;
        };
        let candidates = remap_project_ids("curseforge", loader, &raw);
        let mut resolved: Option<(String, String, Option<String>, String, String)> = None;
        for cand in &candidates {
            if visiting.contains(&cand.to_lowercase()) {
                continue;
            }
            let files = curseforge::list_files_raw(client, key, cand, project_type, mc, loader)
                .await
                .unwrap_or_default();
            let Some(best) = files.first() else {
                continue;
            };
            let vid = best["id"].as_u64().map(|n| n.to_string()).unwrap_or_default();
            if vid.is_empty() {
                continue;
            }
            let filename = best["fileName"].as_str().map(String::from);
            let (title, icon) = match curseforge::mod_name(client, key, cand).await {
                Ok(n) => {
                    let icon = curseforge::mod_icon_url(client, key, cand)
                        .await
                        .unwrap_or_default();
                    (n, icon)
                }
                Err(_) => (cand.clone(), String::new()),
            };
            resolved = Some((cand.clone(), vid, filename, title, icon));
            break;
        }
        let Some((pid, vid, filename, title, icon)) = resolved else {
            continue;
        };
        if !visiting.insert(pid.to_lowercase()) {
            continue;
        }
        let already = already_in_dest(dest_dir, filename.as_deref(), &pid);
        push_dep(
            out,
            StoreDependency {
                project_id: pid.clone(),
                version_id: Some(vid.clone()),
                title: title.clone(),
                icon_url: icon,
                dependency_type: "required".into(),
                already_installed: already,
                required_by: vec![parent_label.clone()],
                filename: filename.clone(),
            },
        );
        Box::pin(walk_curseforge(
            client,
            key,
            &pid,
            &vid,
            project_type,
            mc,
            loader,
            dest_dir,
            depth + 1,
            &title,
            visiting,
            out,
        ))
        .await?;
    }
    Ok(())
}

/// Instala las deps listadas (falla si una requerida no se puede bajar).
pub async fn install_listed(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    mc: &str,
    loader: &str,
    dest_dir: &Path,
    deps: &[StoreDependency],
) -> AppResult<Vec<String>> {
    let mut installed = Vec::new();
    for dep in deps {
        if dep.already_installed {
            continue;
        }
        if let Some(f) = dep.filename.as_deref() {
            if jar_already_present(dest_dir, f) {
                continue;
            }
        }
        if mod_index::project_installed(dest_dir, &dep.project_id) {
            continue;
        }
        let Some(vid) = dep.version_id.as_deref() else {
            continue;
        };
        let filename = match provider {
            "modrinth" => {
                modrinth::install_version_id(app, client, vid, dest_dir.to_path_buf(), None).await?
            }
            "curseforge" => {
                curseforge::install_file_id(
                    app,
                    client,
                    cf_key,
                    &dep.project_id,
                    vid,
                    dest_dir.to_path_buf(),
                )
                .await?
            }
            _ => continue,
        };
        let _ = mod_index::record(
            dest_dir,
            provider,
            &dep.project_id,
            vid,
            &filename,
            None,
            Some(mc),
            &[loader.to_string()],
            &[],
        );
        installed.push(filename);
    }
    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quilt_prefers_qfapi_over_fabric_api() {
        let ids = remap_project_ids("modrinth", "quilt", MR_FAPI);
        assert_eq!(ids[0], MR_QFAPI);
        assert_eq!(ids[1], MR_FAPI);
    }

    #[test]
    fn fabric_maps_qfapi_to_fapi() {
        let ids = remap_project_ids("modrinth", "fabric", MR_QFAPI);
        assert_eq!(ids, vec![MR_FAPI]);
    }

    #[test]
    fn cf_quilt_maps_fabric_api() {
        let ids = remap_project_ids("curseforge", "quilt", CF_FAPI);
        assert_eq!(ids[0], CF_QFAPI);
    }

    #[test]
    fn optimized_maps_qfapi_like_fabric() {
        let ids = remap_project_ids("modrinth", "paraguacraft-optimized", MR_QFAPI);
        assert_eq!(ids, vec![MR_FAPI]);
    }
}
