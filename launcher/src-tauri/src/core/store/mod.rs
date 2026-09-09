//! Tienda integrada (Modrinth + CurseForge).
//!
//! **Exploración (catálogo global):** `search` con `mc`/`loader` vacíos devuelve resultados
//! sin filtrar por instancia — estilo Modrinth browse.
//! **Instalación:** sigue filtrando por mc+loader de la instancia destino (Regla 1).

pub mod autoupdate;
pub mod cfpack;
pub mod curseforge;
pub mod deps;
pub mod destinations;
pub mod mod_index;
pub mod modrinth;
pub mod mrpack;
pub mod overrides;
pub mod server_modpack;

use std::path::PathBuf;

use tauri::AppHandle;

use crate::core::instances;
use crate::core::loaders;
use crate::core::servers;
use crate::error::{AppError, AppResult};
use crate::models::{StoreDependency, StoreProjectDetail, StoreSearchResult, StoreVersion};

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

/// Id canónico de un jar: recorta sufijos de versión (`fabric-api-0.119.jar` → `fabric-api`).
pub fn jar_canonical_id(filename: &str) -> String {
    let mut s = filename.to_lowercase();
    for suf in [".jar.disabled", ".jar", ".zip", ".disabled"] {
        if let Some(stripped) = s.strip_suffix(suf) {
            s = stripped.to_string();
            break;
        }
    }
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if (bytes[i] == b'-' || bytes[i] == b'_')
            && bytes.get(i + 1).is_some_and(|c| c.is_ascii_digit())
        {
            return s[..i].to_string();
        }
    }
    s
}

/// Devuelve true si ya hay un jar de este proyecto (mismo id canónico), no un sub-mod
/// (`sodium` no coincide con `sodium-extra`).
pub fn jar_already_present(dest_dir: &std::path::Path, filename: &str) -> bool {
    let target = dest_dir.join(filename);
    if target.is_file() {
        return true;
    }
    let disabled = dest_dir.join(format!("{filename}.disabled"));
    if disabled.is_file() {
        return true;
    }
    let id = jar_canonical_id(filename);
    if id.is_empty() {
        return false;
    }
    dest_dir.read_dir().into_iter().flatten().flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_lowercase();
        (n.ends_with(".jar") || n.ends_with(".jar.disabled") || n.ends_with(".zip"))
            && jar_canonical_id(&n) == id
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
    let loader = loaders::store_loader_for(loader, mc);
    match provider {
        "modrinth" => modrinth::search(client, query, project_type, mc, &loader, offset, limit).await,
        "curseforge" => {
            curseforge::search(client, cf_key, query, project_type, mc, &loader, offset, limit).await
        }
        other => Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    }
}

/// Ficha de proyecto (galería, descripción, autores, RAM recomendada).
pub async fn project_detail(
    client: &reqwest::Client,
    provider: &str,
    cf_key: &str,
    project_id: &str,
    project_type: &str,
) -> AppResult<StoreProjectDetail> {
    let mut detail = match provider {
        "modrinth" => modrinth::project_detail(client, project_id).await?,
        "curseforge" => curseforge::project_detail(client, cf_key, project_id, project_type).await?,
        other => return Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    };
    if detail.recommended_ram_gb.is_none() {
        detail.recommended_ram_gb = recommended_ram_gb(
            &detail.item.project_type,
            &detail.body,
            &detail.item.description,
        );
    }
    Ok(detail)
}

/// Extrae GB de RAM mencionados junto a "ram" en un texto de tienda.
pub fn parse_recommended_ram_gb(text: &str) -> Option<u32> {
    let t = text.to_ascii_lowercase().replace('\n', " ");
    let chars: Vec<char> = t.chars().collect();
    let mut best: Option<u32> = None;
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            let mut n: u32 = 0;
            while i < chars.len() && chars[i].is_ascii_digit() {
                n = n.saturating_mul(10).saturating_add(chars[i].to_digit(10).unwrap_or(0));
                i += 1;
            }
            let mut j = i;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            let unit: String = chars.get(j..).unwrap_or(&[]).iter().take(4).collect();
            if (unit.starts_with("gb") || unit.starts_with("gi")) && (2..=128).contains(&n) {
                let ctx_from = start.saturating_sub(28);
                let ctx_to = (j + 12).min(chars.len());
                let ctx: String = chars[ctx_from..ctx_to].iter().collect();
                if ctx.contains("ram") || ctx.contains("memory") || ctx.contains("memoria") {
                    best = Some(best.map_or(n, |b| b.max(n)));
                }
            }
            continue;
        }
        i += 1;
    }
    best
}

