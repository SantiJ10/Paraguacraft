//! Modo Competir y presupuesto de recursos.

use crate::core::compete_mode;
use crate::core::instances;
use crate::error::AppResult;

#[tauri::command]
pub fn get_resource_budget(instance_id: String) -> AppResult<compete_mode::ResourceBudget> {
    let meta = instances::ensure_meta(&instance_id)?;
    let settings: crate::models::AppSettings =
        crate::config::read_json(&crate::core::paths::config_file()).unwrap_or_default();
    let ram = if meta.ram_mb > 0 {
        meta.ram_mb
    } else {
        settings.ram_mb
    };
    Ok(compete_mode::resource_budget(ram))
}
