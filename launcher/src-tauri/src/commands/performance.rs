//! Comandos de optimización de rendimiento.

use crate::core::instances;
use crate::core::performance;
use crate::error::AppResult;

#[tauri::command]
pub fn optimize_minecraft_options() -> AppResult<performance::OptionsOptimizeResult> {
    performance::optimize_global_options()
}

#[tauri::command]
pub fn optimize_instance_options(instance_id: String) -> AppResult<performance::OptionsOptimizeResult> {
    let dir = instances::game_dir_for(&instance_id)
        .ok_or_else(|| crate::error::AppError::msg("Instancia no encontrada"))?;
    performance::optimize_instance_options(&dir)
}

#[tauri::command]
pub fn apply_recommended_performance() -> AppResult<crate::models::HardwareInfo> {
    use crate::config;
    use crate::core::paths;
    use crate::models::AppSettings;

    let mut settings: AppSettings = config::read_json(&paths::config_file()).unwrap_or_default();
    let hw = performance::apply_hardware_defaults(&mut settings);
    config::write_json_atomic(&paths::config_file(), &settings)?;
    Ok(hw)
}
