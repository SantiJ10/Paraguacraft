//! Loaders (compatibilidad ESTRICTA - Regla 1).
//!
//! Cada provider consulta la API oficial del loader y devuelve SOLO las
//! versiones que existen para la version de Minecraft elegida. Si la lista
//! viene vacia, ese loader no es instalable para esa version (la UI lo oculta).
//!
//! Providers: Fabric, Quilt, Forge, NeoForge, OptiFine.

pub mod fabric;
pub mod fabric_iris;
pub mod forge;
pub mod neoforge;
pub mod optifine;
pub mod pvp;
pub mod quilt;

use std::path::PathBuf;
use std::process::Command;

use tauri::AppHandle;
use crate::error::{AppError, AppResult};
use crate::models::LoaderInfo;

/// Normaliza el id de loader desde la UI.
/// `fabric-iris` es un preset **aparte** de `fabric` (no se fusionan).
pub fn normalize(loader: &str) -> String {
    let l = loader.trim().to_lowercase().replace(' ', "-").replace('+', "-");
    if l.contains("fabric-iris") || l.contains("fabric_iris") || (l.contains("fabric") && l.contains("iris")) {
        "fabric-iris".into()
    } else if l.contains("paraguacraft-pvp")
        || l.contains("paraguacraft_pvp")
        || (l.contains("paraguacraft") && l.contains("pvp"))
        || l == "pvp"
    {
        "paraguacraft-pvp".into()
    } else if l.contains("neoforge") {
        "neoforge".into()
    } else if l.contains("quilt") {
        "quilt".into()
    } else if l.contains("fabric") {
        "fabric".into()
    } else if l.contains("optifine") {
        "optifine".into()
    } else if l.contains("forge") {
        "forge".into()
    } else {
        "vanilla".into()
    }
}

/// Loader efectivo para filtrar la tienda (fabric-iris usa mods de Fabric).
pub fn store_loader(loader: &str) -> String {
    match normalize(loader).as_str() {
        "fabric-iris" => "fabric".into(),
        "paraguacraft-pvp" => "forge".into(),
        other => other.into(),
    }
}

/// Compara loaders para tienda/instalación (fabric-iris ≡ fabric en Modrinth/CF).
pub fn loaders_compatible(a: &str, b: &str) -> bool {
    store_loader(a) == store_loader(b)
}

/// Versiones exactas del loader disponibles para `mc` (vacio = no compatible).
pub async fn loader_versions(
    client: &reqwest::Client,
    loader: &str,
    mc: &str,
) -> AppResult<Vec<String>> {
    match normalize(loader).as_str() {
        "vanilla" => Ok(vec![]),
        "fabric" => fabric::versions(client, mc).await,
        "fabric-iris" => fabric_iris::versions(client, mc).await,
        "paraguacraft-pvp" => pvp::versions(client, mc).await,
        "quilt" => quilt::versions(client, mc).await,
        "forge" => forge::versions(client, mc).await,
        "neoforge" => neoforge::versions(client, mc).await,
        "optifine" => optifine::versions(client, mc).await,
        other => Err(AppError::msg(format!("Loader desconocido: {other}"))),
    }
}

