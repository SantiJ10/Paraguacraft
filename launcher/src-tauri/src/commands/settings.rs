//! Comandos de configuracion (persistencia real en `launcher_config.json`).

use crate::config;
use crate::core::paths;
use crate::error::AppResult;
use crate::models::AppSettings;

#[tauri::command]
pub fn get_settings() -> AppSettings {
    use crate::core::performance;

    let path = paths::config_file();
    let exists = path.is_file();
    let mut settings: AppSettings = config::read_json(&path).unwrap_or_default();
    if !settings.hardware_defaults_applied {
        if !exists || (settings.ram_mb == 4096 && settings.gc_type == "Auto") {
            let _ = performance::apply_hardware_defaults(&mut settings);
        } else {
            settings.hardware_defaults_applied = true;
        }
        let _ = config::write_json_atomic(&path, &settings);
    }
    settings
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> AppResult<()> {
    // La key de CF vive en app_secrets.json (no en el config publico si viene de .env).
    if let Some(ref k) = settings.curseforge_api_key {
        if !k.trim().is_empty() {
            config::keys::save_curseforge_api_key(k)?;
        }
    }
    config::write_json_atomic(&paths::config_file(), &settings)?;
    if settings.discord_rpc {
        crate::core::extras::discord_rpc::connect(true);
        if let Some(acc) = crate::core::accounts::active_account() {
            let user = acc.username.replace(" [PREMIUM]", "");
            crate::core::extras::discord_rpc::set_launcher_idle(&user);
        }
    } else {
        crate::core::extras::discord_rpc::disconnect();
    }
    Ok(())
}
