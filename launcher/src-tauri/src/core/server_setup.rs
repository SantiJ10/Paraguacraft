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
            let java_major = crate::core::java::required_for_mc(&prof.mc_version);
            for msg in sanitize_server_mods(dir, &prof.mc_version, java_major) {
                crate::core::server_console::append(sid, &msg);
            }
            jar
        }
        "fabric-geyser" => {
            download_fabric(client, app, &prof.mc_version, &jar).await?;
            setup_fabric_mods(client, app, dir, sid, &prof.mc_version, true).await?;
            let java_major = crate::core::java::required_for_mc(&prof.mc_version);
            for msg in sanitize_server_mods(dir, &prof.mc_version, java_major) {
                crate::core::server_console::append(sid, &msg);
            }
            jar
        }
        "forge" | "neoforge" => {
            let res = setup_forge_style(client, app, dir, &prof.mc_version, &kind).await?;
            let _ = setup_forge_mods(client, app, dir, sid, &prof.mc_version, &kind).await;
            let java_major = crate::core::java::required_for_mc(&prof.mc_version);
            for msg in sanitize_server_mods(dir, &prof.mc_version, java_major) {
                crate::core::server_console::append(sid, &msg);
            }
            res
        }
        other => return Err(AppError::msg(format!("Tipo no implementado: {other}"))),
    };

    ensure_playit_plugin(client, dir, &kind).await;
    // Túnel siempre en playit.exe (independiente del proceso Java).
    if let Err(e) = ensure_playit_exe(client, app, dir).await {
        crate::core::server_console::append(
            sid,
            &format!("[launcher] ⚠ playit agent: {e}"),
        );
    }
    if kind.contains("geyser") {
        crate::core::server_console::append(
            sid,
            "[playit] Server Geyser: el launcher puede crear túneles Java + Bedrock con tu secret playit (sin plugin Paper en este modo).",
        );
    }
    Ok(result)
}

/// Desactiva jars playit en plugins/ para que el túnel viva en playit.exe.
async fn ensure_playit_plugin(_client: &reqwest::Client, dir: &Path, kind: &str) {
    let plugins = dir.join("plugins");
    let Ok(rd) = std::fs::read_dir(&plugins) else {
        return;
    };
    for e in rd.flatten() {
        let n = e.file_name().to_string_lossy().to_string();
        let low = n.to_ascii_lowercase();
        if low.contains("playit") && low.ends_with(".jar") && !low.ends_with(".disabled") {
            let dest = e.path().with_file_name(format!("{n}.disabled"));
            let _ = std::fs::rename(e.path(), dest);
        }
    }
    let _ = kind;
}

