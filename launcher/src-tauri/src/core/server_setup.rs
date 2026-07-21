//! Preparación on-demand de servidores (Paper, Fabric, Forge, Geyser).

use std::path::{Path, PathBuf};

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::server_hangar;
use crate::core::server_manager;
use crate::core::servers::ServerProfile;
use crate::error::{AppError, AppResult};

pub const SERVER_TYPES: &[&str] = &[
    "paper",
    "paper-geyser",
    "fabric",
    "fabric-geyser",
    "forge",
    "neoforge",
];

/// Loaders que usan el mecanismo de instalador Maven `--installServer` (Forge/NeoForge):
/// desde MC 1.17 no generan un jar único, sino `run.bat`/`run.sh` + `libraries/`.
pub fn is_forge_style(kind: &str) -> bool {
    kind == "forge" || kind == "neoforge"
}

pub fn normalize_server_type(raw: &str) -> AppResult<String> {
    match raw.trim().to_lowercase().replace('_', "-").as_str() {
        "paper" => Ok("paper".into()),
        "paper-geyser" | "paper+geyser" => Ok("paper-geyser".into()),
        "fabric" => Ok("fabric".into()),
        "fabric-geyser" | "fabric+geyser" => Ok("fabric-geyser".into()),
        "forge" => Ok("forge".into()),
        "neoforge" => Ok("neoforge".into()),
        other => Err(AppError::msg(format!(
            "Tipo de servidor invalido: {other}. Usa: paper, paper-geyser, fabric, fabric-geyser, forge, neoforge."
        ))),
    }
}

pub fn type_label(t: &str) -> &'static str {
    match t {
        "paper-geyser" => "Paper + Geyser",
        "fabric-geyser" => "Fabric + Geyser",
        "fabric" => "Fabric",
        "forge" => "Forge",
        "neoforge" => "NeoForge",
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
    let sid = prof.id.as_str();

    let result = match kind.as_str() {
        "paper" => {
            download_paper(client, app, &prof.mc_version, &jar).await?;
            setup_paper_plugins(client, app, dir, sid, &prof.mc_version, false).await?;
            jar
        }
        "paper-geyser" => {
            download_paper(client, app, &prof.mc_version, &jar).await?;
            setup_paper_plugins(client, app, dir, sid, &prof.mc_version, true).await?;
            jar
        }
        "fabric" => {
            download_fabric(client, app, &prof.mc_version, &jar).await?;
            setup_fabric_mods(client, app, dir, sid, &prof.mc_version, false).await?;
            jar
        }
        "fabric-geyser" => {
            download_fabric(client, app, &prof.mc_version, &jar).await?;
            setup_fabric_mods(client, app, dir, sid, &prof.mc_version, true).await?;
            jar
        }
        "forge" | "neoforge" => setup_forge_style(client, app, dir, &prof.mc_version, &kind).await?,
        other => return Err(AppError::msg(format!("Tipo no implementado: {other}"))),
    };

    ensure_playit_exe(client, app, dir).await?;
    Ok(result)
}

const PLAYIT_MIN_BYTES: u64 = 2_000_000;

