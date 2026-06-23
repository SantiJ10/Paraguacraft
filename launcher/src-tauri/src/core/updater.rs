//! Auto-update del launcher (on-demand, sin timers en background).
//!
//! 1. Consulta GitHub Releases para metadata y URL del instalador.
//! 2. Descarga con progreso (`update://progress`) e instala (MSI/EXE/NSIS).
//! 3. Compatible con `tauri-plugin-updater` cuando hay `latest.json` firmado en releases.

use std::path::{Path, PathBuf};
use std::process::Command;

use tauri::{AppHandle, Emitter};

use crate::core::paths;
use crate::error::{AppError, AppResult};

const REPO: &str = "SantiJ10/Paraguacraft";
const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes: String,
    pub published_at: String,
    /// Instalador detectado en el release (exe/msi) — permite update in-app.
    pub in_app_install: bool,
    pub asset_name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub phase: String,
    pub progress: f64,
    pub message: String,
}

fn emit_progress(app: &AppHandle, phase: &str, progress: f64, message: &str) {
    let _ = app.emit(
        "update://progress",
        UpdateProgress {
            phase: phase.into(),
            progress,
            message: message.into(),
        },
    );
}

fn version_parts(v: &str) -> [u64; 4] {
    let mut out = [0u64; 4];
    for (i, p) in v.trim_start_matches('v').split('.').take(4).enumerate() {
        let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
        out[i] = digits.parse().unwrap_or(0);
    }
    out
}

pub fn version_gt(a: &str, b: &str) -> bool {
    let aa = version_parts(a);
    let bb = version_parts(b);
    aa > bb
}

fn version_base(tag: &str) -> [u64; 4] {
    let base = tag
        .trim_start_matches('v')
        .split('-')
        .next()
        .unwrap_or("");
    version_parts(base)
}

fn is_unified_launcher_release(release: &serde_json::Value) -> bool {
    let tag = release["tag_name"].as_str().unwrap_or("");
    version_base(tag) >= [6, 9, 0, 0]
}

fn is_legacy_python_release(release: &serde_json::Value) -> bool {
    if is_unified_launcher_release(release) || is_tauri_launcher_release(release) {
        return false;
    }
    let tag = release["tag_name"].as_str().unwrap_or("").trim_start_matches('v');
    tag.starts_with("6.") || tag.starts_with("5.")
}

fn is_tauri_launcher_release(release: &serde_json::Value) -> bool {
    if is_unified_launcher_release(release) {
        return true;
    }
    let tag = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();
    if tag.contains("launcher") {
        return true;
    }
    release["assets"]
        .as_array()
        .map(|assets| {
            assets.iter().any(|a| {
                let name = a["name"].as_str().unwrap_or("").to_lowercase();
                (name.contains("launcher") || name == "latest.json")
                    && !name.starts_with("instalar_paraguacraft")
                    && name != "paraguacraft.exe"
            })
        })
        .unwrap_or(false)
}

fn skip_obsolete_asset(name: &str) -> bool {
    name.to_lowercase() == "paraguacraft.exe"
}

fn pick_asset(assets: &[serde_json::Value]) -> Option<(String, String)> {
    let arch_hint = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    };

    let mut scored: Vec<(i32, String, String)> = Vec::new();
    for a in assets {
        let name = a["name"].as_str().unwrap_or("").to_string();
        let url = a["browser_download_url"].as_str().unwrap_or("").to_string();
        if url.is_empty() {
            continue;
        }
        let lower = name.to_lowercase();
        if skip_obsolete_asset(&lower) {
            continue;
        }
        if !(lower.ends_with(".exe") || lower.ends_with(".msi") || lower.ends_with(".appimage")) {
            continue;
        }
        let mut score = 0;
        if lower.starts_with("instalar_paraguacraft") {
            score += 25;
        }
        if lower.contains(arch_hint) || lower.contains("x86_64") || lower.contains("amd64") {
            score += 10;
        }
        if lower.contains("setup") || lower.contains("installer") {
            score += 5;
        }
        if lower.contains("paraguacraft") || lower.contains("launcher") {
            score += 3;
        }
        if lower == "latest.json" {
            score += 20;
        }
        if cfg!(target_os = "windows") && (lower.ends_with(".exe") || lower.ends_with(".msi")) {
            score += 8;
        }
        if cfg!(target_os = "linux") && lower.ends_with(".appimage") {
            score += 8;
        }
        if cfg!(target_os = "macos") && (lower.ends_with(".dmg") || lower.ends_with(".app.tar.gz")) {
            score += 8;
        }
        scored.push((score, name, url));
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().next().map(|(_, n, u)| (n, u))
}

