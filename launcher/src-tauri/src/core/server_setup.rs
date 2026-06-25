//! Preparación on-demand de servidores (Paper, Fabric, Forge, Geyser).

use std::path::{Path, PathBuf};

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::server_manager;
use crate::core::servers::ServerProfile;
use crate::error::{AppError, AppResult};

pub const SERVER_TYPES: &[&str] = &[
    "paper",
    "paper-geyser",
    "fabric",
    "fabric-geyser",
    "forge",
];

pub fn normalize_server_type(raw: &str) -> AppResult<String> {
    match raw.trim().to_lowercase().replace('_', "-").as_str() {
        "paper" => Ok("paper".into()),
        "paper-geyser" | "paper+geyser" => Ok("paper-geyser".into()),
        "fabric" => Ok("fabric".into()),
        "fabric-geyser" | "fabric+geyser" => Ok("fabric-geyser".into()),
        "forge" => Ok("forge".into()),
        other => Err(AppError::msg(format!(
            "Tipo de servidor invalido: {other}. Usa: paper, paper-geyser, fabric, fabric-geyser, forge."
        ))),
    }
}

pub fn type_label(t: &str) -> &'static str {
    match t {
        "paper-geyser" => "Paper + Geyser",
        "fabric-geyser" => "Fabric + Geyser",
        "fabric" => "Fabric",
        "forge" => "Forge",
        _ => "Paper",
    }
}

/// Descarga jar, mods/plugins y EULA según el tipo del servidor.
pub async fn prepare(
    app: &AppHandle,
    client: &reqwest::Client,
    prof: &ServerProfile,
    dir: &Path,
) -> AppResult<PathBuf> {
    std::fs::create_dir_all(dir.join("plugins"))?;
    std::fs::create_dir_all(dir.join("mods"))?;
    std::fs::create_dir_all(dir.join("world"))?;
    write_eula(dir)?;

    let kind = normalize_server_type(&prof.server_type)?;
    let jar = dir.join("server.jar");

    let result = match kind.as_str() {
        "paper" => {
            download_paper(client, app, &prof.mc_version, &jar).await?;
            setup_paper_plugins(client, app, dir, &prof.mc_version, false).await?;
            jar
        }
        "paper-geyser" => {
            download_paper(client, app, &prof.mc_version, &jar).await?;
            setup_paper_plugins(client, app, dir, &prof.mc_version, true).await?;
            jar
        }
        "fabric" => {
            download_fabric(client, app, &prof.mc_version, &jar).await?;
            setup_fabric_mods(client, app, dir, &prof.mc_version, false).await?;
            jar
        }
        "fabric-geyser" => {
            download_fabric(client, app, &prof.mc_version, &jar).await?;
            setup_fabric_mods(client, app, dir, &prof.mc_version, true).await?;
            jar
        }
        "forge" => setup_forge(client, app, dir, &prof.mc_version).await?,
        other => return Err(AppError::msg(format!("Tipo no implementado: {other}"))),
    };

    ensure_playit_exe(client, app, dir).await?;
    Ok(result)
}

const PLAYIT_MIN_BYTES: u64 = 2_000_000;

/// Descarga playit.exe si no existe o es demasiado pequeño (espejo de modelo.py).
async fn ensure_playit_exe(
    client: &reqwest::Client,
    _app: &AppHandle,
    dir: &Path,
) -> AppResult<()> {
    let playit_path = dir.join("playit.exe");
    if playit_path.is_file() {
        if playit_path.metadata().map(|m| m.len()).unwrap_or(0) >= PLAYIT_MIN_BYTES {
            return Ok(());
        }
        let _ = std::fs::remove_file(&playit_path);
    }
    let global = crate::core::paths::data_dir().join("playit.exe");
    if global.is_file() && global.metadata().map(|m| m.len()).unwrap_or(0) >= PLAYIT_MIN_BYTES {
        std::fs::copy(&global, &playit_path)?;
        return Ok(());
    }

    let release: Value = net::fetch_json(
        client,
        "https://api.github.com/repos/playit-cloud/playit-agent/releases/latest",
    )
    .await?;
    let assets = release["assets"].as_array().cloned().unwrap_or_default();
    let exe_url = assets
        .iter()
        .find_map(|a| {
            let n = a["name"].as_str()?.to_lowercase();
            if n.ends_with(".exe") && (n.contains("win") || n.contains("windows")) {
                a["browser_download_url"].as_str().map(String::from)
            } else {
                None
            }
        })
        .ok_or_else(|| AppError::msg("No hay .exe Windows en el release de playit"))?;

    net::download_one(
        client,
        &DownloadItem::new(exe_url, &playit_path),
    )
    .await?;
    if playit_path.metadata().map(|m| m.len()).unwrap_or(0) < PLAYIT_MIN_BYTES {
        let _ = std::fs::remove_file(&playit_path);
        return Err(AppError::msg("Descarga de playit.exe truncada"));
    }
    Ok(())
}

