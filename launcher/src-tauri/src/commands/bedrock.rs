//! Comandos IPC para Minecraft: Bedrock Edition.

use tauri::{AppHandle, State};

use crate::config;
use crate::core::accounts;
use crate::core::bedrock;
use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use crate::state::AppState;

fn require_premium() -> AppResult<String> {
    let account = accounts::active_account()
        .ok_or_else(|| AppError::msg("No hay cuenta activa. Agrega una en Ajustes."))?;
    if !account.premium {
        return Err(AppError::msg(
            "Se necesita cuenta Premium (Microsoft) para jugar Minecraft: Bedrock Edition",
        ));
    }
    Ok(account.username)
}

fn watch_after_launch(app: AppHandle, username: String) {
    let settings = config::read_json::<AppSettings>(&paths::config_file()).unwrap_or_default();
    bedrock::watch_session(app, username, settings.close_on_launch);
}

#[tauri::command]
pub fn get_bedrock_status() -> bedrock::BedrockStatus {
    bedrock::status()
}

#[tauri::command]
pub fn launch_bedrock(app: AppHandle) -> AppResult<()> {
    let username = require_premium()?;
    bedrock::launch(&username)?;
    watch_after_launch(app, username);
    Ok(())
}

#[tauri::command]
pub async fn list_bedrock_versions(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> AppResult<Vec<bedrock::BedrockVersion>> {
    require_premium()?;
    let (http, _net) = state.net_scope();
    bedrock::list_catalog(&http, force.unwrap_or(false)).await
}

#[tauri::command]
pub fn list_installed_bedrock_versions() -> AppResult<Vec<bedrock::BedrockInstalledVersion>> {
    require_premium()?;
    Ok(bedrock::list_installed())
}

#[tauri::command]
pub async fn install_bedrock_version(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
) -> AppResult<()> {
    require_premium()?;
    let (http, _net) = state.net_scope();
    bedrock::install_version(&app, &http, &version).await
}

#[tauri::command]
pub fn switch_bedrock_version(version: String) -> AppResult<()> {
    require_premium()?;
    bedrock::switch_version(&version)
}

#[tauri::command]
pub fn remove_bedrock_version(version: String) -> AppResult<()> {
    require_premium()?;
    bedrock::remove_version(&version)
}

#[tauri::command]
pub fn launch_bedrock_version(app: AppHandle, version: String) -> AppResult<()> {
    let username = bedrock::launch_version(&version)?;
    watch_after_launch(app, username);
    Ok(())
}

#[tauri::command]
pub fn bedrock_developer_mode() -> bool {
    bedrock::developer_mode()
}

#[tauri::command]
pub fn enable_bedrock_developer_mode() -> AppResult<()> {
    require_premium()?;
    bedrock::enable_developer_mode()
}

#[tauri::command]
pub fn backup_bedrock_saves() -> AppResult<String> {
    require_premium()?;
    bedrock::backup_saves()
}

#[tauri::command]
pub fn open_microsoft_store_bedrock() -> AppResult<()> {
    require_premium()?;
    bedrock::open_microsoft_store()
}

#[tauri::command]
pub fn list_bedrock_worlds() -> AppResult<Vec<bedrock::BedrockWorld>> {
    require_premium()?;
    Ok(bedrock::list_worlds())
}

#[tauri::command]
pub fn list_bedrock_packs() -> AppResult<Vec<bedrock::BedrockPack>> {
    require_premium()?;
    Ok(bedrock::list_packs())
}

#[tauri::command]
pub fn open_bedrock_folder(kind: String) -> AppResult<()> {
    require_premium()?;
    bedrock::open_content_folder(&kind)
}

#[tauri::command]
pub fn delete_bedrock_world(id: String) -> AppResult<()> {
    require_premium()?;
    bedrock::delete_world(&id)
}

#[tauri::command]
pub fn delete_bedrock_pack(kind: String, id: String) -> AppResult<()> {
    require_premium()?;
    bedrock::delete_pack(&kind, &id)
}

#[tauri::command]
pub async fn import_bedrock_pack(app: AppHandle) -> AppResult<String> {
    require_premium()?;
    use tauri_plugin_dialog::DialogExt;

    let picked = app
        .dialog()
        .file()
        .add_filter("Mundos y packs Bedrock", &["mcworld", "mcpack", "mcaddon", "zip"])
        .set_title("Importar mundo o pack Bedrock")
        .blocking_pick_file();
    let Some(file) = picked else {
        return Err(AppError::msg("No se seleccionó ningún archivo"));
    };
    let path = file
        .into_path()
        .map_err(|e| AppError::msg(format!("Ruta inválida: {e}")))?;
    tokio::task::spawn_blocking(move || bedrock::import_archive(&path))
        .await
        .unwrap_or_else(|e| Err(AppError::msg(format!("Import abortado: {e}"))))
}
