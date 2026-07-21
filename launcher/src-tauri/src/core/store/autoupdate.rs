//! Auto-actualizacion de contenido (Modrinth `version_files/update`).
//!
//! Compara los `.jar` instalados (por SHA-1) con la ultima version compatible
//! (mismo mc + loader de la instancia) y reemplaza solo lo que cambio,
//! respetando hashes. No toca lo que ya esta al dia.

use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{json, Value};
use tauri::AppHandle;

use crate::core::instances;
use crate::core::loaders;
use crate::core::net::{self, sha1_hex, DownloadItem};
use crate::core::store::curseforge;
use crate::error::{AppError, AppResult};

const API: &str = "https://api.modrinth.com/v2";

fn jars_in_dir(dir: &PathBuf) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            // Respeta mods deshabilitados (*.jar.disabled): extension() solo ve "disabled".
            p.extension().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("jar")) == Some(true)
        })
        .collect()
}

fn jar_hashes(dir: &PathBuf) -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    for p in jars_in_dir(dir) {
        if let Ok(bytes) = std::fs::read(&p) {
            map.insert(sha1_hex(&bytes), p);
        }
    }
    map
}

fn jar_fingerprints(dir: &PathBuf) -> HashMap<u32, PathBuf> {
    let mut map = HashMap::new();
    for p in jars_in_dir(dir) {
        if let Ok(bytes) = std::fs::read(&p) {
            map.insert(curseforge::cf_fingerprint(&bytes), p);
        }
    }
    map
}

/// Actualiza mods/resourcepacks/shaders de una instancia (Modrinth por sha1 +
/// CurseForge por fingerprint murmur2). Devuelve cuantos archivos se actualizaron.
pub async fn update_instance(
    app: &AppHandle,
    client: &reqwest::Client,
    cf_key: &str,
    instance_id: &str,
) -> AppResult<u32> {
    let meta = if instance_id.starts_with("ext::") {
        instances::resolve_external_meta(instance_id)
            .ok_or_else(|| AppError::msg("Instancia no encontrada"))?
    } else {
        instances::ensure_meta(instance_id)?
    };
    let mc = meta.mc_version.clone();
    let loader = loaders::normalize(&meta.loader);

    let base = instances::game_dir_for(instance_id)
        .ok_or_else(|| AppError::msg("Instancia no encontrada"))?;
    let mut updated = 0u32;

    for sub in ["mods", "resourcepacks", "shaderpacks"] {
        let dir = base.join(sub);
        // Hashear todos los jars es I/O + CPU; nunca en el hilo del runtime async.
        let installed = tokio::task::spawn_blocking({
            let d = dir.clone();
            move || jar_hashes(&d)
        })
        .await
        .unwrap_or_default();
        if installed.is_empty() {
            continue;
        }
        let hashes: Vec<String> = installed.keys().cloned().collect();

        // Solo filtramos por loader en mods.
        let loaders_arr: Vec<String> = if sub == "mods" && loader != "vanilla" {
            vec![loader.clone()]
        } else {
            vec![]
        };
        let body = json!({
            "hashes": hashes,
            "algorithm": "sha1",
            "loaders": loaders_arr,
            "game_versions": [mc],
        });

        let resp = client
            .post(format!("{API}/version_files/update"))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let map: HashMap<String, Value> = resp.json().await?;

        let mut items = Vec::new();
        let mut to_remove = Vec::new();
        for (old_hash, version) in &map {
            let files = version["files"].as_array().cloned().unwrap_or_default();
            let file = files
                .iter()
                .find(|f| f["primary"].as_bool().unwrap_or(false))
                .or_else(|| files.first());
            let Some(file) = file else { continue };
            let new_sha1 = file["hashes"]["sha1"].as_str().unwrap_or_default();
            // Ya esta al dia.
            if new_sha1.eq_ignore_ascii_case(old_hash) {
                continue;
            }
            let Some(filename) = file["filename"].as_str() else { continue };
            let Some(url) = file["url"].as_str() else { continue };
            items.push(
                DownloadItem::new(url, dir.join(filename))
                    .with_sha1(Some(new_sha1.to_string())),
            );
            if let Some(old_path) = installed.get(old_hash) {
                // Evitar borrar si el nombre es el mismo (se sobrescribe).
                if old_path.file_name().and_then(|n| n.to_str()) != Some(filename) {
                    to_remove.push(old_path.clone());
                }
            }
            updated += 1;
        }

        if !items.is_empty() {
            net::download_all(client, items, 8, app, &format!("update-{sub}"), "Actualizando contenido")
                .await?;
            for old in to_remove {
                let _ = std::fs::remove_file(old);
            }
        }

        // Segunda pasada: jars que no matchean Modrinth pueden venir de CurseForge.
        // Se identifican por fingerprint (murmur2 modificado), no por nombre/hash sha1.
        if !cf_key.trim().is_empty() {
            updated += update_curseforge_dir(app, client, cf_key, &dir, sub, &mc, &loader).await;
        }
    }

    Ok(updated)
}

/// Actualiza los archivos de `dir` que CurseForge identifica por fingerprint.
/// No aborta el resto de la cola si un mod esta bloqueado o falla individualmente.
async fn update_curseforge_dir(
    app: &AppHandle,
    client: &reqwest::Client,
    cf_key: &str,
    dir: &PathBuf,
    sub: &str,
    mc: &str,
    loader: &str,
) -> u32 {
    let installed = tokio::task::spawn_blocking({
        let d = dir.clone();
        move || jar_fingerprints(&d)
    })
    .await
    .unwrap_or_default();
    if installed.is_empty() {
        return 0;
    }
    let fingerprints: Vec<u32> = installed.keys().copied().collect();
    let matches = match curseforge::check_fingerprints(client, cf_key, &fingerprints).await {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if matches.is_empty() {
        return 0;
    }

    let project_type = if sub == "mods" { "mod" } else { "resourcepack" };
    let mut items = Vec::new();
    let mut to_remove = Vec::new();
    let mut updated = 0u32;

    for (fingerprint, (mod_id, current_file)) in &matches {
        let Some(old_path) = installed.get(fingerprint) else { continue };
        let current_file_id = current_file["id"].as_u64();

        let files = match curseforge::list_files_raw(
            client,
            cf_key,
            &mod_id.to_string(),
            project_type,
            mc,
            loader,
        )
        .await
        {
            Ok(f) => f,
            Err(_) => continue,
        };
        // Se asume la respuesta de CurseForge ordenada del mas reciente al mas viejo.
        let Some(latest) = files.first() else { continue };
        if latest["isAvailable"].as_bool() == Some(false) {
            continue;
        }
        if latest["id"].as_u64() == current_file_id {
            continue; // Ya esta al dia.
        }
        let Some(filename) = latest["fileName"].as_str() else { continue };
        let url = curseforge::file_download_url(latest);
        if url.is_empty() {
            continue;
        }
        let sha1 = curseforge::file_sha1(latest);
        items.push(DownloadItem::new(url, dir.join(filename)).with_sha1(sha1));
        if old_path.file_name().and_then(|n| n.to_str()) != Some(filename) {
            to_remove.push(old_path.clone());
        }
        updated += 1;
    }

    if !items.is_empty() {
        if net::download_all(
            client,
            items,
            8,
            app,
            &format!("update-cf-{sub}"),
            "Actualizando contenido (CurseForge)",
        )
        .await
        .is_ok()
        {
            for old in to_remove {
                let _ = std::fs::remove_file(old);
            }
        } else {
            updated = 0;
        }
    }

    updated
}