fn recommended_ram_gb(project_type: &str, body: &str, summary: &str) -> Option<u32> {
    let blob = format!("{summary}\n{body}");
    if let Some(n) = parse_recommended_ram_gb(&blob) {
        return Some(n);
    }
    if project_type == "modpack" {
        Some(8)
    } else {
        None
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
    let loader = loaders::store_loader_for(loader, mc);
    deps::resolve_required(
        client,
        provider,
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
    let loader = loaders::store_loader_for(loader, mc);
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
        if loader_required && !loaders::loaders_compatible_for(&inst.loader, loader, mc) {
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
    if loader_required && !loaders::loaders_compatible_for(&meta.loader, loader, mc) {
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
    } else if st.starts_with("neoforge") {
        if !loaders::loaders_compatible("neoforge", loader) {
            return Err(AppError::msg(format!(
                "El servidor \"{}\" es NeoForge; el mod requiere {loader}.",
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
            "El servidor \"{}\" no acepta mods de cliente (solo Fabric/Forge/NeoForge). Usá plugins en servidores Paper.",
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
    let dest_for_index = dest_dir.clone();
    let store_loader = loaders::store_loader_for(loader, mc);

    let filename = match provider {
        "modrinth" => {
            let hint = match (filename, download_url) {
                (Some(f), Some(u)) => Some((f, u, sha1.clone())),
                _ => None,
            };
            modrinth::install_version_id(app, client, version_id, dest_dir, hint).await?
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
            .await?
        }
        other => return Err(AppError::msg(format!("Proveedor desconocido: {other}"))),
    };
    if project_type == "mod" {
        let dep_ids = deps::resolve_required(
            client,
            provider,
            cf_key,
            project_id,
            version_id,
            project_type,
            mc,
            &store_loader,
            Some(dest_for_index.as_path()),
        )
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|d| d.project_id)
        .collect::<Vec<_>>();
        let _ = mod_index::record(
            &dest_for_index,
            provider,
            project_id,
            version_id,
            &filename,
            sha1,
            Some(mc),
            &[store_loader],
            &dep_ids,
        );
    }
    Ok(filename)
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
        (
            inst.mc_version.clone(),
            loaders::store_loader_for(&inst.loader, &inst.mc_version),
            base,
        )
    } else {
        let meta = instances::ensure_meta(instance_id)?;
        (
            meta.mc_version.clone(),
            loaders::store_loader_for(&meta.loader, &meta.mc_version),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ram_from_modpack_blurb() {
        assert_eq!(
            parse_recommended_ram_gb("Recommended RAM: 16 GB. Minimum 8GB RAM."),
            Some(16)
        );
        assert_eq!(
            parse_recommended_ram_gb("Allocate at least 8GB RAM for this pack."),
            Some(8)
        );
        assert_eq!(parse_recommended_ram_gb("Sodium is a rendering engine."), None);
        assert_eq!(
            parse_recommended_ram_gb("Memoria recomendada: 12 gb ram"),
            Some(12)
        );
    }

    #[test]
    fn modpack_defaults_to_8gb() {
        assert_eq!(recommended_ram_gb("modpack", "", "un pack"), Some(8));
        assert_eq!(recommended_ram_gb("mod", "", "sodium"), None);
    }

    #[test]
    fn jar_canonical_does_not_confuse_sodium_extra() {
        assert_eq!(jar_canonical_id("sodium-0.6.13+mc1.21.1.jar"), "sodium");
        assert_eq!(
            jar_canonical_id("sodium-extra-0.6.0+mc1.21.1.jar"),
            "sodium-extra"
        );
        assert_eq!(jar_canonical_id("fabric-api-0.119.2+1.21.1.jar"), "fabric-api");
    }
}

