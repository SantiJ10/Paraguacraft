//! Runtime Java oficial de Mojang (`.minecraft/runtime/`).
//! Espejo de `minecraft_launcher_lib.runtime` / `_asegurar_java_runtime` del Python.

use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::core::versions;
use crate::core::java::{is_compatible, verify::verify as verify_java};
use crate::error::{AppError, AppResult};

const JVM_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

pub fn jvm_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86") {
            "windows-x86"
        } else {
            "windows-x64"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "mac-os-arm64"
        } else {
            "mac-os"
        }
    } else if cfg!(target_arch = "x86") {
        "linux-i386"
    } else {
        "linux"
    }
}

/// Componentes Mojang a probar según versión MC (como `_java_exe_para_version`).
pub fn component_candidates(mc_version: &str) -> Vec<&'static str> {
    let parts: Vec<&str> = mc_version.split('.').collect();
    let major: u32 = parts.first().and_then(|p| p.parse().ok()).unwrap_or(1);
    if major >= 26 {
        return vec![
            "java-runtime-epsilon",
            "java-runtime-delta",
            "java-runtime-gamma",
        ];
    }
    if major != 1 {
        return vec!["java-runtime-gamma", "java-runtime-delta"];
    }
    let minor: u32 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0);
    if minor >= 21 || (minor == 20 && parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0) >= 5) {
        vec![
            "java-runtime-delta",
            "java-runtime-loom",
            "java-runtime-gamma",
        ]
    } else if minor >= 17 {
        vec![
            "java-runtime-gamma",
            "java-runtime-beta",
            "java-runtime-delta",
        ]
    } else {
        vec!["jre-legacy", "java-runtime-gamma", "java-runtime-beta"]
    }
}

/// Lee `javaVersion.component` del JSON de versión (sigue `inheritsFrom`).
pub fn component_from_version_id(version_id: &str) -> Option<String> {
    let mut id = version_id.to_string();
    for _ in 0..16 {
        let json = versions::read_local_json(&id)?;
        if let Some(comp) = json
            .get("javaVersion")
            .and_then(|jv| jv.get("component"))
            .and_then(|c| c.as_str())
        {
            return Some(comp.to_string());
        }
        id = json.get("inheritsFrom")?.as_str()?.to_string();
    }
    None
}

fn runtime_base(component: &str) -> PathBuf {
    paths::default_minecraft_dir()
        .join("runtime")
        .join(component)
        .join(jvm_platform())
        .join(component)
}

/// Ruta al `java(w)` del runtime Mojang si ya está instalado.
pub fn find_executable(component: &str) -> Option<PathBuf> {
    let base = runtime_base(component);
    let bin = base.join("bin");
    let javaw = bin.join("javaw.exe");
    if javaw.is_file() {
        return Some(javaw);
    }
    let java = bin.join("java.exe");
    if java.is_file() {
        return Some(java);
    }
    let unix = bin.join("java");
    if unix.is_file() {
        return Some(unix);
    }
    // macOS bundle layout
    let mac = base
        .join("jre.bundle/Contents/Home/bin/java");
    if mac.is_file() {
        return Some(mac);
    }
    None
}

/// Instala el runtime Mojang si falta; devuelve ruta al ejecutable.
pub async fn ensure_runtime(
    app: &AppHandle,
    http: &reqwest::Client,
    component: &str,
) -> AppResult<PathBuf> {
    if let Some(p) = find_executable(component) {
        return Ok(p);
    }
    install_runtime(app, http, component).await?;
    find_executable(component).ok_or_else(|| {
        AppError::msg(format!(
            "Runtime Mojang '{component}' instalado pero no se encontró java.exe"
        ))
    })
}

async fn install_runtime(
    app: &AppHandle,
    http: &reqwest::Client,
    component: &str,
) -> AppResult<()> {
    let manifest: Value = net::fetch_json(http, JVM_MANIFEST_URL).await?;
    let platform = jvm_platform();
    let entry = manifest
        .get(platform)
        .and_then(|p| p.get(component))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| {
            AppError::msg(format!(
                "Runtime Java '{component}' no disponible para {platform}"
            ))
        })?;

    let manifest_url = entry
        .get("manifest")
        .and_then(|m| m.get("url"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| AppError::msg("Manifest de runtime sin URL"))?;

    let platform_manifest: Value = net::fetch_json(http, manifest_url).await?;
    let files = platform_manifest
        .get("files")
        .and_then(|f| f.as_object())
        .ok_or_else(|| AppError::msg("Manifest de runtime inválido"))?;

    let base = runtime_base(component);
    std::fs::create_dir_all(&base)?;

    let mut items = Vec::new();
    for (rel, spec) in files {
        if spec.get("type").and_then(|t| t.as_str()) != Some("file") {
            continue;
        }
        let Some(downloads) = spec.get("downloads") else {
            continue;
        };
        let dl = downloads
            .get("raw")
            .or_else(|| downloads.get("lzma"))
            .ok_or_else(|| AppError::msg(format!("Sin URL de descarga para {rel}")))?;
        let url = dl
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| AppError::msg(format!("URL inválida para {rel}")))?;
        let sha1 = dl.get("sha1").and_then(|s| s.as_str()).map(String::from);
        items.push(DownloadItem::new(url, base.join(rel)).with_sha1(sha1));
    }

    let conc = net::concurrency_from_settings();
    net::download_all(
        http,
        items,
        conc,
        app,
        &format!("jvm-{component}"),
        &format!("Java Mojang ({component})"),
    )
    .await?;

    Ok(())
}

/// Busca el mejor runtime Mojang ya instalado para una versión de MC (compatible).
pub fn find_for_mc(mc_version: &str, version_id: Option<&str>) -> Option<PathBuf> {
    if let Some(vid) = version_id {
        if let Some(comp) = component_from_version_id(vid) {
            if let Some(p) = find_executable(&comp) {
                if runtime_compatible(&p, mc_version) {
                    return Some(p);
                }
            }
        }
    }
    for comp in component_candidates(mc_version) {
        if let Some(p) = find_executable(comp) {
            if runtime_compatible(&p, mc_version) {
                return Some(p);
            }
        }
    }
    None
}

fn runtime_compatible(path: &PathBuf, mc_version: &str) -> bool {
    verify_java(path, "mojang")
        .map(|j| is_compatible(j.version_major, mc_version))
        .unwrap_or(false)
}
