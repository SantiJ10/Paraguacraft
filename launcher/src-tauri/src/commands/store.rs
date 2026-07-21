//! Comandos de la tienda (Modrinth + CurseForge) y import de modpacks.

use tauri::{AppHandle, State};

use crate::config::keys;
use crate::core::store::{self, destinations, InstallDestination};
use crate::error::AppResult;
use crate::core::servers::ServerProfile;
use crate::models::{Instance, ServerWorldsResult, StoreDependency, StoreSearchResult, StoreVersion, WorldInfo};
use crate::state::AppState;

#[tauri::command]
pub async fn store_search(
    state: State<'_, AppState>,
    provider: String,
    query: String,
    project_type: String,
    mc: String,
    loader: String,
    offset: Option<u32>,
    limit: Option<u32>,
) -> AppResult<StoreSearchResult> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::search(
        &http,
        &provider,
        &key,
        &query,
        &project_type,
        &mc,
        &loader,
        offset.unwrap_or(0),
        limit.unwrap_or(40),
    )
    .await
}

/// Dependencias requeridas de una version/archivo concreto, para el modal de
/// confirmacion antes de instalar. `dest_dir` se resuelve igual que en `store_install_version`
/// para marcar `alreadyInstalled` por dependencia cuando ya se conoce el destino.
#[tauri::command]
pub async fn store_list_dependencies(
    state: State<'_, AppState>,
    provider: String,
    project_id: String,
    project_type: String,
    version_id: String,
    mc: String,
    loader: String,
    instance_id: Option<String>,
    destination: Option<String>,
    server_id: Option<String>,
    world_name: Option<String>,
) -> AppResult<Vec<StoreDependency>> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    let dest_dir = if project_type == "modpack" {
        None
    } else {
        let dest = InstallDestination {
            kind: destination.unwrap_or_else(|| "instance".into()),
            instance_id,
            server_id,
            world_name,
        };
        store::resolve_dest_dir(&project_type, &dest, &mc, &loader, project_type == "mod").ok()
    };
    store::list_required_dependencies(
        &http,
        &provider,
        &key,
        &project_id,
        &version_id,
        &project_type,
        &mc,
        &loader,
        dest_dir.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn store_list_versions(
    state: State<'_, AppState>,
    provider: String,
    project_id: String,
    project_type: String,
    mc: String,
    loader: String,
) -> AppResult<Vec<StoreVersion>> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::list_versions(&http, &provider, &key, &project_id, &project_type, &mc, &loader).await
}

#[tauri::command]
pub async fn store_list_project_versions(
    state: State<'_, AppState>,
    provider: String,
    project_id: String,
    project_type: String,
) -> AppResult<Vec<StoreVersion>> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::list_project_versions(&http, &provider, &key, &project_id, &project_type).await
}

#[tauri::command]
pub async fn store_install(
    app: AppHandle,
    state: State<'_, AppState>,
    provider: String,
    project_id: String,
    project_type: String,
    instance_id: String,
) -> AppResult<String> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::install(&app, &http, &provider, &key, &project_id, &project_type, &instance_id).await
}

/// Instala una version concreta tras el asistente de la tienda.
#[tauri::command]
pub async fn store_install_version(
    app: AppHandle,
    state: State<'_, AppState>,
    provider: String,
    project_id: String,
    project_type: String,
    version_id: String,
    mc: String,
    loader: String,
    instance_id: Option<String>,
    destination: Option<String>,
    server_id: Option<String>,
    world_name: Option<String>,
    filename: Option<String>,
    download_url: Option<String>,
    sha1: Option<String>,
) -> AppResult<String> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    let dest = InstallDestination {
        kind: destination.unwrap_or_else(|| "instance".into()),
        instance_id,
        server_id,
        world_name,
    };
    store::install_version(
        &app,
        &http,
        &provider,
        &key,
        &project_id,
        &project_type,
        &version_id,
        &mc,
        &loader,
        dest,
        download_url,
        filename,
        sha1,
    )
    .await
}

#[tauri::command]
pub async fn import_mrpack(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
    mc: String,
) -> AppResult<Instance> {
    let (http, _net) = state.net_scope();
    store::mrpack::import_modrinth(&app, &http, &source, &mc).await
}

/// Instala un modpack desde la tienda usando el id de version (.mrpack → instancia nueva).
#[tauri::command]
pub async fn import_mrpack_version(
    app: AppHandle,
    state: State<'_, AppState>,
    version_id: String,
) -> AppResult<Instance> {
    let (http, _net) = state.net_scope();
    store::mrpack::import_by_version_id(&app, &http, &version_id).await
}

#[tauri::command]
pub async fn import_cfpack(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
    mc: String,
) -> AppResult<Instance> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::cfpack::import_from_project(&app, &http, &key, &source, &mc).await
}

#[tauri::command]
pub async fn import_cfpack_version(
    app: AppHandle,
    state: State<'_, AppState>,
    mod_id: String,
    file_id: String,
) -> AppResult<Instance> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::cfpack::import_by_file_id(&app, &http, &key, &mod_id, &file_id).await
}

/// Instala un modpack Modrinth (.mrpack) en un servidor local Fabric/Forge nuevo.
#[tauri::command]
pub async fn import_mrpack_version_to_server(
    app: AppHandle,
    state: State<'_, AppState>,
    version_id: String,
    ram_mb: Option<u32>,
) -> AppResult<ServerProfile> {
    let (http, _net) = state.net_scope();
    store::server_modpack::import_mrpack_version_to_server(
        &app,
        &state,
        &http,
        &version_id,
        ram_mb.unwrap_or(4096),
    )
    .await
}

/// Instala un modpack CurseForge (.zip) en un servidor local Fabric/Forge nuevo.
#[tauri::command]
pub async fn import_cfpack_version_to_server(
    app: AppHandle,
    state: State<'_, AppState>,
    mod_id: String,
    file_id: String,
    ram_mb: Option<u32>,
) -> AppResult<ServerProfile> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::server_modpack::import_cfpack_version_to_server(
        &app,
        &state,
        &http,
        &key,
        &mod_id,
        &file_id,
        ram_mb.unwrap_or(4096),
    )
    .await
}

#[tauri::command]
pub async fn pick_and_import_cfpack_zip(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Instance> {
    use tauri_plugin_dialog::DialogExt;

    let picked = app
        .dialog()
        .file()
        .add_filter("Modpack CurseForge", &["zip"])
        .blocking_pick_file();
    let Some(file) = picked else {
        return Err(crate::error::AppError::msg("No se seleccionó ningún archivo"));
    };
    let path = file
        .into_path()
        .map_err(|e| crate::error::AppError::msg(format!("Ruta inválida: {e}")))?;
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::cfpack::import_from_zip_path(&app, &http, &key, &path).await
}

#[tauri::command]
pub fn list_instance_worlds(instance_id: String) -> AppResult<Vec<WorldInfo>> {
    destinations::list_instance_worlds(&instance_id)
}

#[tauri::command]
pub fn list_server_worlds(server_id: String) -> AppResult<ServerWorldsResult> {
    let (worlds, default_world) = destinations::list_server_worlds(&server_id)?;
    Ok(ServerWorldsResult {
        worlds,
        default_world,
    })
}

/// Actualiza el contenido compatible de una instancia (mismo mc+loader).
#[tauri::command]
pub async fn update_instance_content(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> AppResult<u32> {
    let key = keys::curseforge_api_key();
    let (http, _net) = state.net_scope();
    store::autoupdate::update_instance(&app, &http, &key, &instance_id).await
}
