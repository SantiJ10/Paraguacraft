//! Comandos IPC para Minecraft: Bedrock Edition.

use tauri::{AppHandle, State};

use crate::config;
use crate::core::accounts;
use crate::core::bedrock;
use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use crate::state::AppState;

async fn run_blocking<T: Send + 'static>(
    f: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    tokio::task::spawn_blocking(f)
        .await
        .unwrap_or_else(|e| Err(AppError::msg(format!("Tarea abortada: {e}"))))
}

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

fn watch_after_launch(app: AppHandle, username: String, version: Option<String>) {
    let settings = config::read_json::<AppSettings>(&paths::config_file()).unwrap_or_default();
    bedrock::watch_session(app, username, settings.close_on_launch, version);
}

#[tauri::command]
pub async fn get_bedrock_status() -> bedrock::BedrockStatus {
    tokio::task::spawn_blocking(bedrock::status)
        .await
        .unwrap_or_else(|_| bedrock::BedrockStatus {
            platform_supported: cfg!(windows),
            installed: false,
            premium_allowed: false,
            username: None,
            store_installed: false,
            managed_active: false,
            active_version: None,
            developer_mode: false,
            conflict_store: false,
        })
}

#[tauri::command]
pub async fn launch_bedrock(app: AppHandle) -> AppResult<()> {
    let (username, version) = run_blocking(|| {
        let username = require_premium()?;
        bedrock::launch(&username)?;
        Ok((username, bedrock::status().active_version))
    })
    .await?;
    watch_after_launch(app, username, version);
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
pub async fn list_installed_bedrock_versions() -> AppResult<Vec<bedrock::BedrockInstalledVersion>> {
    run_blocking(|| {
        require_premium()?;
        Ok(bedrock::list_installed())
    })
    .await
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
pub async fn switch_bedrock_version(version: String) -> AppResult<()> {
    run_blocking(move || {
        require_premium()?;
        bedrock::switch_version(&version)
    })
    .await
}

#[tauri::command]
pub async fn remove_bedrock_version(version: String) -> AppResult<()> {
    run_blocking(move || {
        require_premium()?;
        bedrock::remove_version(&version)
    })
    .await
}

#[tauri::command]
pub async fn launch_bedrock_version(app: AppHandle, version: String) -> AppResult<()> {
    let ver = version.clone();
    let username = run_blocking(move || bedrock::launch_version(&version)).await?;
    watch_after_launch(app, username, Some(ver));
    Ok(())
}

#[tauri::command]
pub async fn bedrock_developer_mode() -> bool {
    tokio::task::spawn_blocking(bedrock::developer_mode)
        .await
        .unwrap_or(false)
}

#[tauri::command]
pub async fn enable_bedrock_developer_mode() -> AppResult<()> {
    run_blocking(|| {
        require_premium()?;
        bedrock::enable_developer_mode()
    })
    .await
}

#[tauri::command]
pub async fn backup_bedrock_saves() -> AppResult<String> {
    run_blocking(|| {
        require_premium()?;
        bedrock::backup_saves()
    })
    .await
}

#[tauri::command]
pub async fn open_microsoft_store_bedrock() -> AppResult<()> {
    run_blocking(|| {
        require_premium()?;
        bedrock::open_microsoft_store()
    })
    .await
}

#[tauri::command]
pub async fn list_bedrock_worlds() -> AppResult<Vec<bedrock::BedrockWorld>> {
    run_blocking(|| {
        require_premium()?;
        Ok(bedrock::list_worlds())
    })
    .await
}

#[tauri::command]
pub async fn list_bedrock_packs() -> AppResult<Vec<bedrock::BedrockPack>> {
    run_blocking(|| {
        require_premium()?;
        Ok(bedrock::list_packs())
    })
    .await
}

#[tauri::command]
pub async fn open_bedrock_folder(kind: String) -> AppResult<()> {
    run_blocking(move || {
        require_premium()?;
        bedrock::open_content_folder(&kind)
    })
    .await
}

#[tauri::command]
pub async fn delete_bedrock_world(id: String) -> AppResult<()> {
    run_blocking(move || {
        require_premium()?;
        bedrock::delete_world(&id)
    })
    .await
}

#[tauri::command]
pub async fn delete_bedrock_pack(kind: String, id: String) -> AppResult<()> {
    run_blocking(move || {
        require_premium()?;
        bedrock::delete_pack(&kind, &id)
    })
    .await
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
