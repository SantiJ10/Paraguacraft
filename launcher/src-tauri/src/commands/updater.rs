//! Comandos de auto-update del launcher (on-demand).

use tauri::{AppHandle, State};

use crate::core::updater::{self, UpdateInfo};
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn check_launcher_update(state: State<'_, AppState>) -> AppResult<UpdateInfo> {
    let (http, _net) = state.net_scope();
    updater::check(&http).await
}

#[tauri::command]
pub async fn download_and_install_launcher_update(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let (http, _net) = state.net_scope();
    let info = updater::check(&http).await?;
    if !info.update_available {
        return Err(crate::error::AppError::msg("No hay actualizaciones disponibles"));
    }
    updater::download_and_install(&app, &http, &info).await
}
