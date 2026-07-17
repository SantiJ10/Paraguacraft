//! Comandos de loaders (compatibilidad estricta via APIs oficiales).

use tauri::{AppHandle, State};

use crate::core::loaders;
use crate::error::AppResult;
use crate::models::LoaderInfo;
use crate::state::AppState;

/// Loaders disponibles para una version de MC (vanilla siempre; el resto solo
/// si la API oficial confirma versiones para esa MC).
#[tauri::command]
pub async fn list_loaders(state: State<'_, AppState>, mc: String) -> AppResult<Vec<LoaderInfo>> {
    let (http, _net) = state.net_scope();
    loaders::available_loaders(&http, &mc).await
}

/// Versiones EXACTAS de un loader para una MC.
#[tauri::command]
pub async fn list_loader_versions(
    state: State<'_, AppState>,
    loader: String,
    mc: String,
) -> AppResult<Vec<String>> {
    let (http, _net) = state.net_scope();
    loaders::loader_versions(&http, &loader, &mc).await
}

#[tauri::command]
pub async fn install_loader(
    app: AppHandle,
    state: State<'_, AppState>,
    mc: String,
    loader: String,
    loader_version: String,
) -> AppResult<String> {
    let (http, _net) = state.net_scope();
    loaders::install_loader(&app, &http, &mc, &loader, &loader_version).await
}

/// Instala el bundle de mods Fabric + Iris en una instancia existente.
#[tauri::command]
pub async fn install_fabric_iris_bundle(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> AppResult<()> {
    let meta = crate::core::instances::ensure_meta(&instance_id)?;
    if loaders::normalize(&meta.loader) != "fabric-iris" {
        return Err(crate::error::AppError::msg("La instancia no usa Fabric + Iris"));
    }
    let (http, _net) = state.net_scope();
    let dir = crate::core::instances::instance_dir(&instance_id);
    loaders::fabric_iris::install_bundle(&app, &http, &meta.mc_version, &dir).await
}

/// Instala el bundle PvP (ParaguacraftPvP + OptiFine) en una instancia 1.8.9.
#[tauri::command]
pub async fn install_pvp_bundle(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> AppResult<()> {
    let meta = crate::core::instances::ensure_meta(&instance_id)?;
    if loaders::normalize(&meta.loader) != "paraguacraft-pvp" {
        return Err(crate::error::AppError::msg("La instancia no usa Paraguacraft PvP"));
    }
    if meta.mc_version != loaders::pvp::MC {
        return Err(crate::error::AppError::msg(
            "Paraguacraft PvP solo esta disponible para Minecraft 1.8.9",
        ));
    }
    if crate::core::game_session::is_running() {
        crate::core::game_session::queue_pvp_sync(instance_id);
        return Ok(());
    }
    let (http, _net) = state.net_scope();
    let dir = crate::core::instances::instance_dir(&instance_id);
    loaders::pvp::install_bundle(&app, &http, &dir).await
}

/// Estado del cliente PvP publicado (manifest) vs instalado en una instancia.
#[tauri::command]
pub async fn get_pvp_client_status(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: Option<String>,
) -> AppResult<crate::models::PvpClientStatus> {
    let (http, _net) = state.net_scope();
    let dir = if let Some(id) = instance_id {
        Some(crate::core::instances::instance_dir(&id))
    } else {
        crate::core::instances::list_local()
            .into_iter()
            .find(|i| loaders::normalize(&i.loader) == "paraguacraft-pvp")
            .map(|i| crate::core::instances::instance_dir(&i.id))
    };
    Ok(loaders::pvp::client_status(&app, &http, dir.as_deref()).await)
}
