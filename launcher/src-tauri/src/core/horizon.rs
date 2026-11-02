//! Integración oficial al estilo Paraguacraft para el motor nativo **Horizon** (`horizon-rust/horizon`).
//!
//! Horizon es una reescritura experimental de Minecraft `26.3` en Rust + Vulkan:
//! - Sin Java / JVM (arranque en `<1s`, ~300 MB RAM base).
//! - Hasta 1024 chunks con LODs acelerados por GPU Vulkan.
//! - **Cumplimiento estricto**: exige cuenta oficial Microsoft con Minecraft comprado (`acc.premium == true`)
//!   y extrae texturas/modelos directamente de los assets oficiales `26.3` de Mojang.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::config;
use crate::core::accounts;
use crate::core::hardware;
use crate::core::instances;
use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::core::versions;
use crate::error::{AppError, AppResult};
use crate::models::Account;

pub const HORIZON_MC_VERSION: &str = "26.3";
const HORIZON_RELEASES_API: &str = "https://api.github.com/repos/horizon-rust/horizon/releases?per_page=15";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HorizonMeta {
    #[serde(default)]
    installed_tag: Option<String>,
    #[serde(default)]
    binary_path: Option<String>,
    #[serde(default)]
    custom_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HorizonStatus {
    pub platform_supported: bool,
    pub premium_allowed: bool,
    pub username: Option<String>,
    pub assets_263_ready: bool,
    pub installed: bool,
    pub binary_path: Option<String>,
    pub installed_tag: Option<String>,
    pub latest_tag: Option<String>,
    pub custom_binary: bool,
    pub gpu_name: String,
    pub vulkan_hint: String,
    pub horizon_dir: String,
}

pub fn horizon_dir() -> PathBuf {
    let dir = paths::data_dir().join("horizon");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn meta_path() -> PathBuf {
    horizon_dir().join("_horizon_meta.json")
}

fn read_meta() -> HorizonMeta {
    config::read_json::<HorizonMeta>(&meta_path()).unwrap_or_default()
}

fn write_meta(meta: &HorizonMeta) -> AppResult<()> {
    config::write_json_atomic(&meta_path(), meta)
}

/// Verifica estrictamente que haya una cuenta oficial Microsoft con Minecraft activo.
/// Nunca se saltea ni falsifica: Horizon requiere propiedad legítima del juego.
pub fn require_official_microsoft_account() -> AppResult<Account> {
    let acc = accounts::active_account().ok_or_else(|| {
        AppError::msg(
            "No hay ninguna cuenta activa. Iniciá sesión con tu cuenta Microsoft oficial en Ajustes para usar Horizon.",
        )
    })?;
    if !acc.premium || acc.kind != "microsoft" {
        return Err(AppError::msg(
            "Horizon exige obligatoriamente una cuenta Microsoft oficial con Minecraft comprado. Cambiá a tu cuenta Microsoft en Ajustes o jugá con una instancia Java normal.",
        ));
    }
    Ok(acc)
}

pub fn assets_263_ready() -> bool {
    if !versions::profile_locally_complete(HORIZON_MC_VERSION) {
        return false;
    }
    let Some(v) = versions::read_local_json(HORIZON_MC_VERSION) else {
        return false;
    };
    if let Some(asset_id) = v["assetIndex"]["id"].as_str() {
        if !asset_id.is_empty() {
            let idx = paths::default_minecraft_dir()
                .join("assets")
                .join("indexes")
                .join(format!("{asset_id}.json"));
            if !idx.is_file() {
                return false;
            }
        }
    }
    true
}

fn find_horizon_exe_in_dir(root: &Path) -> Option<PathBuf> {
    let preferred = [
        "horizon.exe",
        "horizon-client.exe",
        "Horizon.exe",
        "horizon-launcher.exe",
        "Horizon Launcher.exe",
        "horizon_client.exe",
        "horizon-client-win64.exe",
        "horizon-win64.exe",
    ];
    for name in preferred {
        let p = root.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    // Búsqueda recursiva profunda (hasta 3 niveles) tras extraer zip.
    let mut fallback_exe: Option<PathBuf> = None;
    let mut stack = vec![(root.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > 3 {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_file() {
                let low = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if low.ends_with(".exe") && !low.contains("uninstall") && !low.contains("vc_redist") {
                    // Prioridad 1: nombre contiene "horizon" y no "server"
                    if low.contains("horizon") && !low.contains("server") {
                        return Some(path);
                    }
                    // Prioridad 2: cualquier .exe en la raíz o primera subcarpeta
                    if fallback_exe.is_none() && depth <= 1 {
                        fallback_exe = Some(path);
                    }
                }
            } else if path.is_dir() {
                // Saltar carpetas obvias que no contienen el binario
                let dir_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if !dir_name.contains("uninstall") && !dir_name.contains("vc_redist") {
                    stack.push((path, depth + 1));
                }
            }
        }
    }
    fallback_exe
}

pub fn resolve_installed_binary() -> (Option<PathBuf>, HorizonMeta) {
    let meta = read_meta();
    if let Some(ref custom) = meta.binary_path {
        let p = PathBuf::from(custom);
        if p.is_file() {
            return (Some(p), meta);
        }
    }
    let dir = horizon_dir();
    let found = find_horizon_exe_in_dir(&dir);
    (found, meta)
}

fn build_vulkan_hint(gpu_name: &str) -> String {
    let low = gpu_name.to_ascii_lowercase();
    if low.contains("rtx") || low.contains("gtx 16") || low.contains("gtx 10") || low.contains("radeon rx") || low.contains("arc") {
        format!("{gpu_name} — Compatible con Vulkan 1.2+ (ideal para Horizon y LODs lejanos).")
    } else if low.contains("iris") || low.contains("uhd") || low.contains("vega") || low.contains("radeon") || low.contains("geforce") {
        format!("{gpu_name} — Soporta Vulkan; mantené tus drivers al día para mejor estabilidad en Horizon.")
    } else if low.contains("hd graphics 4000") || low.contains("hd graphics 3000") || low.contains("hd graphics 2000") {
        format!("{gpu_name} — Atención: GPU integrada antigua sin soporte completo de Vulkan 1.2; te recomendamos Java 26.3 Optimized.")
    } else {
        format!("{gpu_name} — Horizon requiere drivers con soporte Vulkan 1.2+ en Windows x64.")
    }
}

pub async fn status(client: Option<&reqwest::Client>) -> HorizonStatus {
    let hw = hardware::detect();
    let acc = accounts::active_account();
    let premium_allowed = acc
        .as_ref()
        .map(|a| a.premium && a.kind == "microsoft")
        .unwrap_or(false);
    let username = acc.filter(|a| a.premium && a.kind == "microsoft").map(|a| a.username);

    let (exe_opt, meta) = resolve_installed_binary();
    let latest_tag = if let Some(http) = client {
        fetch_latest_release_asset(http)
            .await
            .ok()
            .map(|(tag, _)| tag)
    } else {
        None
    };

    HorizonStatus {
        platform_supported: cfg!(all(target_os = "windows", target_arch = "x86_64")),
        premium_allowed,
        username,
        assets_263_ready: assets_263_ready(),
        installed: exe_opt.is_some(),
        binary_path: exe_opt.map(|p| p.to_string_lossy().to_string()),
        installed_tag: meta.installed_tag,
        latest_tag,
        custom_binary: meta.custom_binary,
        vulkan_hint: build_vulkan_hint(&hw.gpu_name),
        gpu_name: hw.gpu_name,
        horizon_dir: horizon_dir().to_string_lossy().to_string(),
    }
}

async fn fetch_latest_release_asset(client: &reqwest::Client) -> AppResult<(String, String)> {
    let resp = client
        .get(HORIZON_RELEASES_API)
        .header("User-Agent", "Paraguacraft-Launcher")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AppError::msg(format!("No se pudo consultar GitHub de Horizon: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::msg(format!(
            "GitHub API devolvió HTTP {} al buscar builds de Horizon.",
            resp.status()
        )));
    }
    let releases: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::msg(format!("Respuesta inválida de GitHub: {e}")))?;

    let Some(arr) = releases.as_array() else {
        return Err(AppError::msg("Formato inesperado en releases de Horizon."));
    };

    let mut fallback_launcher: Option<(String, String)> = None;

    for rel in arr {
        let tag = rel["tag_name"].as_str().unwrap_or_default().to_string();
        let Some(assets) = rel["assets"].as_array() else {
            continue;
        };
        // Prioridad 1: build directo del motor cliente `horizon-windows-x64.zip` (ej. nightly-*)
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or_default().to_ascii_lowercase();
            let url = asset["browser_download_url"].as_str().unwrap_or_default();
            if url.is_empty() {
                continue;
            }
            if name == "horizon-windows-x64.zip"
                || (name.contains("horizon")
                    && name.contains("windows")
                    && (name.contains("x64") || name.contains("amd64"))
                    && !name.contains("launcher")
                    && name.ends_with(".zip"))
            {
                return Ok((tag, url.to_string()));
            }
            if fallback_launcher.is_none()
                && (name == "horizon-launcher-windows-x64.zip"
                    || (name.contains("windows") && name.ends_with(".zip")))
            {
                fallback_launcher = Some((tag.clone(), url.to_string()));
            }
        }
    }

    if let Some(fb) = fallback_launcher {
        return Ok(fb);
    }

    Err(AppError::msg(
        "No se encontró un paquete Windows x64 publicado en los últimos releases de horizon-rust/horizon.",
    ))
}