/// Descarga directa sin GitHub API (evita 403 rate limit al preparar servidores).
const PLAYIT_WIN_URLS: &[&str] = &[
    "https://github.com/playit-cloud/playit-agent/releases/download/v1.0.10/playit-windows-x86_64-signed.exe",
    "https://github.com/playit-cloud/playit-agent/releases/download/v1.0.10/playit-windows-x86_64.exe",
];

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

    let tmp = playit_path.with_extension("part");
    for url in PLAYIT_WIN_URLS {
        if net::download_one(client, &DownloadItem::new(url.to_string(), tmp.clone()))
            .await
            .is_err()
        {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        if tmp.metadata().map(|m| m.len()).unwrap_or(0) < PLAYIT_MIN_BYTES {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        if playit_path.exists() {
            let _ = std::fs::remove_file(&playit_path);
        }
        std::fs::rename(&tmp, &playit_path)?;
        let _ = std::fs::copy(&playit_path, &global);
        return Ok(());
    }

    Err(AppError::msg(
        "No se pudo descargar playit.exe (URLs directas fallaron). Descargalo manualmente desde playit.gg.",
    ))
}

fn write_eula(dir: &Path) -> AppResult<()> {
    server_manager::write_eula(dir)
}

/// Plugins/mods opcionales (ViaVersion, Geyser, SkinsRestorer…): un fallo no debe abortar
/// la preparación del server.jar, pero sí debe quedar registrado en la consola integrada.
async fn try_optional<F, Fut>(server_id: &str, label: &str, f: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<()>>,
{
    if let Err(e) = f().await {
        crate::core::server_console::append(
            server_id,
            &format!("[launcher] ⚠ {label}: {e}"),
        );
    }
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

async fn setup_forge_style(
    client: &reqwest::Client,
    app: &AppHandle,
    dir: &Path,
    mc: &str,
    kind: &str,
) -> AppResult<PathBuf> {
    if let Some(existing) = find_server_jar(dir, kind) {
        return Ok(existing);
    }
    if kind == "neoforge" {
        server_manager::download_neoforge_server(client, app, mc, dir).await
    } else {
        server_manager::download_forge_server(client, app, mc, dir).await
    }
}

/// Localiza el jar/launcher ejecutable del servidor en `dir`. Para Forge/NeoForge (1.17+)
/// no hay un jar único: cuenta como "ya preparado" si existe `run.bat`/`run.sh`.
pub fn find_server_jar(dir: &Path, kind: &str) -> Option<PathBuf> {
    let direct = dir.join("server.jar");
    if direct.is_file() && !is_forge_style(kind) {
        return Some(direct);
    }
    if is_forge_style(kind) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) != Some("jar") {
                    continue;
                }
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.contains(kind) || name.contains("minecraft") || name.ends_with("-universal.jar") {
                    return Some(p);
                }
            }
        }
        for script in ["run.bat", "run.sh"] {
            let p = dir.join(script);
            if p.is_file() {
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
    server_id: &str,
    mc: &str,
    with_geyser: bool,
) -> AppResult<()> {
    let plugins = dir.join("plugins");
    std::fs::create_dir_all(&plugins)?;

    let via_version = plugins.join("ViaVersion.jar");
    try_optional(server_id, "ViaVersion", || {
        download_hangar_plugin(client, dir, "ViaVersion", "ViaVersion", mc, &via_version)
    })
    .await;
    let via_backwards = plugins.join("ViaBackwards.jar");
    try_optional(server_id, "ViaBackwards", || {
        download_hangar_plugin(client, dir, "ViaVersion", "ViaBackwards", mc, &via_backwards)
    })
    .await;
    let skins = plugins.join("SkinsRestorer.jar");
    try_optional(server_id, "SkinsRestorer", || {
        download_modrinth_file(
            client,
            app,
            "skinsrestorer",
            &["paper", "spigot"],
            mc,
            skins,
        )
    })
    .await;

    if with_geyser {
        let geyser = plugins.join("Geyser-Spigot.jar");
        try_optional(server_id, "Geyser", || {
            download_geyser_plugin(client, app, "geyser", "paper", mc, &geyser)
        })
        .await;
        let floodgate = plugins.join("Floodgate-Spigot.jar");
        try_optional(server_id, "Floodgate", || {
            download_geyser_plugin(client, app, "floodgate", "paper", mc, &floodgate)
        })
        .await;
    }

    let badges_name = if mc.starts_with("1.8") {
        "ParaguacraftBadges-1.0.0.jar"
    } else {
        "ParaguacraftBadges-Paper-1.0.0.jar"
    };
    let badges = plugins.join("ParaguacraftBadges.jar");
    try_optional(server_id, "ParaguacraftBadges", || {
        ensure_paraguacraft_badges_plugin(client, badges_name, &badges)
    })
    .await;
    Ok(())
}

async fn ensure_paraguacraft_badges_plugin(
    client: &reqwest::Client,
    bundled_name: &str,
    dest: &Path,
) -> AppResult<()> {
    if dest.is_file() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 1_000 {
        return Ok(());
    }
    let url = format!(
        "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/bundled/server/{bundled_name}"
    );
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("part");
    net::download_one(client, &DownloadItem::new(url, tmp.clone())).await?;
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
    server_id: &str,
    mc: &str,
    with_geyser: bool,
) -> AppResult<()> {
    let mods = dir.join("mods");
    std::fs::create_dir_all(&mods)?;

    let fabric_api = mods.join("fabric-api.jar");
    try_optional(server_id, "Fabric API", || {
        download_modrinth_file(client, app, "fabric-api", &["fabric"], mc, fabric_api)
    })
    .await;
    let skins = mods.join("SkinsRestorer.jar");
    try_optional(server_id, "SkinsRestorer", || {
        download_modrinth_file(
            client,
            app,
            "skinsrestorer",
            &["fabric"],
            mc,
            skins,
        )
    })
    .await;

    if with_geyser {
        let geyser = mods.join("Geyser-Fabric.jar");
        try_optional(server_id, "Geyser", || {
            download_geyser_plugin(client, app, "geyser", "fabric", mc, &geyser)
        })
        .await;
        let floodgate = mods.join("Floodgate-Fabric.jar");
        try_optional(server_id, "Floodgate", || {
            download_geyser_plugin(client, app, "floodgate", "fabric", mc, &floodgate)
        })
        .await;
    }
    Ok(())
}

async fn download_hangar_plugin(
    client: &reqwest::Client,
    dir: &Path,
    owner: &str,
    slug: &str,
    mc: &str,
    dest: &Path,
) -> AppResult<()> {
    if dest.is_file() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 100_000 {
        return Ok(());
    }
    let installed = server_hangar::install_plugin(client, dir, owner, slug, mc, false).await?;
    let default_path = dir.join("plugins").join(&installed);
    if default_path != dest && default_path.is_file() {
        if dest.exists() {
            let _ = std::fs::remove_file(dest);
        }
        std::fs::rename(&default_path, dest)?;
    }
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

const GEYSER_DOWNLOAD_API: &str = "https://download.geysermc.org/v2";

/// La API de descargas de GeyserMC ya no acepta `versions/latest/builds/latest` (alias
/// retirado); hay que resolver la última versión y el último build a mano. Tampoco existe
/// una clave de plataforma "paper": el jar de Spigot cubre Spigot y Paper por igual.
fn geyser_platform_key(platform: &str) -> &'static str {
    if platform == "fabric" { "fabric" } else { "spigot" }
}

async fn download_geyser_official(
    client: &reqwest::Client,
    app: &AppHandle,
    project: &str,
    platform: &str,
    dest: &Path,
) -> AppResult<()> {
    let info: Value =
        net::fetch_json(client, &format!("{GEYSER_DOWNLOAD_API}/projects/{project}")).await?;
    let version = info["versions"]
        .as_array()
        .and_then(|a| a.last())
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::msg(format!("GeyserMC: sin versiones publicadas para {project}")))?
        .to_string();
    let builds: Value = net::fetch_json(
        client,
        &format!("{GEYSER_DOWNLOAD_API}/projects/{project}/versions/{version}/builds"),
    )
    .await?;
    let key = geyser_platform_key(platform);
    let build_num = builds["builds"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|b| !b["downloads"][key].is_null())
        .filter_map(|b| b["build"].as_u64())
        .max()
        .ok_or_else(|| {
            AppError::msg(format!("GeyserMC: {project} {version} no publica build para '{key}'"))
        })?;
    let url = format!(
        "{GEYSER_DOWNLOAD_API}/projects/{project}/versions/{version}/builds/{build_num}/downloads/{key}"
    );
    net::download_all(
        client,
        vec![DownloadItem::new(url, dest.to_path_buf())],
        1,
        app,
        &format!("geyser-{project}"),
        dest.file_name().and_then(|n| n.to_str()).unwrap_or("geyser"),
    )
    .await
}

async fn download_geyser_plugin(
    client: &reqwest::Client,
    app: &AppHandle,
    project: &str,
    platform: &str,
    mc: &str,
    dest: &Path,
) -> AppResult<()> {
    if dest.is_file() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 100_000 {
        return Ok(());
    }
    match download_geyser_official(client, app, project, platform, dest).await {
        Ok(()) => Ok(()),
        // Fallback Modrinth (mismo helper con doble intento con/sin filtro de mc que usan
        // fabric-api/SkinsRestorer más arriba) si la web oficial no tiene ese build/plataforma.
        Err(_) => {
            let loaders: &[&str] = if platform == "fabric" { &["fabric"] } else { &["paper", "spigot"] };
            download_modrinth_file(client, app, project, loaders, mc, dest.to_path_buf()).await
        }
    }
}