/// Lista los loaders realmente disponibles para `mc`, consultando cada API.
/// Vanilla siempre esta. Las consultas corren en paralelo.
pub async fn available_loaders(client: &reqwest::Client, mc: &str) -> AppResult<Vec<LoaderInfo>> {
    let (fab, qui, forge_v, neo, opti) = tokio::join!(
        fabric::versions(client, mc),
        quilt::versions(client, mc),
        forge::versions(client, mc),
        neoforge::versions(client, mc),
        optifine::versions(client, mc),
    );

    let mut out = vec![LoaderInfo {
        id: "vanilla".into(),
        name: "Vanilla".into(),
        description: "Minecraft oficial sin mods.".into(),
        versions: vec![],
        recommended: None,
    }];

    let fabric_ok = fab.as_ref().ok().filter(|v| !v.is_empty()).cloned();

    let mut push = |id: &str, name: &str, desc: &str, res: AppResult<Vec<String>>| {
        if let Ok(versions) = res {
            if !versions.is_empty() {
                let recommended = versions.first().cloned();
                out.push(LoaderInfo {
                    id: id.into(),
                    name: name.into(),
                    description: desc.into(),
                    versions,
                    recommended,
                });
            }
        }
    };

    push("fabric", "Fabric", "Loader ligero y rapido, ideal para optimizacion.", fab);
    push("quilt", "Quilt", "Fork de Fabric con features extra.", qui);
    push("forge", "Forge", "El loader clasico, maxima compatibilidad de mods.", forge_v);
    push("neoforge", "NeoForge", "Sucesor moderno de Forge.", neo);
    push("optifine", "OptiFine", "Shaders y opciones graficas (standalone).", opti);
    // Preset aparte: solo si Fabric existe para esta MC.
    if let Some(fi_vers) = fabric_ok {
        let recommended = fi_vers.first().cloned();
        out.push(LoaderInfo {
            id: "fabric-iris".into(),
            name: "Fabric + Iris".into(),
            description: "Fabric con Sodium, Iris, Lithium y mods de optimizacion.".into(),
            versions: fi_vers,
            recommended,
        });
    }

    if mc == pvp::MC {
        if let Ok(pvp_vers) = pvp::versions(client, mc).await {
            if !pvp_vers.is_empty() {
                let recommended = pvp_vers.first().cloned();
                out.push(LoaderInfo {
                    id: "paraguacraft-pvp".into(),
                    name: "Paraguacraft PvP".into(),
                    description: "Forge 1.8.9 + OptiFine HD U M5 + cliente competitivo Paraguacraft.".into(),
                    versions: pvp_vers,
                    recommended,
                });
            }
        }
    }

    Ok(out)
}

/// Extrae la version del loader desde un `version id` instalado (ej. `fabric-loader-0.19.3-1.21.11`).
pub fn loader_version_from_version_id(loader: &str, version_id: &str, mc: &str) -> Option<String> {
    let kind = normalize(loader);
    let mc_suffix = format!("-{mc}");
    match kind.as_str() {
        "fabric" | "fabric-iris" => version_id
            .strip_prefix("fabric-loader-")?
            .strip_suffix(&mc_suffix)
            .map(String::from),
        "quilt" => version_id
            .strip_prefix("quilt-loader-")?
            .strip_suffix(&mc_suffix)
            .map(String::from),
        "forge" | "paraguacraft-pvp" => version_id
            .strip_prefix(&format!("{mc}-forge-"))
            .map(String::from),
        "neoforge" => version_id
            .strip_prefix(&format!("{mc}-neoforge-"))
            .map(String::from),
        _ => None,
    }
}

