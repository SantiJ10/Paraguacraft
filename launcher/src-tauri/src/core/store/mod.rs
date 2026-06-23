//! Tienda integrada (Modrinth + CurseForge).
//!
//! **Exploración (catálogo global):** `search` con `mc`/`loader` vacíos devuelve resultados
//! sin filtrar por instancia — estilo Modrinth browse.
//! **Instalación:** sigue filtrando por mc+loader de la instancia destino (Regla 1).

pub mod autoupdate;
pub mod cfpack;
pub mod curseforge;
pub mod destinations;
pub mod modrinth;
pub mod mrpack;

use std::path::PathBuf;

use tauri::AppHandle;

use crate::core::instances;
use crate::core::loaders;
use crate::error::{AppError, AppResult};
use crate::models::{StoreItem, StoreVersion};

/// Subcarpeta de la instancia segun el tipo de contenido.
pub fn content_subdir(project_type: &str) -> &'static str {
    match project_type {
        "resourcepack" => "resourcepacks",
        "shader" => "shaderpacks",
        "datapack" => "datapacks",
        "plugin" => "plugins",
        _ => "mods",
    }
}

/// Busca en el proveedor indicado. Con `mc`/`loader` vacíos explora el catálogo global.
pub async fn search(
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    query: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
) -> AppResult<Vec<StoreItem>> {
    let loader = loaders::store_loader(loader);
    match provider {
        "modrinth" => modrinth::search(client, query, project_type, mc, &loader).await,
        "curseforge" => curseforge::search(client, cf_key, query, project_type, mc, &loader).await,
        other => Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    }
}

/// Lista versiones/archivos del proyecto compatibles con mc + loader.
pub async fn list_versions(
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    project_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
) -> AppResult<Vec<StoreVersion>> {
    let loader = loaders::store_loader(loader);
    match provider {
        "modrinth" => {
            modrinth::list_versions(client, project_id, project_type, mc, &loader).await
        }
        "curseforge" => {
            curseforge::list_versions(client, cf_key, project_id, project_type, mc, &loader).await
        }
        other => Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    }
}

/// Todas las versiones del proyecto (asistente modpack: mc + loaders reales del proyecto).
pub async fn list_project_versions(
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    project_id: &str,
    project_type: &str,
) -> AppResult<Vec<StoreVersion>> {
    match provider {
        "modrinth" => modrinth::list_all_versions(client, project_id, project_type).await,
        "curseforge" => {
            curseforge::list_all_versions(client, cf_key, project_id, project_type).await
        }
        other => Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    }
}

fn validate_instance(
    instance_id: &str,
    mc: &str,
    loader: &str,
    loader_required: bool,
) -> AppResult<PathBuf> {
    if instance_id.starts_with("ext::") {
        let inst = instances::scan::scan_external()
            .into_iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| {
                AppError::msg(
                    "Instancia externa no encontrada. Usa «Escanear» en Instancias y reintenta.",
                )
            })?;
        if inst.mc_version != mc {
            return Err(AppError::msg(format!(
                "La instancia \"{}\" usa Minecraft {}, no {mc}.",
                inst.name, inst.mc_version
            )));
        }
        if loader_required && !loaders::loaders_compatible(&inst.loader, loader) {
            return Err(AppError::msg(format!(
                "La instancia \"{}\" usa {}, incompatible con {loader}.",
                inst.name, inst.loader
            )));
        }
        return instances::importers::external_game_dir(instance_id).ok_or_else(|| {
            AppError::msg("No se pudo resolver la carpeta de juego de esta instancia externa.")
        });
    }

    let meta = instances::ensure_meta(instance_id)?;
    if meta.mc_version != mc {
        return Err(AppError::msg(format!(
            "La instancia \"{}\" usa Minecraft {}, no {mc}.",
            meta.name, meta.mc_version
        )));
    }
    if loader_required && !loaders::loaders_compatible(&meta.loader, loader) {
        return Err(AppError::msg(format!(
            "La instancia \"{}\" usa {}, incompatible con {loader}.",
            meta.name, meta.loader
        )));
    }
    Ok(instances::instance_dir(instance_id))
}

