//! Descarga de JARs de servidor local (Paper, Fabric, Forge).

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::core::loaders::{self, forge};
use crate::core::net::{self, DownloadItem};
use crate::error::{AppError, AppResult};

pub fn write_eula(dir: &Path) -> AppResult<()> {
    std::fs::write(dir.join("eula.txt"), "eula=true\n")?;
    Ok(())
}

pub async fn download_paper_server(
    client: &reqwest::Client,
    app: &AppHandle,
    mc: &str,
    dest: &Path,
) -> AppResult<()> {
    write_eula(dest.parent().unwrap_or(dest))?;
    let url = format!("https://api.papermc.io/v2/projects/paper/versions/{mc}/builds");
    let builds: Value = net::fetch_json(client, &url).await?;
    let arr = builds["builds"].as_array().cloned().unwrap_or_default();
    let build = arr
        .iter()
        .rev()
        .find(|b| b["channel"].as_str() == Some("default"))
        .or_else(|| arr.last())
        .ok_or_else(|| AppError::msg(format!("No hay builds de Paper para {mc}")))?;
    let build_num = build["build"].as_u64().unwrap_or(0);
    let jar_name = build["downloads"]["application"]["name"]
        .as_str()
        .ok_or_else(|| AppError::msg("Paper: sin nombre de jar"))?;
    let dl = format!(
        "https://api.papermc.io/v2/projects/paper/versions/{mc}/builds/{build_num}/downloads/{jar_name}"
    );
    net::download_all(
        client,
        vec![DownloadItem::new(dl, dest.to_path_buf())],
        1,
        app,
        "server-paper",
        &format!("Paper {mc}"),
    )
    .await
}

pub async fn download_fabric_server(
    client: &reqwest::Client,
    app: &AppHandle,
    mc: &str,
    dest: &Path,
) -> AppResult<()> {
    write_eula(dest.parent().unwrap_or(dest))?;
    let entries: Value = net::fetch_json(
        client,
        &format!("https://meta.fabricmc.net/v2/versions/loader/{mc}"),
    )
    .await?;
    let arr = entries.as_array().cloned().unwrap_or_default();
    let entry = arr
        .iter()
        .find(|e| e["loader"]["stable"].as_bool().unwrap_or(false))
        .or_else(|| arr.first())
        .ok_or_else(|| AppError::msg(format!("Fabric no soporta Minecraft {mc}")))?;
    let loader_ver = entry["loader"]["version"]
        .as_str()
        .ok_or_else(|| AppError::msg("Fabric: sin version de loader"))?;
    let inst_ver = entry["installer"]["version"]
        .as_str()
        .ok_or_else(|| AppError::msg("Fabric: sin version de installer"))?;
    let dl = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{mc}/{loader_ver}/{inst_ver}/server/jar"
    );
    net::download_all(
        client,
        vec![DownloadItem::new(dl, dest.to_path_buf())],
        1,
        app,
        "server-fabric",
        &format!("Fabric {mc}"),
    )
    .await
}

pub async fn download_forge_server(
    client: &reqwest::Client,
    app: &AppHandle,
    mc: &str,
    dir: &Path,
) -> AppResult<PathBuf> {
    write_eula(dir)?;
    let vers = forge::versions(client, mc).await?;
    let fv = vers
        .first()
        .ok_or_else(|| AppError::msg(format!("Forge no disponible para {mc}")))?;
    let full = forge::resolve_maven_id(mc, fv);
    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{full}/forge-{full}-installer.jar"
    );
    let tmp = dir.join(format!("forge-{full}-installer.jar"));
    net::download_all(
        client,
        vec![DownloadItem::new(installer_url, tmp.clone())],
        1,
        app,
        "server-forge",
        &format!("Forge {full}"),
    )
    .await?;

    let state = app.state::<crate::state::AppState>();
    let java = loaders::installer_java(app, &state, mc).await?;
    let dir_owned = dir.to_path_buf();
    let tmp_for_cmd = tmp.clone();
    let out = tauri::async_runtime::spawn_blocking(move || {
        let mut cmd = Command::new(&java);
        cmd.current_dir(&dir_owned)
            .arg("-jar")
            .arg(&tmp_for_cmd)
            .arg("--installServer");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        cmd.output()
    })
    .await
    .map_err(|e| AppError::msg(format!("Forge installServer: {e}")))??;

    if !out.status.success() {
        return Err(AppError::msg(format!(
            "Forge installServer fallo: {}",
            String::from_utf8_lossy(&out.stderr).chars().take(400).collect::<String>()
        )));
    }
    let _ = std::fs::remove_file(&tmp);
    find_forge_jar(dir).ok_or_else(|| AppError::msg("Forge server.jar no encontrado"))
}

fn find_forge_jar(dir: &Path) -> Option<PathBuf> {
    let universal = dir.join("forge.jar");
    if universal.is_file() {
        return Some(universal);
    }
    std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let n = e.file_name().to_string_lossy().to_lowercase();
        if n.ends_with("-universal.jar") || n.contains("forge") && n.ends_with(".jar") {
            Some(e.path())
        } else {
            None
        }
    })
}
