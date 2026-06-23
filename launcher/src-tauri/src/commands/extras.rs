//! Comandos de Extras (rendimiento, mantenimiento, Discord).

use crate::config;
use crate::core::extras::{self, maintenance};
use crate::core::paths;
use crate::error::AppResult;
use crate::models::AppSettings;

#[tauri::command]
pub fn get_extras_status() -> extras::ExtrasStatus {
    extras::status()
}

#[tauri::command]
pub fn activate_game_mode() -> AppResult<Vec<String>> {
    extras::game_mode::activate()
}

#[tauri::command]
pub fn deactivate_game_mode() -> AppResult<Vec<String>> {
    extras::game_mode::deactivate()
}

#[tauri::command]
pub fn activate_turbo_mode() -> AppResult<Vec<String>> {
    extras::turbo::activate()
}

#[tauri::command]
pub fn deactivate_turbo_mode() -> AppResult<Vec<String>> {
    extras::turbo::deactivate()
}

#[tauri::command]
pub fn set_java_priority(level: String) -> AppResult<u32> {
    extras::java_priority::set_level(&level)
}

#[tauri::command]
pub fn get_cleanup_info() -> maintenance::CleanupInfo {
    maintenance::info()
}

#[tauri::command]
pub fn run_cleanup(kind: String) -> AppResult<u32> {
    maintenance::run(&kind)
}

#[tauri::command]
pub fn sync_discord_rpc() {
    let settings: AppSettings = config::read_json(&paths::config_file()).unwrap_or_default();
    if settings.discord_rpc {
        extras::discord_rpc::connect(true);
        if let Some(acc) = crate::core::accounts::active_account() {
            let user = acc.username.replace(" [PREMIUM]", "");
            extras::discord_rpc::set_launcher_idle(&user);
        }
    } else {
        extras::discord_rpc::disconnect();
    }
}
