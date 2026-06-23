//! Comandos de versiones de Minecraft.

use tauri::{AppHandle, State};

use crate::core::versions;
use crate::error::AppResult;
use crate::models::MinecraftVersion;
use crate::state::AppState;

#[tauri::command]
pub async fn list_minecraft_versions(
    state: State<'_, AppState>,
) -> AppResult<Vec<MinecraftVersion>> {
    let (http, _net) = state.net_scope();
    versions::list_versions(&http).await
}

#[tauri::command]
pub async fn install_minecraft_version(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let (http, _net) = state.net_scope();
    versions::install_vanilla(&app, &http, &id).await
}
