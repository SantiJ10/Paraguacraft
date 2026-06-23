//! Resolución de Java por versión de Minecraft: Mojang runtime → Temurin → sistema.

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::core::java::adoptium;
use crate::core::java::detect::detect_all;
use crate::core::java::mojang;
use crate::core::java::{is_compatible, required_for_mc};
use crate::error::{AppError, AppResult};
use crate::models::JavaInstallation;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaRole {
    /// Lanzar el juego (sin consola en Windows → javaw).
    Launch,
    /// Instaladores Forge/NeoForge/OptiFine (java.exe).
    Installer,
}

fn source_rank(source: &str) -> u8 {
    match source.split(':').next().unwrap_or(source) {
        "paraguacraft" => 0,
        "mojang" => 1,
        "java_home" => 2,
        "path" => 3,
        _ => 4,
    }
}

/// Elige el mejor Java detectado compatible con la versión de MC.
pub fn pick_compatible(javas: &[JavaInstallation], mc_version: &str) -> Option<JavaInstallation> {
    let required = required_for_mc(mc_version);
    let mut compatible: Vec<&JavaInstallation> = javas
        .iter()
        .filter(|j| is_compatible(j.version_major, mc_version))
        .collect();
    if compatible.is_empty() {
        return None;
    }
    compatible.sort_by(|a, b| {
        let a_exact = a.version_major == required;
        let b_exact = b.version_major == required;
        b_exact
            .cmp(&a_exact)
            .then_with(|| a.version_major.cmp(&b.version_major))
            .then_with(|| source_rank(&a.source).cmp(&source_rank(&b.source)))
    });
    compatible.first().map(|j| (*j).clone())
}

fn format_path(path: &Path, role: JavaRole) -> PathBuf {
    let s = path.to_string_lossy();
    match role {
        JavaRole::Launch => PathBuf::from(prefer_javaw(&s)),
        JavaRole::Installer => PathBuf::from(prefer_java_exe(&s)),
    }
}

pub fn prefer_javaw(path: &str) -> String {
    if cfg!(target_os = "windows") {
        path.replace("java.exe", "javaw.exe")
    } else {
        path.to_string()
    }
}

pub fn prefer_java_exe(path: &str) -> String {
    if cfg!(target_os = "windows") {
        path.replace("javaw.exe", "java.exe")
    } else {
        path.to_string()
    }
}

/// Valida que un Java manual sea compatible con la versión de MC.
pub fn validate_override(path: &str, mc_version: &str) -> AppResult<()> {
    use crate::core::java::verify::verify;
    let info = verify(Path::new(path), "custom").ok_or_else(|| {
        AppError::msg(format!("Ruta de Java inválida: {path}"))
    })?;
    if !is_compatible(info.version_major, mc_version) {
        let req = required_for_mc(mc_version);
        return Err(AppError::msg(format!(
            "Java {} no es compatible con Minecraft {mc_version} (se necesita Java {req}).",
            info.version_major
        )));
    }
    Ok(())
}

/// Resuelve Java para lanzar (sin descargar). Falla si no hay compatible.
pub fn resolve_sync(
    mc_version: &str,
    version_id: Option<&str>,
    override_path: Option<&str>,
    role: JavaRole,
) -> AppResult<PathBuf> {
    if let Some(p) = override_path.filter(|s| !s.is_empty()) {
        validate_override(p, mc_version)?;
        return Ok(format_path(Path::new(p), role));
    }

    if let Some(p) = mojang::find_for_mc(mc_version, version_id) {
        return Ok(format_path(&p, role));
    }

    if let Some(j) = pick_compatible(&detect_all(), mc_version) {
        return Ok(format_path(Path::new(&j.path), role));
    }

    let req = required_for_mc(mc_version);
    Err(AppError::msg(format!(
        "No hay Java {req} compatible con Minecraft {mc_version}. Instalalo en Ajustes → Java."
    )))
}

/// Asegura Java para lanzar: Mojang runtime → detect → descarga Temurin si falta.
pub async fn ensure_launch_java(
    app: &AppHandle,
    state: &AppState,
    mc_version: &str,
    version_id: &str,
    meta_override: Option<&str>,
    settings_override: Option<&str>,
) -> AppResult<PathBuf> {
    let user_override = meta_override
        .filter(|s| !s.is_empty())
        .or_else(|| settings_override.filter(|s| !s.is_empty()));

    if let Some(p) = user_override {
        validate_override(p, mc_version)?;
        return Ok(format_path(Path::new(p), JavaRole::Launch));
    }

    // Runtime oficial Mojang (descarga si falta).
    if let Some(comp) = mojang::component_from_version_id(version_id) {
        let (http, _guard) = state.net_scope();
        if let Ok(p) = mojang::ensure_runtime(app, &http, &comp).await {
            return Ok(format_path(&p, JavaRole::Launch));
        }
    }

    if let Some(p) = mojang::find_for_mc(mc_version, Some(version_id)) {
        return Ok(format_path(&p, JavaRole::Launch));
    }

    if let Some(j) = pick_compatible(&detect_all(), mc_version) {
        return Ok(format_path(Path::new(&j.path), JavaRole::Launch));
    }

    let required = required_for_mc(mc_version);
    if let Some(p) = adoptium::find_installed(required) {
        return Ok(format_path(&p, JavaRole::Launch));
    }

    let (http, _guard) = state.net_scope();
    *state.java_cache.lock().unwrap() = None;
    let path = adoptium::download(app, &http, required, false).await?;
    Ok(format_path(Path::new(&path), JavaRole::Launch))
}

/// Java para instaladores de loaders (Forge, etc.). Descarga Temurin si falta.
pub async fn ensure_installer_java(
    app: &AppHandle,
    state: &AppState,
    mc_version: &str,
) -> AppResult<PathBuf> {
    if let Ok(p) = resolve_sync(mc_version, None, None, JavaRole::Installer) {
        return Ok(p);
    }
    let required = required_for_mc(mc_version);
    if let Some(p) = adoptium::find_installed(required) {
        return Ok(format_path(&p, JavaRole::Installer));
    }
    let (http, _guard) = state.net_scope();
    let path = adoptium::download(app, &http, required, false).await?;
    Ok(format_path(Path::new(&path), JavaRole::Installer))
}

/// Java para instaladores (sync, sin descarga). Preferí `ensure_installer_java`.
pub fn resolve_installer_java(mc_version: &str) -> AppResult<PathBuf> {
    resolve_sync(mc_version, None, None, JavaRole::Installer)
}