fn write_eula(dir: &Path) -> AppResult<()> {
    server_manager::write_eula(dir)
}

async fn download_paper(
    client: &reqwest::Client,
    app: &AppHandle,
    mc: &str,
    jar: &Path,
) -> AppResult<()> {
    if jar.is_file() && jar.metadata().map(|m| m.len()).unwrap_or(0) > 1_000_000 {
        return Ok(());
    }
    server_manager::download_paper_server(client, app, mc, jar).await
}

async fn download_fabric(
    client: &reqwest::Client,
    app: &AppHandle,
    mc: &str,
    jar: &Path,
) -> AppResult<()> {
    if jar.is_file() && jar.metadata().map(|m| m.len()).unwrap_or(0) > 100_000 {
        return Ok(());
    }
    server_manager::download_fabric_server(client, app, mc, jar).await
}

async fn setup_forge(
    client: &reqwest::Client,
    app: &AppHandle,
    dir: &Path,
    mc: &str,
) -> AppResult<PathBuf> {
    if let Some(existing) = find_server_jar(dir, "forge") {
        return Ok(existing);
    }
    server_manager::download_forge_server(client, app, mc, dir).await
}

/// Localiza el jar ejecutable del servidor en `dir`.
pub fn find_server_jar(dir: &Path, kind: &str) -> Option<PathBuf> {
    let direct = dir.join("server.jar");
    if direct.is_file() && kind != "forge" {
        return Some(direct);
    }
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("jar") {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if kind == "forge" && (name.contains("forge") || name.contains("minecraft")) {
                return Some(p);
            }
        }
    }
    if direct.is_file() {
        return Some(direct);
    }
    None
}

async fn setup_paper_plugins(
    client: &reqwest::Client,
    app: &AppHandle,
    dir: &Path,
    mc: &str,
    with_geyser: bool,
) -> AppResult<()> {
    let plugins = dir.join("plugins");
    std::fs::create_dir_all(&plugins)?;

    let _ = download_github_release(client, app, "ViaVersion/ViaVersion", &plugins.join("ViaVersion.jar")).await;
    let _ = download_github_release(client, app, "ViaVersion/ViaBackwards", &plugins.join("ViaBackwards.jar")).await;
    let _ = download_modrinth_file(
        client,
        app,
        "skinsrestorer",
        &["paper", "spigot"],
        mc,
        plugins.join("SkinsRestorer.jar"),
    )
    .await;

    if with_geyser {
        let _ = download_geyser_plugin(client, app, "geyser", "paper", &plugins.join("Geyser-Spigot.jar")).await;
        let _ = download_geyser_plugin(client, app, "floodgate", "paper", &plugins.join("Floodgate-Spigot.jar")).await;
    }

    if mc.starts_with("1.8") {
        let _ = ensure_paraguacraft_badges_plugin(client, &plugins.join("ParaguacraftBadges.jar")).await;
    }
    Ok(())
}

async fn ensure_paraguacraft_badges_plugin(
    client: &reqwest::Client,
    dest: &Path,
) -> AppResult<()> {
    if dest.is_file() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 1_000 {
        return Ok(());
    }
    let url = "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/bundled/server/ParaguacraftBadges-1.0.0.jar";
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("part");
    net::download_one(client, &DownloadItem::new(url.to_string(), tmp.clone())).await?;
    if dest.exists() {
        let _ = std::fs::remove_file(dest);
    }
    std::fs::rename(&tmp, dest).map_err(|e| AppError::msg(format!("ParaguacraftBadges: {e}")))?;
    Ok(())
}

async fn setup_fabric_mods(
    client: &reqwest::Client,
    app: &AppHandle,
    dir: &Path,
    mc: &str,
    with_geyser: bool,
) -> AppResult<()> {
    let mods = dir.join("mods");
    std::fs::create_dir_all(&mods)?;

    let _ = download_modrinth_file(
        client,
        app,
        "fabric-api",
        &["fabric"],
        mc,
        mods.join("fabric-api.jar"),
    )
    .await;
    let _ = download_modrinth_file(
        client,
        app,
        "skinsrestorer",
        &["fabric"],
        mc,
        mods.join("SkinsRestorer.jar"),
    )
    .await;

    if with_geyser {
        let _ = download_geyser_plugin(client, app, "geyser", "fabric", &mods.join("Geyser-Fabric.jar")).await;
        let _ = download_geyser_plugin(client, app, "floodgate", "fabric", &mods.join("Floodgate-Fabric.jar")).await;
    }
    Ok(())
}