fn cmp_semver(a: &str, b: &str) -> std::cmp::Ordering {
    let pa: Vec<u32> = a
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    let pb: Vec<u32> = b
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    for i in 0..pa.len().max(pb.len()) {
        let da = *pa.get(i).unwrap_or(&0);
        let db = *pb.get(i).unwrap_or(&0);
        match da.cmp(&db) {
            std::cmp::Ordering::Equal => {}
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

/// Busca un perfil ya instalado en `versions/` para mc + loader (instancias legacy Python).
pub fn find_version_id_for_loader(mc: &str, loader: &str) -> Option<String> {
    let kind = normalize(loader);
    let dir = crate::core::versions::versions_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return None;
    };
    let mc_suffix = format!("-{mc}");
    let mut candidates: Vec<(String, String)> = Vec::new();
    for e in entries.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if crate::core::versions::read_local_json(&name).is_none() {
            continue;
        }
        let loader_ver = match kind.as_str() {
            "fabric" | "fabric-iris" if name.starts_with("fabric-loader-") && name.ends_with(&mc_suffix) => {
                loader_version_from_version_id(&kind, &name, mc)
            }
            "quilt" if name.starts_with("quilt-loader-") && name.ends_with(&mc_suffix) => {
                loader_version_from_version_id(&kind, &name, mc)
            }
            "forge" | "paraguacraft-pvp" if name.starts_with(&format!("{mc}-forge-")) => {
                loader_version_from_version_id(&kind, &name, mc)
            }
            "neoforge" if name.starts_with(&format!("{mc}-neoforge-")) => {
                loader_version_from_version_id(&kind, &name, mc)
            }
            "optifine" if name.contains(mc) && name.to_lowercase().contains("optifine") => Some(String::new()),
            "vanilla" if name == mc => Some(String::new()),
            _ => None,
        };
        if loader_ver.is_some() {
            candidates.push((name, loader_ver.unwrap_or_default()));
        }
    }
    candidates.sort_by(|a, b| cmp_semver(&a.1, &b.1));
    candidates.pop().map(|(id, _)| id)
}

/// Resuelve la version del loader cuando falta (instancias importadas del launcher Python).
pub async fn resolve_loader_version(
    client: &reqwest::Client,
    mc: &str,
    loader: &str,
    provided: &str,
) -> AppResult<String> {
    let trimmed = provided.trim();
    if !trimmed.is_empty() {
        return Ok(trimmed.to_string());
    }
    if let Some(vid) = find_version_id_for_loader(mc, loader) {
        if let Some(v) = loader_version_from_version_id(loader, &vid, mc) {
            if !v.is_empty() {
                return Ok(v);
            }
        }
    }
    let vers = loader_versions(client, loader, mc).await?;
    vers.first().cloned().ok_or_else(|| {
        AppError::msg(format!(
            "No hay version de {loader} para Minecraft {mc}. Elegi una en Configuracion de la instancia."
        ))
    })
}

/// Instala el loader pedido y devuelve el `version id` a lanzar.
/// Para todos se asegura primero la base vanilla.
pub async fn install_loader(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader: &str,
    loader_version: &str,
) -> AppResult<String> {
    let kind = normalize(loader);
    if kind == "vanilla" {
        crate::core::versions::install_vanilla(app, client, mc).await?;
        return Ok(mc.to_string());
    }
    // Base vanilla siempre necesaria (inheritsFrom / merge).
    crate::core::versions::install_vanilla(app, client, mc).await?;

    let loader_version = resolve_loader_version(client, mc, loader, loader_version).await?;

    match kind.as_str() {
        "fabric" => fabric::install(app, client, mc, &loader_version).await,
        "fabric-iris" => {
            let id = fabric_iris::install_fabric_profile(app, client, mc, &loader_version).await?;
            Ok(id)
        }
        "paraguacraft-pvp" => pvp::install(app, client, mc, &loader_version).await,
        "quilt" => quilt::install(app, client, mc, &loader_version).await,
        "forge" => forge::install(app, client, mc, &loader_version).await,
        "neoforge" => neoforge::install(app, client, mc, &loader_version).await,
        "optifine" => optifine::install(app, client, mc, &loader_version).await,
        other => Err(AppError::msg(format!("Loader desconocido: {other}"))),
    }
}

// ── Helpers compartidos por los providers ──────────────────────────────────

/// Extrae todos los `<version>...</version>` de un `maven-metadata.xml`.
pub fn parse_maven_versions(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<version>") {
        let after = &rest[start + 9..];
        if let Some(end) = after.find("</version>") {
            out.push(after[..end].trim().to_string());
            rest = &after[end + 10..];
        } else {
            break;
        }
    }
    out
}

/// Elige Java para instaladores de loaders según la versión de MC (descarga Temurin si falta).
pub async fn installer_java(
    app: &AppHandle,
    state: &crate::state::AppState,
    mc_version: &str,
) -> AppResult<PathBuf> {
    crate::core::java::resolve::ensure_installer_java(app, state, mc_version).await
}

/// Ejecuta un instalador headless y espera a que termine (en spawn_blocking).
pub async fn run_installer(java: PathBuf, args: Vec<String>) -> AppResult<()> {
    let out = tauri::async_runtime::spawn_blocking(move || {
        let mut cmd = Command::new(&java);
        cmd.args(&args);
        hide_console(&mut cmd);
        cmd.output()
    })
    .await
    .map_err(|e| AppError::msg(format!("Fallo al lanzar instalador: {e}")))??;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(AppError::msg(format!(
            "El instalador fallo (codigo {:?}): {}",
            out.status.code(),
            stderr.chars().take(500).collect::<String>()
        )));
    }
    Ok(())
}

/// Busca en `versions/` la primera carpeta cuyo nombre contenga todos los `needles`.
pub fn find_installed_version_id(needles: &[&str]) -> Option<String> {
    let dir = crate::core::versions::versions_dir();
    let entries = std::fs::read_dir(dir).ok()?;
    let mut best: Option<String> = None;
    for e in entries.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        let low = name.to_lowercase();
        if needles.iter().all(|n| low.contains(&n.to_lowercase())) {
            // Preferimos el match mas largo (mas especifico).
            if best.as_ref().map(|b| name.len() > b.len()).unwrap_or(true) {
                best = Some(name);
            }
        }
    }
    best
}

#[cfg(target_os = "windows")]
fn hide_console(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_console(_cmd: &mut Command) {}