/// Rellena dependencias estándar sin sobrescribir jar/configs existentes.
pub async fn soft_ensure_launcher_deps(
    client: &reqwest::Client,
    app: &AppHandle,
    prof: &ServerProfile,
    dir: &Path,
) -> AppResult<()> {
    let kind = normalize_server_type(&prof.server_type)?;
    let sid = prof.id.as_str();
    let _ = std::fs::create_dir_all(dir.join("plugins"));
    let _ = std::fs::create_dir_all(dir.join("mods"));

    // Quitar JARs incompatibles (p. ej. SkinsRestorer 26.x o mods solo-cliente) antes de re-descargar.
    if kind.starts_with("fabric") || is_forge_style(&kind) {
        let java_major = crate::core::java::required_for_mc(&prof.mc_version);
        for msg in sanitize_server_mods(dir, &prof.mc_version, java_major) {
            crate::core::server_console::append(sid, &msg);
        }
    }

    if kind.starts_with("paper") {
        let with_geyser = kind.contains("geyser");
        // setup_paper_* ya saltan/ toleran fallos; no reescribe properties del user.
        let _ = setup_paper_plugins(client, app, dir, sid, &prof.mc_version, with_geyser).await;
        ensure_playit_plugin(client, dir, &kind).await;
        if let Err(e) = ensure_playit_exe(client, app, dir).await {
            crate::core::server_console::append(sid, &format!("[launcher] ⚠ playit agent: {e}"));
        }
    } else if kind.starts_with("fabric") {
        let with_geyser = kind.contains("geyser");
        let _ = setup_fabric_mods(client, app, dir, sid, &prof.mc_version, with_geyser).await;
        if let Err(e) = ensure_playit_exe(client, app, dir).await {
            crate::core::server_console::append(sid, &format!("[launcher] ⚠ playit agent: {e}"));
        }
    } else if is_forge_style(&kind) {
        let _ = setup_forge_mods(client, app, dir, sid, &prof.mc_version, &kind).await;
        if let Err(e) = ensure_playit_exe(client, app, dir).await {
            crate::core::server_console::append(sid, &format!("[launcher] ⚠ playit agent: {e}"));
        }
    }
    Ok(())
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
            &["paper", "spigot", "bukkit"],
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
    let packet_fixer = mods.join("packet-fixer.jar");
    try_optional(server_id, "Packet Fixer", || {
        download_modrinth_file(client, app, "packet-fixer", &["fabric"], mc, packet_fixer)
    })
    .await;
    let ferrite = mods.join("ferrite-core.jar");
    try_optional(server_id, "FerriteCore", || {
        download_modrinth_file(client, app, "ferrite-core", &["fabric"], mc, ferrite)
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

/// Instala mods de red/estabilidad para servidores Forge y NeoForge (evita desconexiones
/// «Bad packet ID» / «Timed out» / «Packet too large» al entrar con ping alto o túnel Playit).
async fn setup_forge_mods(
    client: &reqwest::Client,
    app: &AppHandle,
    dir: &Path,
    server_id: &str,
    mc: &str,
    kind: &str,
) -> AppResult<()> {
    let mods = dir.join("mods");
    std::fs::create_dir_all(&mods)?;
    let loaders: &'static [&'static str] = if kind == "neoforge" {
        &["neoforge"]
    } else {
        &["forge"]
    };

    if kind != "neoforge" && mc == "1.20.1" {
        for fname in ["connectivity-1.20.1-7.6.jar", "cupboard-1.20.1-3.5.jar"] {
            let dest = mods.join(fname);
            try_optional(server_id, fname, || {
                ensure_bundled_optimized_mod(client, fname, &dest)
            })
            .await;
        }
    }

    let packet_fixer = mods.join("packet-fixer.jar");
    try_optional(server_id, "Packet Fixer", || {
        download_modrinth_file(client, app, "packet-fixer", loaders, mc, packet_fixer)
    })
    .await;

    let ferrite = mods.join("ferrite-core.jar");
    try_optional(server_id, "FerriteCore", || {
        download_modrinth_file(client, app, "ferrite-core", loaders, mc, ferrite)
    })
    .await;

    let modernfix = mods.join("modernfix.jar");
    try_optional(server_id, "ModernFix", || {
        download_modrinth_file(client, app, "modernfix", loaders, mc, modernfix)
    })
    .await;

    Ok(())
}

async fn ensure_bundled_optimized_mod(
    client: &reqwest::Client,
    filename: &str,
    dest: &Path,
) -> AppResult<()> {
    if dest.is_file() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 10_000 {
        return Ok(());
    }
    // Intentar copiar desde bundled/optimized-mods local primero
    let candidates = [
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../bundled/optimized-mods")
            .join(filename),
        crate::core::paths::data_dir()
            .join("cache/optimized-mods")
            .join(filename),
    ];
    for src in &candidates {
        if src.is_file() && src.metadata().map(|m| m.len()).unwrap_or(0) > 10_000 {
            std::fs::copy(src, dest)?;
            return Ok(());
        }
    }
    let url = format!(
        "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/bundled/optimized-mods/{filename}"
    );
    let tmp = dest.with_extension("part");
    net::download_one(client, &DownloadItem::new(url, tmp.clone())).await?;
    if dest.exists() {
        let _ = std::fs::remove_file(dest);
    }
    std::fs::rename(&tmp, dest)?;
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
    let java_major = crate::core::java::required_for_mc(mc);
    if dest.is_file()
        && dest.metadata().map(|m| m.len()).unwrap_or(0) > 100_000
        && !jar_incompatible_with_server(dest, mc, java_major)
    {
        return Ok(());
    }
    if dest.is_file() && jar_incompatible_with_server(dest, mc, java_major) {
        let _ = quarantine_mod_jar(dest);
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

fn mc_candidates_for_modrinth(mc: &str) -> Vec<&str> {
    match mc {
        "1.8.9" => vec!["1.8.9", "1.8.8", "1.8"],
        "1.12.2" => vec!["1.12.2", "1.12"],
        other => vec![other],
    }
}

/// Descarga directa de un plugin/mod desde Modrinth (usado también como fallback de Hangar).
pub async fn download_modrinth_plugin_direct(
    client: &reqwest::Client,
    slug: &str,
    loaders: &[&str],
    mc: &str,
    dest: &Path,
) -> AppResult<()> {
    let java_major = crate::core::java::required_for_mc(mc);
    let loaders_json = format!(
        "[{}]",
        loaders
            .iter()
            .map(|l| format!("\"{l}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    let mc_cands = mc_candidates_for_modrinth(mc);
    let mcs_json = format!(
        "[{}]",
        mc_cands
            .iter()
            .map(|v| format!("\"{v}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    let url = format!(
        "https://api.modrinth.com/v2/project/{slug}/version?game_versions={}&loaders={}",
        net::url_encode(&mcs_json),
        net::url_encode(&loaders_json)
    );
    let versions: Value = net::fetch_json(client, &url).await.unwrap_or(Value::Array(vec![]));
    let arr = versions
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or_else(|| AppError::msg(format!("Modrinth: {slug} no disponible para {mc}")))?;

    for ver in arr.iter().take(6) {
        let game_versions: Vec<String> = ver["game_versions"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        if !game_versions.is_empty()
            && !game_versions.iter().any(|v| mc_cands.contains(&v.as_str()))
        {
            continue;
        }
        let files = ver["files"].as_array().cloned().unwrap_or_default();
        let Some(file) = files
            .iter()
            .find(|f| f["primary"].as_bool().unwrap_or(false))
            .or_else(|| files.first())
        else {
            continue;
        };
        let Some(dl) = file["url"].as_str() else {
            continue;
        };
        let tmp = dest.with_extension("part");
        net::download_one(client, &DownloadItem::new(dl.to_string(), tmp.clone())).await?;
        if jar_incompatible_with_server(&tmp, mc, java_major) {
            let _ = std::fs::remove_file(&tmp);
            continue;
        }
        if dest.exists() {
            let _ = std::fs::remove_file(dest);
        }
        std::fs::rename(&tmp, dest)?;
        return Ok(());
    }
    Err(AppError::msg(format!(
        "Modrinth: {slug} sin versión compatible con MC {mc} (Java {java_major})"
    )))
}

async fn download_modrinth_file(
    client: &reqwest::Client,
    app: &AppHandle,
    slug: &str,
    loaders: &[&str],
    mc: &str,
    dest: PathBuf,
) -> AppResult<()> {
    let java_major = crate::core::java::required_for_mc(mc);
    if dest.is_file()
        && slug.eq_ignore_ascii_case("skinsrestorer")
        && java_major >= 17
        && skinsrestorer_needs_elyby_upgrade(&dest)
    {
        let _ = quarantine_mod_jar(&dest);
    }
    if dest.is_file()
        && dest.metadata().map(|m| m.len()).unwrap_or(0) > 20_000
        && !jar_incompatible_with_server(&dest, mc, java_major)
    {
        return Ok(());
    }
    if dest.is_file() && jar_incompatible_with_server(&dest, mc, java_major) {
        let _ = quarantine_mod_jar(&dest);
    }

    let loaders_json = format!(
        "[{}]",
        loaders
            .iter()
            .map(|l| format!("\"{l}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    let mc_cands = mc_candidates_for_modrinth(mc);
    let mcs_json = format!(
        "[{}]",
        mc_cands
            .iter()
            .map(|v| format!("\"{v}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    let url = format!(
        "https://api.modrinth.com/v2/project/{slug}/version?game_versions={}&loaders={}",
        net::url_encode(&mcs_json),
        net::url_encode(&loaders_json)
    );
    let versions: Value = net::fetch_json(client, &url).await.unwrap_or(Value::Array(vec![]));
    let arr = versions
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or_else(|| AppError::msg(format!("Modrinth: {slug} no disponible para {mc}")))?;

    for ver in arr.iter().take(6) {
        let game_versions: Vec<String> = ver["game_versions"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        if !game_versions.is_empty()
            && !game_versions.iter().any(|v| mc_cands.contains(&v.as_str()))
        {
            continue;
        }
        let files = ver["files"].as_array().cloned().unwrap_or_default();
        let Some(file) = files
            .iter()
            .find(|f| f["primary"].as_bool().unwrap_or(false))
            .or_else(|| files.first())
        else {
            continue;
        };
        let Some(dl) = file["url"].as_str() else {
            continue;
        };
        let filename = file["filename"].as_str().unwrap_or("mod.jar");
        let target = if dest.extension().is_some() {
            dest.clone()
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
        if jar_incompatible_with_server(&target, mc, java_major) {
            let _ = std::fs::remove_file(&target);
            continue;
        }
        return Ok(());
    }
    Err(AppError::msg(format!(
        "Modrinth: {slug} sin build compatible con MC {mc} (Java {java_major})"
    )))
}

const CLIENT_ONLY_MOD_PREFIXES: &[&str] = &[
    "oculus-",
    "iris-",
    "sodium-",
    "embeddium-",
    "rubidium-",
    "kryptonreforged",
    "immediatelyfast-",
    "3dskinlayers-",
    "skinlayers3d-",
    "entity_model_features_",
    "entity_texture_features_",
    "moreculling-",
    "dynamic-fps-",
    "dynamic_fps-",
    "reeses-sodium-options",
    "reeses_sodium_options",
    "sodium-extra-",
    "sodium_extra-",
    "custom-crosshair-mod",
    "freelook-",
    "chat_heads-",
    "toastcontrol-",
    "cleanview-",
    "controlling-",
    "searchables-",
    "configured-",
    "catalogue-",
    "modmenu-",
    "zoomify-",
    "lambdynamiclights-",
    "continuity-",
    "citresewn-",
    "bobby-",
    "farsight-",
    "cubes-without-borders",
    " crashassistant",
    "crashassistant-",
];

fn is_known_client_only_jar_name(lower_name: &str) -> bool {
    CLIENT_ONLY_MOD_PREFIXES
        .iter()
        .any(|p| lower_name.starts_with(p) || lower_name.contains(p))
}

/// Cuarentena de JARs del launcher o del pack incompatibles con MC/Java del server o solo-cliente.
/// Devuelve mensajes para la consola del launcher.
pub fn sanitize_server_mods(dir: &Path, mc: &str, java_major: u32) -> Vec<String> {
    let mut msgs = Vec::new();
    let effective_java = if java_major > 0 {
        java_major
    } else {
        crate::core::java::required_for_mc(mc)
    };
    for sub in ["mods", "plugins"] {
        let folder = dir.join(sub);
        let Ok(rd) = std::fs::read_dir(&folder) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("jar"))
                != Some(true)
            {
                continue;
            }
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let force_native_math =
                name.contains("c2me") && name.contains("natives-math") && effective_java < 25;
            let client_only = sub == "mods" && is_known_client_only_jar_name(&name);
            if force_native_math || client_only || jar_incompatible_with_server(&p, mc, effective_java) {
                match quarantine_mod_jar(&p) {
                    Ok(dest) => {
                        msgs.push(format!(
                            "[launcher] ⚠ Cuarentena ({}) en {sub}/: {} → {}",
                            if client_only {
                                "mod solo-cliente".to_string()
                            } else {
                                format!("incompatible con MC {mc} / Java {effective_java}")
                            },
                            p.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                            dest.file_name().and_then(|n| n.to_str()).unwrap_or("?")
                        ));
                    }
                    Err(err) => {
                        msgs.push(format!(
                            "[launcher] ⚠ No se pudo aislar {}: {err}",
                            p.file_name().and_then(|n| n.to_str()).unwrap_or("?")
                        ));
                    }
                }
            }
        }
    }
    msgs
}

fn quarantine_mod_jar(path: &Path) -> AppResult<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::msg("Ruta de mod inválida"))?;
    let quarantine = parent.join(".paraguacraft-incompatible");
    std::fs::create_dir_all(&quarantine)?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mod.jar");
    let mut dest = quarantine.join(name);
    if dest.exists() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        dest = quarantine.join(format!("{stamp}_{name}"));
    }
    if std::fs::rename(path, &dest).is_err() {
        std::fs::copy(path, &dest)
            .map_err(|e| AppError::msg(format!("Cuarentena de mod (copia): {e}")))?;
        std::fs::remove_file(path)
            .map_err(|e| AppError::msg(format!("Cuarentena de mod (borrar origen): {e}")))?;
    }
    Ok(dest)
}

/// Lee `fabric.mod.json` / `quilt.mod.json` y detecta depends MC/Java fuera de rango.
/// SkinsRestorer 15.11+ trae Ely.by. Si se puede leer la versión y es más vieja, hay que actualizar.
fn skinsrestorer_needs_elyby_upgrade(path: &Path) -> bool {
    match skinsrestorer_jar_version(path) {
        Some(v) => v < (15, 11, 0),
        None => false,
    }
}

fn skinsrestorer_jar_version(path: &Path) -> Option<(u32, u32, u32)> {
    let file = std::fs::File::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    for entry_name in ["paper-plugin.yml", "plugin.yml", "fabric.mod.json"] {
        let Ok(mut entry) = zip.by_name(entry_name) else {
            continue;
        };
        let mut buf = String::new();
        if std::io::Read::read_to_string(&mut entry, &mut buf).is_err() {
            continue;
        }
        if let Some(v) = parse_plugin_version(&buf) {
            return Some(v);
        }
    }
    None
}

fn parse_plugin_version(text: &str) -> Option<(u32, u32, u32)> {
    for line in text.lines() {
        let t = line.trim();
        let raw = if let Some(rest) = t.strip_prefix("version:") {
            rest.trim().trim_matches('"').trim_matches('\'').to_string()
        } else if let Some(rest) = t.strip_prefix("\"version\"") {
            let rest = rest.trim().trim_start_matches(':').trim();
            rest.trim_matches(',').trim().trim_matches('"').to_string()
        } else {
            continue;
        };
        let ver = raw.split(['-', '+']).next().unwrap_or(&raw);
        let mut parts = ver.split('.');
        let maj: u32 = parts.next()?.parse().ok()?;
        let min: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let pat: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
        return Some((maj, min, pat));
    }
    None
}

/// Lee el bytecode `.class` principal (excluyendo `META-INF/versions/` y `module-info.class`)
/// para detectar qué versión de Java exige un JAR (52 = Java 8, 61 = Java 17, 65 = Java 21, 69 = Java 25).
fn read_jar_bytecode_java_major(path: &Path) -> Option<u32> {
    let file = std::fs::File::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    for i in 0..zip.len() {
        let Ok(mut entry) = zip.by_index(i) else {
            continue;
        };
        let name = entry.name().to_string();
        if !name.ends_with(".class")
            || name.starts_with("META-INF/")
            || name.ends_with("module-info.class")
            || name.ends_with("package-info.class")
        {
            continue;
        }
        let mut header = [0u8; 8];
        if std::io::Read::read_exact(&mut entry, &mut header).is_ok() {
            if &header[0..4] == &[0xCA, 0xFE, 0xBA, 0xBE] {
                let major = u16::from_be_bytes([header[6], header[7]]) as u32;
                if major >= 45 {
                    return Some(major.saturating_sub(44));
                }
            }
        }
    }
    None
}

fn jar_incompatible_with_server(path: &Path, mc: &str, java_major: u32) -> bool {
    let effective_java = if java_major > 0 {
        java_major
    } else {
        crate::core::java::required_for_mc(mc)
    };

    if effective_java > 0 {
        if let Some(class_java) = read_jar_bytecode_java_major(path) {
            if class_java > effective_java {
                return true;
            }
        }
    }

    let Some(meta) = read_fabric_mod_json(path) else {
        return false;
    };
    if meta.get("environment").and_then(|e| e.as_str()) == Some("client") {
        return true;
    }
    let depends = meta.get("depends").cloned().unwrap_or(Value::Null);

    if effective_java > 0 {
        if let Some(min_java) = parse_java_dep_min(&depends["java"]) {
            if effective_java < min_java {
                return true;
            }
        }
    }

    let mc_dep = depends.get("minecraft").cloned().unwrap_or(Value::Null);
    if mc_dep_incompatible_with(&mc_dep, mc) {
        return true;
    }

    false
}

fn read_fabric_mod_json(path: &Path) -> Option<Value> {
    let file = std::fs::File::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    for entry_name in ["fabric.mod.json", "quilt.mod.json"] {
        if let Ok(mut entry) = zip.by_name(entry_name) {
            let mut buf = String::new();
            if std::io::Read::read_to_string(&mut entry, &mut buf).is_ok() {
                if let Ok(v) = serde_json::from_str::<Value>(&buf) {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn parse_java_dep_min(dep: &Value) -> Option<u32> {
    let s = match dep {
        Value::String(s) => s.as_str(),
        Value::Array(a) => a.first()?.as_str()?,
        _ => return None,
    };
    // ">=25", ">=21", "21", "~21"
    let digits: String = s
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// MC 1.21.x vs depends que piden esquema año (26.x) de Fabric loader reciente.
fn mc_dep_incompatible_with(dep: &Value, server_mc: &str) -> bool {
    let req = match dep {
        Value::String(s) => s.clone(),
        Value::Array(a) => a
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        _ => return false,
    };
    if req.is_empty() {
        return false;
    }
    let server_year_style = server_mc
        .split('.')
        .next()
        .and_then(|p| p.parse::<u32>().ok())
        .map(|n| n >= 26)
        .unwrap_or(false);
    // Server 1.x: cualquier depends que mencione solo 26.x es incompatible.
    if !server_year_style {
        let wants_26 = req.contains("26.") || req.contains("~26") || req.contains(">=26") || req.contains("=26");
        if wants_26 && !req.contains("1.2") {
            return true;
        }
        // Si declara lista de 1.21.x y no incluye el exacto, no bloqueamos (rango ~1.21).
    }
    // Server 26.x con depends solo 1.21: menos común; no forzar.
    false
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plugin_yml_and_fabric_json_versions() {
        assert_eq!(
            parse_plugin_version("name: SkinsRestorer\nversion: 15.10.2\n"),
            Some((15, 10, 2))
        );
        assert_eq!(
            parse_plugin_version("version: '15.11.0'\n"),
            Some((15, 11, 0))
        );
        assert_eq!(
            parse_plugin_version("{\n  \"id\": \"skinsrestorer\",\n  \"version\": \"15.11.0-SNAPSHOT\"\n}\n"),
            Some((15, 11, 0))
        );
        assert!(parse_plugin_version("version: 15.10.2").unwrap() < (15, 11, 0));
        assert!(parse_plugin_version("version: 15.11.0").unwrap() >= (15, 11, 0));
    }
}