async fn download_github_release(
    client: &reqwest::Client,
    app: &AppHandle,
    repo: &str,
    dest: &Path,
) -> AppResult<()> {
    if dest.is_file() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 100_000 {
        return Ok(());
    }
    let api: Value = net::fetch_json(
        client,
        &format!("https://api.github.com/repos/{repo}/releases/latest"),
    )
    .await?;
    let assets = api["assets"].as_array().cloned().unwrap_or_default();
    let url = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| {
                    n.to_lowercase().ends_with(".jar")
                        && !n.to_lowercase().contains("sources")
                        && !n.to_lowercase().contains("javadoc")
                })
                .unwrap_or(false)
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .ok_or_else(|| AppError::msg(format!("GitHub: sin jar en {repo}")))?;
    net::download_all(
        client,
        vec![DownloadItem::new(url.to_string(), dest.to_path_buf())],
        1,
        app,
        &format!("plugin-{repo}"),
        dest.file_name().and_then(|n| n.to_str()).unwrap_or("plugin"),
    )
    .await?;
    Ok(())
}

async fn download_modrinth_file(
    client: &reqwest::Client,
    app: &AppHandle,
    slug: &str,
    loaders: &[&str],
    mc: &str,
    dest: PathBuf,
) -> AppResult<()> {
    if dest.is_file() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 100_000 {
        return Ok(());
    }
    let loaders_json = format!("[{}]", loaders.iter().map(|l| format!("\"{l}\"")).collect::<Vec<_>>().join(","));
    let mut url = format!(
        "https://api.modrinth.com/v2/project/{slug}/version?game_versions={}&loaders={}",
        net::url_encode(&format!("[\"{mc}\"]")),
        net::url_encode(&loaders_json)
    );
    let mut versions: Value = net::fetch_json(client, &url).await.unwrap_or(Value::Array(vec![]));
    if !versions.as_array().map(|a| !a.is_empty()).unwrap_or(false) {
        url = format!(
            "https://api.modrinth.com/v2/project/{slug}/version?loaders={}",
            net::url_encode(&loaders_json)
        );
        versions = net::fetch_json(client, &url).await?;
    }
    let ver = versions
        .as_array()
        .and_then(|a| a.first())
        .ok_or_else(|| AppError::msg(format!("Modrinth: {slug} no disponible para {mc}")))?;
    let files = ver["files"].as_array().cloned().unwrap_or_default();
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool().unwrap_or(false))
        .or_else(|| files.first())
        .ok_or_else(|| AppError::msg(format!("Modrinth: {slug} sin archivos")))?;
    let dl = file["url"].as_str().ok_or_else(|| AppError::msg("Modrinth: sin URL"))?;
    let filename = file["filename"].as_str().unwrap_or("mod.jar");
    let target = if dest.extension().is_some() {
        dest
    } else {
        dest.with_file_name(filename)
    };
    net::download_all(
        client,
        vec![DownloadItem::new(dl.to_string(), target.clone())],
        1,
        app,
        &format!("modrinth-{slug}"),
        filename,
    )
    .await?;
    Ok(())
}

async fn download_geyser_plugin(
    client: &reqwest::Client,
    app: &AppHandle,
    project: &str,
    platform: &str,
    dest: &Path,
) -> AppResult<()> {
    if dest.is_file() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 100_000 {
        return Ok(());
    }
    let url = format!(
        "https://download.geysermc.org/v2/projects/{project}/versions/latest/builds/latest/downloads/{platform}"
    );
    match net::download_all(
        client,
        vec![DownloadItem::new(url, dest.to_path_buf())],
        1,
        app,
        &format!("geyser-{project}"),
        dest.file_name().and_then(|n| n.to_str()).unwrap_or("geyser"),
    )
    .await
    {
        Ok(()) => Ok(()),
        Err(_) => {
            let loaders: Vec<&str> = if platform == "fabric" {
                vec!["fabric"]
            } else {
                vec!["paper", "spigot"]
            };
            let loaders_json = format!(
                "[{}]",
                loaders.iter().map(|l| format!("\"{l}\"")).collect::<Vec<_>>().join(",")
            );
            let url = format!(
                "https://api.modrinth.com/v2/project/{project}/version?loaders={}",
                net::url_encode(&loaders_json)
            );
            if let Ok(versions) = net::fetch_json::<Value>(client, &url).await {
                if let Some(ver) = versions.as_array().and_then(|a| a.first()) {
                    if let Some(file) = ver["files"].as_array().and_then(|f| f.first()) {
                        if let Some(dl) = file["url"].as_str() {
                            return net::download_all(
                                client,
                                vec![DownloadItem::new(dl.to_string(), dest.to_path_buf())],
                                1,
                                app,
                                &format!("geyser-mr-{project}"),
                                project,
                            )
                            .await
                            .map(|_| ());
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