pub async fn prepare_assets(app: &AppHandle, client: &reqwest::Client) -> AppResult<()> {
    let _acc = require_official_microsoft_account()?;
    versions::install_vanilla(app, client, HORIZON_MC_VERSION).await
}

pub async fn install_engine(app: &AppHandle, client: &reqwest::Client) -> AppResult<HorizonStatus> {
    if !cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        return Err(AppError::msg(
            "El motor nativo Horizon actualmente requiere Windows de 64 bits (x64).",
        ));
    }
    let _acc = require_official_microsoft_account()?;

    let (tag, download_url) = fetch_latest_release_asset(client).await?;
    let root = horizon_dir();
    let zip_dest = root.join("horizon-package.zip");
    let _ = std::fs::remove_file(&zip_dest);

    net::download_all(
        client,
        vec![DownloadItem::new(download_url, zip_dest.clone())],
        1,
        app,
        "horizon-engine",
        &format!("Motor Horizon ({tag})"),
    )
    .await?;

    let root_extract = root.clone();
    let zip_extract = zip_dest.clone();
    let exe_path = tokio::task::spawn_blocking(move || -> AppResult<PathBuf> {
        extract_zip(&zip_extract, &root_extract)?;
        let _ = std::fs::remove_file(&zip_extract);
        find_horizon_exe_in_dir(&root_extract).ok_or_else(|| {
            AppError::msg(
                "El paquete de Horizon se descargó pero no contenía un ejecutable .exe reconocible.",
            )
        })
    })
    .await
    .map_err(|e| AppError::msg(format!("Extracción abortada: {e}")))??;

    let meta = HorizonMeta {
        installed_tag: Some(tag.clone()),
        binary_path: Some(exe_path.to_string_lossy().to_string()),
        custom_binary: false,
    };
    write_meta(&meta)?;

    // Crear/actualizar la instancia de Horizon en instancias/
    let _instance_dir = get_or_create_horizon_instance()?;
    if let Some(mut instance_meta) = instances::read_meta(&instances::folder_for(HORIZON_MC_VERSION, "horizon")) {
        instance_meta.loader_version = tag.clone();
        instance_meta.version_id = Some(format!("horizon-{}", HORIZON_MC_VERSION));
        let _ = instances::write_meta(&instances::folder_for(HORIZON_MC_VERSION, "horizon"), &instance_meta);
    }

    let mut st = status(None).await;
    st.latest_tag = Some(tag);
    Ok(st)
}

