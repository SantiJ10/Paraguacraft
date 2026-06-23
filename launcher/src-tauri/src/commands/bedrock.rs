//! Comandos IPC para Minecraft: Bedrock Edition.

use tauri::AppHandle;

use crate::config;
use crate::core::accounts;
use crate::core::bedrock;
use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;

#[tauri::command]
pub fn get_bedrock_status() -> bedrock::BedrockStatus {
    bedrock::status()
}

#[tauri::command]
pub fn launch_bedrock(app: AppHandle) -> AppResult<()> {
    let account = accounts::active_account()
        .ok_or_else(|| AppError::msg("No hay cuenta activa. Agrega una en Ajustes."))?;
    if !account.premium {
        return Err(AppError::msg(
            "Se necesita cuenta Premium (Microsoft) para jugar Minecraft: Bedrock Edition",
        ));
    }

    let settings = config::read_json::<AppSettings>(&paths::config_file()).unwrap_or_default();
    let username = account.username.clone();

    bedrock::launch(&username)?;
    bedrock::watch_session(app, username, settings.close_on_launch);
    Ok(())
}
