//! Provider NeoForge (maven.neoforged.net).

use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

const MAVEN: &str = "https://maven.neoforged.net/releases/net/neoforged/neoforge";

/// Deriva el prefijo NeoForge (`20.4.`, `21.0.`, ...) desde una version de MC.
fn neoforge_prefix(mc: &str) -> Option<String> {
    let parts: Vec<&str> = mc.split('.').collect();
    if parts.first() != Some(&"1") {
        return None;
    }
    let minor: u32 = parts.get(1)?.parse().ok()?;
    let patch: u32 = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
    Some(format!("{minor}.{patch}."))
}

pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    let Some(prefix) = neoforge_prefix(mc) else {
        return Ok(vec![]);
    };
    let url = format!("{MAVEN}/maven-metadata.xml");
    let xml = match net::fetch_bytes(client, &url).await {
        Ok(b) => String::from_utf8_lossy(&b).to_string(),
        Err(_) => return Ok(vec![]),
    };
    let mut matching: Vec<String> = super::parse_maven_versions(&xml)
        .into_iter()
        .filter(|v| v.starts_with(&prefix) && !v.contains("beta"))
        .collect();
    matching.reverse(); // ultima primero
    Ok(matching)
}

pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    version: &str,
) -> AppResult<String> {
    let installer_url = format!("{MAVEN}/{version}/neoforge-{version}-installer.jar");
    let installers = paths::java_dir().join("installers");
    std::fs::create_dir_all(&installers)?;
    let jar = installers.join(format!("neoforge-{version}-installer.jar"));

    net::download_all(
        client,
        vec![DownloadItem::new(installer_url, jar.clone())],
        1,
        app,
        "install-loader",
        &format!("Instalador NeoForge {version}"),
    )
    .await?;

    let java = super::installer_java(mc)?;
    let mc_dir = paths::default_minecraft_dir();
    std::fs::create_dir_all(&mc_dir)?;
    super::run_installer(
        java,
        vec![
            "-jar".into(),
            jar.to_string_lossy().to_string(),
            "--installClient".into(),
            mc_dir.to_string_lossy().to_string(),
        ],
    )
    .await?;

    super::find_installed_version_id(&["neoforge", version])
        .ok_or_else(|| AppError::msg("NeoForge se instalo pero no se encontro el perfil generado"))
}