/// Obtiene o crea la instancia de Horizon en la carpeta instancias.
/// Devuelve la ruta del directorio de la instancia.
pub fn get_or_create_horizon_instance() -> AppResult<PathBuf> {
    let mc_version = HORIZON_MC_VERSION;
    let instance_folder = instances::folder_for(mc_version, "horizon");
    let instance_dir = instances::instance_dir(&instance_folder);

    // Crear el directorio si no existe
    std::fs::create_dir_all(&instance_dir)?;

    // Crear/actualizar metadata si no existe
    if instances::read_meta(&instance_folder).is_none() {
        let meta = instances::InstanceMeta {
            name: format!("{} Horizon", mc_version),
            icon: instances::icon_for_loader("horizon"),
            mc_version: mc_version.to_string(),
            loader: "horizon".to_string(),
            loader_version: "latest".to_string(),
            source: "paraguacraft".to_string(),
            ram_mb: 2048,
            ram_min_mb: 1024,
            total_play_minutes: 0,
            last_played: None,
            version_id: Some(format!("horizon-{}", mc_version)),
            auto_managed: true,
            jvm_args: None,
            gc: None,
            java_path: None,
            performance_tier: None,
            show_game_console: None,
        };
        instances::write_meta(&instance_folder, &meta)?;
    }

    Ok(instance_dir)
}

fn extract_zip(zip_path: &Path, dest_dir: &Path) -> AppResult<()> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| AppError::msg(format!("No se pudo abrir ZIP de Horizon: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::msg(format!("ZIP de Horizon corrupto: {e}")))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::msg(format!("Entrada ZIP inválida: {e}")))?;
        let Some(enclosed) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
            continue;
        };
        let out_path = dest_dir.join(enclosed);
        if entry.is_dir() {
            let _ = std::fs::create_dir_all(&out_path);
        } else {
            if let Some(parent) = out_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let mut outfile = std::fs::File::create(&out_path).map_err(|e| {
                AppError::msg(format!(
                    "No se pudo escribir {} (¿Horizon sigue abierto?): {e}",
                    out_path.display()
                ))
            })?;
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| AppError::msg(format!("Error leyendo ZIP: {e}")))?;
            use std::io::Write;
            outfile
                .write_all(&buf)
                .map_err(|e| AppError::msg(format!("Error guardando archivo: {e}")))?;
        }
    }
    Ok(())
}

