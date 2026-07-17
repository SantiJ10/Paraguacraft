//! Chequeo pre-lanzamiento on-demand (sin timers en background).

use std::path::Path;

use sysinfo::Disks;
use tauri::AppHandle;

use crate::core::accounts;
use crate::core::instances;
use crate::core::java;
use crate::core::loaders;
use crate::core::paths;
use crate::core::versions;
use crate::error::AppResult;
use crate::models::AppSettings;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreLaunchCheckItem {
    pub id: String,
    pub label: String,
    /// "ok" | "warn" | "error" | "info"
    pub status: String,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreLaunchCheckReport {
    pub ready: bool,
    pub items: Vec<PreLaunchCheckItem>,
}

pub async fn run(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_id: &str,
) -> AppResult<PreLaunchCheckReport> {
    let meta = instances::ensure_meta(instance_id)?;
    let game_dir = instances::instance_dir(instance_id);
    let settings: AppSettings = crate::config::read_json(&paths::config_file()).unwrap_or_default();
    let mut items = Vec::new();
    let mut blocking = false;

    // Cuenta activa
    if accounts::active_account().is_some() {
        items.push(ok(
            "account",
            "Cuenta",
            "Hay una cuenta activa para iniciar sesión.",
            None,
        ));
    } else {
        blocking = true;
        items.push(err(
            "account",
            "Cuenta",
            "No hay cuenta activa.",
            Some("Agregá una cuenta en Ajustes → Cuentas."),
        ));
    }

    // Perfil de versión / loader
    if let Some(ref vid) = meta.version_id {
        if versions::read_local_json(vid).is_some() {
            items.push(ok(
                "profile",
                "Perfil de juego",
                format!("Perfil instalado: {vid}"),
                None,
            ));
        } else {
            blocking = true;
            items.push(err(
                "profile",
                "Perfil de juego",
                "El perfil de la instancia no está instalado.",
                Some("Usá Reparar instancia o reinstalá el loader."),
            ));
        }
    } else if loaders::normalize(&meta.loader) != "vanilla" {
        items.push(warn(
            "profile",
            "Perfil de juego",
            "Sin version_id guardado; se resolverá al lanzar.",
            None,
        ));
    } else {
        items.push(ok(
            "profile",
            "Perfil de juego",
            format!("Vanilla {} listo para resolver.", meta.mc_version),
            None,
        ));
    }

    // Java
    let required = java::required_for_mc(&meta.mc_version);
    let java_path = meta
        .java_path
        .as_deref()
        .or(settings.java_path.as_deref());
    match java_path {
        Some(path) if !path.trim().is_empty() => {
            if let Some(info) = java::verify::verify(Path::new(path), "custom") {
                if java::is_compatible(info.version_major, &meta.mc_version) {
                    items.push(ok(
                        "java",
                        "Java",
                        format!(
                            "Java {} ({}) compatible con Minecraft {}.",
                            info.version_major, path, meta.mc_version
                        ),
                        None,
                    ));
                } else {
                    blocking = true;
                    items.push(err(
                        "java",
                        "Java",
                        format!(
                            "Java {} no sirve para Minecraft {} (se necesita major {}).",
                            info.version_major, meta.mc_version, required
                        ),
                        Some("Elegí otra Java en la configuración de la instancia."),
                    ));
                }
            } else {
                blocking = true;
                items.push(err(
                    "java",
                    "Java",
                    format!("No se pudo verificar Java en {path}"),
                    Some("Reinstalá Temurin desde Ajustes → Java."),
                ));
            }
        }
        _ => items.push(warn(
            "java",
            "Java",
            format!(
                "Se usará Java automática (Minecraft {} requiere Java {}).",
                meta.mc_version, required
            ),
            Some("Si falla el lanzamiento, fijá Java manualmente en la instancia."),
        )),
    }

    // Cliente PvP
    if loaders::normalize(&meta.loader) == "paraguacraft-pvp" {
        let status = loaders::pvp::client_status(app, client, Some(&game_dir)).await;
        if status.up_to_date {
            items.push(ok(
                "pvp",
                "Cliente PvP",
                format!(
                    "Cliente al día (v{}).",
                    status.installed_version.unwrap_or(status.remote_version)
                ),
                None,
            ));
        } else if status.installed_filename.is_some() {
            items.push(warn(
                "pvp",
                "Cliente PvP",
                format!(
                    "Hay update disponible (instalado v{} → remoto v{}).",
                    status
                        .installed_version
                        .unwrap_or_else(|| "?".into()),
                    status.remote_version
                ),
                Some("En Modo Competir se difiere la descarga hasta cerrar Minecraft."),
            ));
        } else {
            blocking = true;
            items.push(err(
                "pvp",
                "Cliente PvP",
                "No se encontró el mod ParaguacraftPvP.",
                Some("Repará la instancia o sincronizá el cliente PvP."),
            ));
        }
    }

    // Espacio en disco
    match disk_free_mb(&game_dir) {
        Some(mb) if mb < 500 => {
            blocking = true;
            items.push(err(
                "disk",
                "Espacio en disco",
                format!("Quedan solo {mb} MB libres en el disco de la instancia."),
                Some("Liberá espacio antes de jugar o mover la carpeta .minecraft."),
            ));
        }
        Some(mb) if mb < 2048 => {
            items.push(warn(
                "disk",
                "Espacio en disco",
                format!("Poco espacio libre ({mb} MB). Minecraft puede necesitar más al guardar mundos."),
                None,
            ));
        }
        Some(mb) => items.push(ok(
            "disk",
            "Espacio en disco",
            format!("{mb} MB libres en el disco de la instancia."),
            None,
        )),
        None => items.push(warn(
            "disk",
            "Espacio en disco",
            "No se pudo medir el espacio libre.",
            None,
        )),
    }

    // Tip antivirus (solo informativo)
    #[cfg(windows)]
    items.push(info(
        "av",
        "Antivirus",
        "Si el juego no arranca o borra mods, agregá excepción para la carpeta .minecraft y Paraguacraft.",
        Some("Windows Security → Protección contra virus → Exclusiones."),
    ));
    #[cfg(not(windows))]
    items.push(info(
        "av",
        "Permisos",
        "Asegurate de que la carpeta de la instancia no esté bloqueada por el sistema.",
        None,
    ));

    // Conflictos de mods (heurístico)
    match crate::core::mod_conflicts::scan(instance_id) {
        Ok(conflicts) => {
            for c in conflicts {
                let is_error = c.severity == "error";
                if is_error {
                    blocking = true;
                }
                items.push(PreLaunchCheckItem {
                    id: format!("modconflict-{}", c.title.to_lowercase().replace(' ', "-")),
                    label: c.title.clone(),
                    status: if is_error { "error".into() } else { "warn".into() },
                    message: c.detail,
                    hint: if c.mods.is_empty() {
                        None
                    } else {
                        Some(format!("Mods: {}", c.mods.join(", ")))
                    },
                });
            }
        }
        Err(e) => items.push(warn(
            "modconflicts",
            "Mods",
            format!("No se pudo analizar conflictos: {e}"),
            None,
        )),
    }

    // Compatibilidad Modrinth (loader / versión MC)
    let mut content_items = instances::content::list(instance_id)?;
    if let Ok(()) = instances::content_metadata::enrich(client, instance_id, &game_dir, &mut content_items).await {
        for item in &content_items {
            if item.folder == "mods" && item.enabled && item.compatible == Some(false) {
                items.push(warn(
                    "modcompat",
                    "Compatibilidad",
                    item.compat_message.clone().unwrap_or_else(|| {
                        format!("{} puede no ser compatible con esta instancia.", item.display_name.as_deref().unwrap_or(&item.name))
                    }),
                    Some("Revisá la tienda Modrinth o actualizá el mod."),
                ));
            }
        }
    }

    let ready = !blocking;
    Ok(PreLaunchCheckReport { ready, items })
}

fn disk_free_mb(path: &Path) -> Option<u64> {
    let canonical = path.canonicalize().ok()?;
    let disks = Disks::new_with_refreshed_list();
    let mut best: Option<u64> = None;
    let mut best_len = 0usize;
    for disk in disks.list() {
        let mount = disk.mount_point();
        if canonical.starts_with(mount) {
            let len = mount.as_os_str().len();
            if len >= best_len {
                best_len = len;
                best = Some(disk.available_space() / 1024 / 1024);
            }
        }
    }
    best
}

fn ok(id: &str, label: &str, message: impl Into<String>, hint: Option<&str>) -> PreLaunchCheckItem {
    item(id, label, "ok", message, hint)
}

fn warn(id: &str, label: &str, message: impl Into<String>, hint: Option<&str>) -> PreLaunchCheckItem {
    item(id, label, "warn", message, hint)
}

fn err(id: &str, label: &str, message: impl Into<String>, hint: Option<&str>) -> PreLaunchCheckItem {
    item(id, label, "error", message, hint)
}

fn info(id: &str, label: &str, message: impl Into<String>, hint: Option<&str>) -> PreLaunchCheckItem {
    item(id, label, "info", message, hint)
}

fn item(
    id: &str,
    label: &str,
    status: &str,
    message: impl Into<String>,
    hint: Option<&str>,
) -> PreLaunchCheckItem {
    PreLaunchCheckItem {
        id: id.into(),
        label: label.into(),
        status: status.into(),
        message: message.into(),
        hint: hint.map(str::to_string),
    }
}