pub async fn check(client: &reqwest::Client) -> AppResult<UpdateInfo> {
    let url = format!("https://api.github.com/repos/{REPO}/releases?per_page=20");
    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "ParaguacraftLauncher/2.0")
        .send()
        .await?
        .error_for_status()?;
    let releases: Vec<serde_json::Value> = resp.json().await?;

    let launcher_release = releases.iter().find(|r| is_tauri_launcher_release(r));

    let release = if let Some(r) = launcher_release {
        r
    } else {
        // Sin releases del launcher Tauri: no ofrecer el Python v6.x como actualización.
        return Ok(UpdateInfo {
            current_version: CURRENT.to_string(),
            latest_version: CURRENT.to_string(),
            update_available: false,
            download_url: None,
            release_notes: String::new(),
            published_at: String::new(),
            in_app_install: false,
            asset_name: None,
        });
    };

    let tag = release["tag_name"]
        .as_str()
        .unwrap_or(CURRENT)
        .trim_start_matches('v');
    let notes = release["body"].as_str().unwrap_or("").to_string();
    let published = release["published_at"].as_str().unwrap_or("").to_string();

    let assets = release["assets"].as_array().cloned().unwrap_or_default();
    let picked = pick_asset(&assets);
    let (asset_name, download_url) = if let Some((n, u)) = picked {
        (Some(n), Some(u))
    } else {
        (None, release["html_url"].as_str().map(String::from))
    };

    let in_app_install = download_url
        .as_ref()
        .map(|u| {
            let l = u.to_lowercase();
            l.ends_with(".exe") || l.ends_with(".msi") || l.ends_with(".appimage")
        })
        .unwrap_or(false);

    let mut update_available = !tag.is_empty() && tag != CURRENT && version_gt(tag, CURRENT);
    if update_available && is_legacy_python_release(release) && !is_tauri_launcher_release(release) {
        update_available = false;
    }

    Ok(UpdateInfo {
        current_version: CURRENT.to_string(),
        latest_version: if update_available {
            tag.to_string()
        } else {
            CURRENT.to_string()
        },
        update_available,
        download_url,
        release_notes: notes,
        published_at: published,
        in_app_install,
        asset_name,
    })
}

fn updates_dir() -> PathBuf {
    let dir = paths::data_dir().join("updates");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Descarga el instalador del release a `{data}/updates/`.
pub async fn download_update(app: &AppHandle, client: &reqwest::Client, url: &str, filename: &str) -> AppResult<PathBuf> {
    emit_progress(app, "download", 0.0, "Iniciando descarga…");
    let dest = updates_dir().join(filename);
    let tmp = dest.with_extension("part");

    let resp = client
        .get(url)
        .header("User-Agent", "ParaguacraftLauncher/2.0")
        .send()
        .await?
        .error_for_status()?;

    let total = resp.content_length().unwrap_or(0);
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(&tmp).await?;
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        if total > 0 {
            let pct = (downloaded as f64 / total as f64).min(1.0);
            emit_progress(
                app,
                "download",
                pct,
                &format!("Descargando… {}%", (pct * 100.0) as u32),
            );
        }
    }
    file.flush().await?;
    drop(file);
    tokio::fs::rename(&tmp, &dest).await?;
    emit_progress(app, "download", 1.0, "Descarga completa.");
    Ok(dest)
}

/// Ejecuta el instalador descargado y cierra el launcher.
pub fn run_installer(path: &Path) -> AppResult<()> {
    if !path.is_file() {
        return Err(AppError::msg("Instalador no encontrado"));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        if ext == "msi" {
            Command::new("msiexec")
                .args(["/i", &path.to_string_lossy(), "/passive", "/norestart"])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| AppError::msg(format!("No se pudo ejecutar MSI: {e}")))?;
        } else {
            Command::new(path)
                .args(["/S"])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| AppError::msg(format!("No se pudo ejecutar instalador: {e}")))?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
        Command::new(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo ejecutar AppImage: {e}")))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir instalador: {e}")))?;
    }

    Ok(())
}

pub async fn download_and_install(
    app: &AppHandle,
    client: &reqwest::Client,
    info: &UpdateInfo,
) -> AppResult<()> {
    let url = info
        .download_url
        .as_ref()
        .ok_or_else(|| AppError::msg("Sin URL de descarga"))?;
    let name = info
        .asset_name
        .clone()
        .or_else(|| {
            url.rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .map(String::from)
        })
        .unwrap_or_else(|| "ParaguacraftLauncher-setup.exe".into());

    if !info.in_app_install {
        return Err(AppError::msg(
            "No hay instalador directo. Usa el enlace del release en GitHub.",
        ));
    }

    let path = download_update(app, client, url, &name).await?;
    emit_progress(app, "install", 0.0, "Iniciando instalador…");
    run_installer(&path)?;
    emit_progress(app, "install", 1.0, "Instalador lanzado. Cerrá el launcher para completar.");
    Ok(())
}

#[allow(dead_code)]
pub fn current_version() -> &'static str {
    CURRENT
}