pub fn set_custom_binary(path: &Path) -> AppResult<HorizonStatus> {
    let _acc = require_official_microsoft_account()?;
    if !path.is_file() {
        return Err(AppError::msg("El archivo seleccionado no existe."));
    }
    let ext_ok = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("exe"))
        .unwrap_or(false);
    if !ext_ok {
        return Err(AppError::msg("Seleccioná un ejecutable .exe válido de Horizon."));
    }
    let meta = HorizonMeta {
        installed_tag: Some("Binario local".to_string()),
        binary_path: Some(path.to_string_lossy().to_string()),
        custom_binary: true,
    };
    write_meta(&meta)?;

    let hw = hardware::detect();
    let acc = accounts::active_account();
    Ok(HorizonStatus {
        platform_supported: cfg!(all(target_os = "windows", target_arch = "x86_64")),
        premium_allowed: true,
        username: acc.map(|a| a.username),
        assets_263_ready: assets_263_ready(),
        installed: true,
        binary_path: Some(path.to_string_lossy().to_string()),
        installed_tag: meta.installed_tag,
        latest_tag: None,
        custom_binary: true,
        vulkan_hint: build_vulkan_hint(&hw.gpu_name),
        gpu_name: hw.gpu_name,
        horizon_dir: horizon_dir().to_string_lossy().to_string(),
    })
}

pub async fn launch(app: &AppHandle, client: &reqwest::Client) -> AppResult<std::process::Child> {
    // 1. Verificación estricta de cuenta Microsoft oficial y token Mojang vigente.
    let acc = require_official_microsoft_account()?;
    let token = accounts::ensure_valid_token(client, &acc.id).await.map_err(|e| {
        AppError::msg(format!(
            "No se pudo validar tu sesión oficial de Microsoft/Mojang para Horizon: {e}"
        ))
    })?;

    // 2. Garantizar assets oficiales 26.3 de Mojang en .minecraft.
    if !assets_263_ready() {
        versions::install_vanilla(app, client, HORIZON_MC_VERSION).await?;
    }

    // 3. Garantizar que el binario de Horizon esté instalado.
    let (mut exe_opt, _) = resolve_installed_binary();
    if exe_opt.is_none() {
        let _ = install_engine(app, client).await?;
        exe_opt = resolve_installed_binary().0;
    }
    let exe = exe_opt.ok_or_else(|| {
        AppError::msg("No se encontró el ejecutable de Horizon. Tocá «Descargar Horizon» primero.")
    })?;

    // 4. Obtener/crear la instancia de Horizon
    let instance_dir = get_or_create_horizon_instance()?;
    let _instance_meta = instances::read_meta(&instances::folder_for(HORIZON_MC_VERSION, "horizon"))
        .unwrap_or_else(|| instances::InstanceMeta {
            name: format!("{} Horizon", HORIZON_MC_VERSION),
            icon: instances::icon_for_loader("horizon"),
            mc_version: HORIZON_MC_VERSION.to_string(),
            loader: "horizon".to_string(),
            loader_version: "latest".to_string(),
            source: "paraguacraft".to_string(),
            ram_mb: 2048,
            ram_min_mb: 1024,
            total_play_minutes: 0,
            last_played: None,
            version_id: Some(format!("horizon-{}", HORIZON_MC_VERSION)),
            auto_managed: true,
            jvm_args: None,
            gc: None,
            java_path: None,
            performance_tier: None,
            show_game_console: None,
        });

    let work_dir = exe
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(horizon_dir);

    let mut cmd = Command::new(&exe);
    cmd.current_dir(&work_dir)
        .env("MINECRAFT_DIR", &instance_dir)
        .env("HORIZON_MC_USERNAME", &acc.username)
        .env("HORIZON_MC_UUID", &acc.uuid)
        .env("HORIZON_MC_ACCESS_TOKEN", &token.mc_access_token);

    let child = cmd
        .spawn()
        .map_err(|e| AppError::msg(format!("No se pudo iniciar Horizon ({}): {e}", exe.display())))?;

    Ok(child)
}

pub fn open_folder() -> AppResult<()> {
    let dir = horizon_dir();
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = dir;
    }
    Ok(())
}
