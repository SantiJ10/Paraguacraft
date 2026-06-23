//! Provider OptiFine (bmclapi mirror para listado/instalador standalone).
//! Descarga del mod JAR para Forge: sitio oficial optifine.net.

use std::path::Path;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

const BMCL: &str = "https://bmclapi2.bangbang93.com/optifine";
const OPTIFINE_SITE: &str = "https://optifine.net";
/// User-Agent tipo navegador: optifine.net rechaza peticiones sin ello.
const OPTIFINE_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Nombre de archivo del mod OptiFine para carpeta `mods/` (Forge).
pub fn mod_jar_filename(mc: &str, of_type: &str, of_patch: &str) -> String {
    format!("OptiFine_{mc}_{of_type}_{of_patch}.jar")
}

async fn fetch_optifine_html(client: &reqwest::Client, url: &str) -> AppResult<String> {
    let resp = client
        .get(url)
        .header(reqwest::header::USER_AGENT, OPTIFINE_UA)
        .send()
        .await?
        .error_for_status()?;
    Ok(String::from_utf8_lossy(&resp.bytes().await?).to_string())
}

/// Extrae la URL mirror `downloadx?f=...&x=...` de la pagina adloadx de OptiFine.
fn parse_downloadx_url(html: &str, filename: &str) -> AppResult<String> {
    let marker = format!("downloadx?f={filename}");
    let idx = html.find(&marker).ok_or_else(|| {
        AppError::msg(format!(
            "OptiFine: no se encontro enlace de descarga para {filename} en optifine.net"
        ))
    })?;
    let before = &html[..idx];
    let href_key = before
        .rfind("href='")
        .or_else(|| before.rfind("href=\""))
        .ok_or_else(|| AppError::msg("OptiFine: enlace de descarga mal formado"))?;
    let quote = if html[href_key..].starts_with("href='") {
        '\''
    } else {
        '"'
    };
    let href_start = href_key + if quote == '\'' { 6 } else { 6 };
    let rest = &html[href_start..];
    let href_end = rest.find(quote).ok_or_else(|| AppError::msg("OptiFine: enlace truncado"))?;
    let path = &rest[..href_end];
    Ok(format!("{OPTIFINE_SITE}/{path}"))
}

/// Descarga el JAR mod de OptiFine desde **optifine.net** (mirror oficial, sin BMCL).
pub async fn download_mod_jar_official(
    client: &reqwest::Client,
    mc: &str,
    of_type: &str,
    of_patch: &str,
    dest: &Path,
) -> AppResult<()> {
    let filename = mod_jar_filename(mc, of_type, of_patch);
    let adload = format!("{OPTIFINE_SITE}/adloadx?f={filename}");
    let html = fetch_optifine_html(client, &adload).await?;
    let dl_url = parse_downloadx_url(&html, &filename)?;

    let resp = client
        .get(&dl_url)
        .header(reqwest::header::USER_AGENT, OPTIFINE_UA)
        .send()
        .await?
        .error_for_status()?;
    let bytes = resp.bytes().await?;
    if bytes.len() < 4 || bytes[0] != 0x50 || bytes[1] != 0x4B {
        return Err(AppError::msg(
            "OptiFine: la respuesta de optifine.net no es un JAR valido",
        ));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        let _ = std::fs::remove_file(dest);
    }
    std::fs::write(dest, &bytes)?;
    Ok(())
}

/// Versiones OptiFine para `mc`, como "<type>_<patch>" (ej: "HD_U_M5").
pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    let url = format!("{BMCL}/{mc}");
    let builds: Value = match net::fetch_json(client, &url).await {
        Ok(v) => v,
        Err(_) => return Ok(vec![]),
    };
    let Some(arr) = builds.as_array() else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    for b in arr {
        let ty = b["type"].as_str().unwrap_or("");
        let patch = b["patch"].as_str().unwrap_or("");
        if !ty.is_empty() && !patch.is_empty() {
            out.push(format!("{ty}_{patch}"));
        }
    }
    Ok(out)
}

pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    version: &str,
) -> AppResult<String> {
    // version = "<type>_<patch>"; separamos por el ultimo '_'.
    let (of_type, of_patch) = version
        .rsplit_once('_')
        .ok_or_else(|| AppError::msg("Version OptiFine invalida"))?;

    let dl_url = format!("{BMCL}/{mc}/{of_type}/{of_patch}");
    let installers = paths::java_dir().join("installers");
    std::fs::create_dir_all(&installers)?;
    let jar = installers.join(format!("OptiFine_{mc}_{of_type}_{of_patch}.jar"));

    net::download_all(
        client,
        vec![DownloadItem::new(dl_url, jar.clone())],
        1,
        app,
        "install-loader",
        &format!("Descargando OptiFine {version}"),
    )
    .await?;

    // El instalador de OptiFine (clase optifine.Installer) crea la version.
    let java = super::installer_java(mc)?;
    let mc_dir = paths::default_minecraft_dir();
    super::run_installer(
        java,
        vec![
            "-cp".into(),
            jar.to_string_lossy().to_string(),
            "optifine.Installer".into(),
            mc_dir.to_string_lossy().to_string(),
        ],
    )
    .await?;

    super::find_installed_version_id(&[mc, "OptiFine"])
        .ok_or_else(|| AppError::msg("OptiFine se instalo pero no se encontro el perfil generado"))
}
