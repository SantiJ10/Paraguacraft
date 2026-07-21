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

/// PaperMC apagó la API `api.papermc.io/v2` (Fill v2) el 1-jul-2026; hoy responde 410 Gone.
/// El reemplazo es Fill v3 en `fill.papermc.io`: la lista de builds ya viene como array
/// plano (no envuelto en `{"builds": [...]}`) y cada build trae la URL de descarga lista
/// en `downloads."server:default".url` (ya no hay que armar el path a mano).
const PAPER_FILL_API: &str = "https://fill.papermc.io/v3";

pub async fn download_paper_server(
    client: &reqwest::Client,
    app: &AppHandle,
    mc: &str,
    dest: &Path,
) -> AppResult<()> {
    write_eula(dest.parent().unwrap_or(dest))?;
    let url = format!("{PAPER_FILL_API}/projects/paper/versions/{mc}/builds");
    let builds: Value = net::fetch_json(client, &url).await.map_err(|_| {
        AppError::msg(format!(
            "Paper no tiene builds para Minecraft {mc}. Probá con Fabric, Forge o con otra versión."
        ))
    })?;
    let arr = builds.as_array().cloned().unwrap_or_default();
    // El listado viene del build más nuevo al más viejo. Preferimos el canal más estable
    // disponible, pero versiones muy nuevas (recién salidas) a veces todavía no tienen
    // ningún build STABLE — en ese caso caemos a BETA/ALPHA en vez de fallar.
    let build = ["STABLE", "BETA", "ALPHA"]
        .iter()
        .find_map(|ch| arr.iter().find(|b| b["channel"].as_str() == Some(*ch)))
        .or_else(|| arr.first())
        .ok_or_else(|| AppError::msg(format!("No hay builds de Paper para Minecraft {mc}.")))?;
    let dl = build["downloads"]["server:default"]["url"]
        .as_str()
        .ok_or_else(|| AppError::msg("Paper: el build no trae URL de descarga"))?
        .to_string();
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
        .ok_or_else(|| {
            AppError::msg(format!(
                "Fabric no tiene loader para Minecraft {mc}. Probá con Paper, Forge o NeoForge."
            ))
        })?;
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
    let fv = vers.first().ok_or_else(|| {
        AppError::msg(format!(
            "Forge no tiene builds para Minecraft {mc}. Probá con NeoForge, Fabric o Paper."
        ))
    })?;
    let full = forge::resolve_maven_id(mc, fv);
    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{full}/forge-{full}-installer.jar"
    );
    run_installer_server(app, client, mc, dir, &installer_url, &format!("Forge {full}")).await?;
    find_forge_style_jar(dir, "forge").ok_or_else(|| AppError::msg("Forge: server.jar no encontrado tras instalar"))
}

/// NeoForge (sucesor de Forge para 1.20.2+). Mismo mecanismo de instalador Maven que Forge,
/// pero maven/artefacto propios y sin el sufijo legado `-{mc}` al final del id de versión.
pub async fn download_neoforge_server(
    client: &reqwest::Client,
    app: &AppHandle,
    mc: &str,
    dir: &Path,
) -> AppResult<PathBuf> {
    write_eula(dir)?;
    let vers = loaders::neoforge::versions(client, mc).await?;
    let version = vers.first().ok_or_else(|| {
        AppError::msg(format!(
            "NeoForge no tiene builds para Minecraft {mc} (NeoForge solo soporta 1.20.2+). Probá con Forge, Fabric o Paper."
        ))
    })?;
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{version}/neoforge-{version}-installer.jar"
    );
    run_installer_server(app, client, mc, dir, &installer_url, &format!("NeoForge {version}")).await?;
    find_forge_style_jar(dir, "neoforge")
        .ok_or_else(|| AppError::msg("NeoForge: server.jar no encontrado tras instalar"))
}

/// Descarga el .jar del instalador (Forge/NeoForge comparten el mismo flujo
/// `--installServer`) y lo corre con el Java correcto para esa versión de MC.
async fn run_installer_server(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    dir: &Path,
    installer_url: &str,
    label: &str,
) -> AppResult<()> {
    let tmp = dir.join(
        installer_url
            .rsplit('/')
            .next()
            .unwrap_or("installer.jar")
            .to_string(),
    );
    net::download_all(
        client,
        vec![DownloadItem::new(installer_url.to_string(), tmp.clone())],
        1,
        app,
        "server-installer",
        label,
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
    .map_err(|e| AppError::msg(format!("{label} installServer: {e}")))??;

    if !out.status.success() {
        return Err(AppError::msg(format!(
            "{label} installServer falló: {}",
            String::from_utf8_lossy(&out.stderr).chars().take(400).collect::<String>()
        )));
    }
    let _ = std::fs::remove_file(&tmp);
    Ok(())
}

/// Forge/NeoForge <= 1.16.5 generan un jar "universal" ejecutable directo. Desde 1.17+ el
/// instalador ya no genera ese jar: deja `run.bat`/`run.sh` + `libraries/` y ahí vive el
/// classpath real (`servers.rs::start_mc` ya sabe ejecutar `run.bat` para estos casos). Por
/// eso, si no hay jar universal pero sí `run.bat`, lo tratamos como instalación exitosa.
fn find_forge_style_jar(dir: &Path, kind: &str) -> Option<PathBuf> {
    let universal = dir.join("forge.jar");
    if universal.is_file() {
        return Some(universal);
    }
    if let Some(found) = std::fs::read_dir(dir).ok().and_then(|rd| {
        rd.flatten().find_map(|e| {
            let n = e.file_name().to_string_lossy().to_lowercase();
            if n.ends_with("-universal.jar") || (n.contains(kind) && n.ends_with(".jar")) {
                Some(e.path())
            } else {
                None
            }
        })
    }) {
        return Some(found);
    }
    let run_bat = dir.join("run.bat");
    if run_bat.is_file() {
        return Some(run_bat);
    }
    let run_sh = dir.join("run.sh");
    if run_sh.is_file() {
        return Some(run_sh);
    }
    None
}
