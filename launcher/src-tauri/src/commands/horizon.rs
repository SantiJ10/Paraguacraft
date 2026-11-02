//! Comandos IPC para la integración de Horizon (motor nativo Rust + Vulkan para MC 26.3).

use std::path::PathBuf;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::core::horizon::{self, HorizonStatus};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub async fn get_horizon_status(
    state: State<'_, AppState>,
    check_remote: Option<bool>,
) -> AppResult<HorizonStatus> {
    if check_remote.unwrap_or(false) {
        let (http, _net) = state.net_scope();
        Ok(horizon::status(Some(&http)).await)
    } else {
        Ok(horizon::status(None).await)
    }
}

#[tauri::command]
pub async fn prepare_horizon_assets(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<HorizonStatus> {
    let (http, _net) = state.net_scope();
    horizon::prepare_assets(&app, &http).await?;
    Ok(horizon::status(None).await)
}

#[tauri::command]
pub async fn install_horizon_engine(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<HorizonStatus> {
    let (http, _net) = state.net_scope();
    horizon::install_engine(&app, &http).await
}

#[tauri::command]
pub async fn pick_horizon_binary(app: AppHandle) -> AppResult<Option<HorizonStatus>> {
    horizon::require_official_microsoft_account()?;
    let picked = tokio::task::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("Seleccionar ejecutable de Horizon (.exe)")
            .add_filter("Ejecutable Windows", &["exe"])
            .blocking_pick_file()
    })
    .await
    .map_err(|e| AppError::msg(format!("Diálogo abortado: {e}")))?;

    let Some(file_path) = picked else {
        return Ok(None);
    };
    let path: PathBuf = file_path
        .into_path()
        .map_err(|e| AppError::msg(format!("Ruta inválida: {e}")))?;
    let st = horizon::set_custom_binary(&path)?;
    Ok(Some(st))
}

#[tauri::command]
pub async fn launch_horizon_engine(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<u32> {
    let (http, _net) = state.net_scope();
    horizon::launch(&app, &http).await
}

#[tauri::command]
pub fn open_horizon_folder() -> AppResult<()> {
    horizon::open_folder()
}