/// Destino de instalación desde la tienda.
#[derive(Debug, Clone, Default)]
pub struct InstallDestination {
    /// `instance` (default) | `server`
    pub kind: String,
    pub instance_id: Option<String>,
    pub server_id: Option<String>,
    pub world_name: Option<String>,
}

fn resolve_dest_dir(
    project_type: &str,
    dest: &InstallDestination,
    mc: &str,
    loader: &str,
    loader_required: bool,
) -> AppResult<PathBuf> {
    match project_type {
        "plugin" => {
            let sid = dest
                .server_id
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::msg("Seleccioná un servidor local para instalar el plugin."))?;
            destinations::plugin_dest_dir(sid)
        }
        "datapack" if dest.kind == "server" => {
            let sid = dest
                .server_id
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::msg("Seleccioná un servidor local para el datapack."))?;
            destinations::datapack_dest_server(
                sid,
                dest.world_name.as_deref(),
            )
        }
        "datapack" => {
            let iid = dest
                .instance_id
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::msg("Seleccioná una instancia para el datapack."))?;
            let _ = validate_instance(iid, mc, loader, false)?;
            destinations::datapack_dest_instance(iid, dest.world_name.as_deref())
        }
        _ => {
            let iid = dest
                .instance_id
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::msg("Seleccioná una instancia destino."))?;
            let base = validate_instance(iid, mc, loader, loader_required)?;
            Ok(base.join(content_subdir(project_type)))
        }
    }
}

/// Instala una version concreta (instancia, servidor local o mundo).
pub async fn install_version(
    app: &AppHandle,
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    project_id: &str,
    project_type: &str,
    version_id: &str,
    mc: &str,
    loader: &str,
    dest: InstallDestination,
) -> AppResult<String> {
    if project_type == "modpack" {
        return Err(AppError::msg(
            "Los modpacks se instalan con import_mrpack_version (crean una instancia nueva).",
        ));
    }
    let loader_required = modrinth::loader_relevant(project_type)
        && project_type != "plugin"
        && dest.kind != "server";
    let dest_dir = resolve_dest_dir(project_type, &dest, mc, loader, loader_required)?;

    match provider {
        "modrinth" => modrinth::install_version_id(app, client, version_id, dest_dir).await,
        "curseforge" => {
            curseforge::install_file_id(
                app,
                client,
                cf_key,
                project_id,
                version_id,
                dest_dir,
            )
            .await
        }
        other => Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    }
}

/// Instala un proyecto en la instancia, usando su metadata (mc + loader) para
/// filtrar (Regla 1) y colocarlo en la subcarpeta correcta.
pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    project_id: &str,
    project_type: &str,
    instance_id: &str,
) -> AppResult<String> {
    let (mc, loader, base) = if instance_id.starts_with("ext::") {
        let inst = instances::scan::scan_external()
            .into_iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| AppError::msg("Instancia externa no encontrada"))?;
        let base = instances::importers::external_game_dir(instance_id)
            .ok_or_else(|| AppError::msg("Sin carpeta de juego para instancia externa"))?;
        (inst.mc_version.clone(), loaders::store_loader(&inst.loader), base)
    } else {
        let meta = instances::ensure_meta(instance_id)?;
        (
            meta.mc_version.clone(),
            loaders::store_loader(&meta.loader),
            instances::instance_dir(instance_id),
        )
    };

    let dest_dir: PathBuf = base.join(content_subdir(project_type));

    match provider {
        "modrinth" => {
            modrinth::install(app, client, project_id, project_type, &mc, &loader, dest_dir).await
        }
        "curseforge" => {
            curseforge::install(app, client, cf_key, project_id, project_type, &mc, &loader, dest_dir)
                .await
        }
        other => Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    }
}
