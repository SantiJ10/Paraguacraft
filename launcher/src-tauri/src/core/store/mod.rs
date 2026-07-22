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
pub mod server_modpack;

use std::path::PathBuf;

use tauri::AppHandle;

use crate::core::instances;
use crate::core::loaders;
use crate::core::servers;
use crate::error::{AppError, AppResult};
use crate::models::{StoreDependency, StoreSearchResult, StoreVersion};

/// Ejecuta trabajo bloqueante (ZIP, hashing, I/O de disco grande) en el pool de
/// `tokio::spawn_blocking` en vez del runtime async, para que la UI de Tauri
/// nunca vea el hilo principal ocupado ("No responde") durante instalaciones grandes.
pub async fn run_blocking<F, T>(f: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::msg(format!("Tarea interna interrumpida: {e}")))?
}

/// Devuelve true si ya hay un jar de este nombre (o variante `.disabled`/parcial por
/// coincidencia de "stem") en `dest_dir`. Evita re-descargar dependencias ya instaladas.
pub fn jar_already_present(dest_dir: &std::path::Path, filename: &str) -> bool {
    let target = dest_dir.join(filename);
    if target.is_file() {
        return true;
    }
    let stem = filename
        .trim_end_matches(".jar")
        .trim_end_matches(".disabled")
        .to_lowercase();
    dest_dir.read_dir().into_iter().flatten().flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_lowercase();
        (n.ends_with(".jar") || n.ends_with(".jar.disabled")) && n.contains(&stem)
    })
}

/// True si `mods/` ya tiene un JAR de la familia (p.ej. `sodium`), ignorando sub-modulos
/// como `sodium-extra` o `reeses-sodium-options`.
pub fn mod_family_present(dest_dir: &std::path::Path, family: &str, exclude_substrings: &[&str]) -> bool {
    let family = family.to_lowercase();
    dest_dir.read_dir().into_iter().flatten().flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_lowercase();
        if !n.ends_with(".jar") && !n.ends_with(".jar.disabled") {
            return false;
        }
        if exclude_substrings.iter().any(|x| n.contains(x)) {
            return false;
        }
        n.contains(&family)
    })
}

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
/// `offset`/`limit` habilitan paginación real (no solo los primeros resultados).
pub async fn search(
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    query: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
    offset: u32,
    limit: u32,
) -> AppResult<StoreSearchResult> {
    let loader = loaders::store_loader(loader);
    match provider {
        "modrinth" => modrinth::search(client, query, project_type, mc, &loader, offset, limit).await,
        "curseforge" => {
            curseforge::search(client, cf_key, query, project_type, mc, &loader, offset, limit).await
        }
        other => Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    }
}

/// Dependencias requeridas/embebidas de una version concreta, para el modal de
/// "descarga inteligente de dependencias" antes de instalar (Fase Tienda).
/// `dest_dir` (si se conoce el destino) marca `already_installed` por dependencia.
pub async fn list_required_dependencies(
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    project_id: &str,
    file_id_or_version_id: &str,
    project_type: &str,
    mc: &str,
    loader: &str,
    dest_dir: Option<&std::path::Path>,
) -> AppResult<Vec<StoreDependency>> {
    let loader = loaders::store_loader(loader);
    match provider {
        "modrinth" => {
            modrinth::list_required_dependencies(client, file_id_or_version_id, mc, &loader, dest_dir)
                .await
        }
        "curseforge" => {
            curseforge::list_required_dependencies(
                client,
                cf_key,
                project_id,
                file_id_or_version_id,
                project_type,
                mc,
                &loader,
                dest_dir,
            )
            .await
        }
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
        let inst = instances::scan::find_external(instance_id).ok_or_else(|| {
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

fn validate_server_mod(server_id: &str, mc: &str, loader: &str) -> AppResult<PathBuf> {
    let prof = servers::profile_by_id(server_id)?;
    if !prof.mc_version.is_empty()
        && prof.mc_version != "?"
        && prof.mc_version != mc
    {
        return Err(AppError::msg(format!(
            "El servidor \"{}\" usa Minecraft {}, no {mc}.",
            prof.name, prof.mc_version
        )));
    }
    let st = prof.server_type.as_str();
    if st.starts_with("fabric") {
        if !loaders::loaders_compatible("fabric", loader) {
            return Err(AppError::msg(format!(
                "El servidor \"{}\" es Fabric; el mod requiere {loader}.",
                prof.name
            )));
        }
    } else if st.starts_with("forge") {
        if !loaders::loaders_compatible("forge", loader) {
            return Err(AppError::msg(format!(
                "El servidor \"{}\" es Forge; el mod requiere {loader}.",
                prof.name
            )));
        }
    } else {
        return Err(AppError::msg(format!(
            "El servidor \"{}\" no acepta mods de cliente (solo Fabric/Forge). Usá plugins en servidores Paper.",
            prof.name
        )));
    }
    destinations::mod_dest_dir(server_id)
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

/// Resuelve la carpeta destino de una instalacion (instancia/servidor/mundo). Publica
/// para que los comandos puedan usarla al listar dependencias sin duplicar la instancia.
pub fn resolve_dest_dir(
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
        _ if dest.kind == "server" => {
            let sid = dest
                .server_id
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::msg("Seleccioná un servidor local destino."))?;
            if project_type == "mod" {
                validate_server_mod(sid, mc, loader)
            } else {
                return Err(AppError::msg(
                    "Solo los mods pueden instalarse en un servidor local.",
                ));
            }
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
    download_url: Option<String>,
    filename: Option<String>,
    sha1: Option<String>,
) -> AppResult<String> {
    if project_type == "modpack" {
        return Err(AppError::msg(
            "Los modpacks se instalan con import_mrpack_version (crean una instancia nueva).",
        ));
    }
    let loader_required = project_type == "mod";
    let dest_dir = resolve_dest_dir(project_type, &dest, mc, loader, loader_required)?;

    match provider {
        "modrinth" => {
            let hint = match (filename, download_url) {
                (Some(f), Some(u)) => Some((f, u, sha1)),
                _ => None,
            };
            modrinth::install_version_id(app, client, version_id, dest_dir, hint).await
        }
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
        let inst = instances::scan::find_external(instance_id)
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
