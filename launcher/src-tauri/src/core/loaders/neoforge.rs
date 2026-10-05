//! Provider NeoForge (maven.neoforged.net).

use tauri::{AppHandle, Manager};

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

const MAVEN: &str = "https://maven.neoforged.net/releases/net/neoforged/neoforge";

/// Deriva el prefijo de la rama NeoForge (`20.4.`, `21.0.`, `26.2.`) desde una
/// versión de Minecraft.
///
/// NeoForge numera cada rama como la MC, con dos salvedades:
/// - **1.20.1 no tiene NeoForge.** La rama más vieja es `20.2.` (= 1.20.2), así que
///   para 1.20.1 el prefijo `20.1.` no existe en el maven y la lista queda vacía
///   (la UI esconde NeoForge, que es lo correcto).
/// - El nuevo esquema de MC (`26.1`, `26.2`…) se traduce directo a `26.<minor>.`.
///
/// Se ignoran los sufijos de snapshot (`26.2-snapshot-8` → `26.2.`).
fn neoforge_prefix(mc: &str) -> Option<String> {
    let clean = mc.trim().split(['-', '+']).next().unwrap_or("");
    let parts: Vec<&str> = clean.split('.').collect();
    let major: u32 = parts.first()?.parse().ok()?;
    let minor: u32 = parts.get(1)?.parse().ok()?;
    if major == 1 {
        let patch: u32 = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
        Some(format!("{minor}.{patch}."))
    } else {
        Some(format!("{major}.{minor}."))
    }
}

/// `21.9.16-beta`, `26.1.0.0-alpha.5+snapshot-2`, … no son estables.
fn is_prerelease(v: &str) -> bool {
    let low = v.to_ascii_lowercase();
    ["-beta", "-alpha", "-rc", "snapshot", "-pre"]
        .iter()
        .any(|k| low.contains(k))
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
    let matching: Vec<String> = super::parse_maven_versions(&xml)
        .into_iter()
        .filter(|v| v.starts_with(&prefix))
        .collect();
    // Preferimos estables, pero si la rama solo publicó betas (1.20.3, 1.20.5, 1.21.7,
    // 1.21.9…) las ofrecemos igual: mejor una beta elegida a mano que ningún NeoForge.
    let mut out: Vec<String> = matching
        .iter()
        .filter(|v| !is_prerelease(v))
        .cloned()
        .collect();
    if out.is_empty() {
        out = matching;
    }
    super::sort_versions_desc(&mut out);
    Ok(out)
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

    let state = app.state::<crate::state::AppState>();
    let java = super::installer_java(app, &state, mc).await?;
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

    super::find_installed_version_id(&[&format!("{mc}-neoforge-{version}")])
        .or_else(|| super::find_installed_version_id(&["neoforge", version]))
        .ok_or_else(|| AppError::msg("NeoForge se instalo pero no se encontro el perfil generado"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ramas verificadas contra `install_profile.json` de cada installer del maven.
    #[test]
    fn prefix_matches_the_maven_branch() {
        assert_eq!(neoforge_prefix("1.20.2").as_deref(), Some("20.2."));
        assert_eq!(neoforge_prefix("1.20.4").as_deref(), Some("20.4."));
        assert_eq!(neoforge_prefix("1.20.6").as_deref(), Some("20.6."));
        assert_eq!(neoforge_prefix("1.21").as_deref(), Some("21.0."));
        assert_eq!(neoforge_prefix("1.21.1").as_deref(), Some("21.1."));
        assert_eq!(neoforge_prefix("1.21.11").as_deref(), Some("21.11."));
    }

    /// El esquema nuevo de MC (26.x) también tiene rama NeoForge: antes el `!= "1"`
    /// las ocultaba todas y el launcher ofrecía cero NeoForge en 26.1/26.2/26.3.
    #[test]
    fn prefix_covers_the_new_mc_versioning() {
        assert_eq!(neoforge_prefix("26.2").as_deref(), Some("26.2."));
        assert_eq!(neoforge_prefix("26.1.2").as_deref(), Some("26.1."));
        assert_eq!(neoforge_prefix("26.2-snapshot-8").as_deref(), Some("26.2."));
        assert_eq!(neoforge_prefix("1.21.9").as_deref(), Some("21.9."));
    }

    /// NeoForge no publicó para 1.20.1 (la rama más vieja es 20.2.x = 1.20.2):
    /// lista vacía → la UI esconde NeoForge, que es lo correcto.
    #[test]
    fn no_neoforge_for_1_20_1() {
        assert_eq!(neoforge_prefix("1.20.1").as_deref(), Some("20.1."));
    }

    #[test]
    fn prerelease_detection() {
        assert!(is_prerelease("21.9.16-beta"));
        assert!(is_prerelease("26.1.0.0-alpha.5+snapshot-2"));
        assert!(!is_prerelease("20.4.251"));
        assert!(!is_prerelease("21.1.255"));
    }
}
