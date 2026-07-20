//! Modo Competir y presupuesto de recursos.

use crate::core::compete_mode;
use crate::core::hardware;
use crate::core::instances;
use crate::core::loaders;
use crate::error::{AppError, AppResult};

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

/// Re-aplica el preset PvP (culling/HUD/toggle sprint) de la instancia segun
/// su loader y el tier de hardware detectado. Util cuando Paraguabot sugiere
/// "Sincronizar PvP" tras un ajuste manual que desalineo la config in-game.
#[tauri::command]
pub fn sync_pvp_config(instance_id: String) -> AppResult<String> {
    let meta = instances::ensure_meta(&instance_id)?;
    let normalized = loaders::normalize(&meta.loader);
    let game_dir = instances::instance_dir(&instance_id);
    let hw = hardware::detect();

    if normalized == "paraguacraft-pvp-modern" {
        compete_mode::apply_modern_compete_profile(&game_dir, &hw.perfil_sugerido)?;
        Ok("Configuracion PvP Modern sincronizada.".into())
    } else if normalized == "paraguacraft-pvp" {
        compete_mode::apply_pvp_compete_profile(&game_dir, &hw.perfil_sugerido)?;
        Ok("Configuracion PvP 1.8.9 sincronizada.".into())
    } else {
        Err(AppError::msg(
            "Esta instancia no usa el loader PvP Paraguacraft (Modern o 1.8.9).",
        ))
    }
}
