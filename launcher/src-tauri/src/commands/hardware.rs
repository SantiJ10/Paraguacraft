//! Comando IPC de hardware. Delega en `core::hardware`.

use crate::core::hardware;
use crate::models::HardwareInfo;

#[tauri::command]
pub fn get_hardware_info() -> Result<HardwareInfo, String> {
    Ok(hardware::detect())
}
