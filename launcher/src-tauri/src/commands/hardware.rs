//! Comando IPC de hardware. Delega en `core::hardware`.

use crate::core::hardware;
use crate::models::HardwareInfo;

/// Async + spawn_blocking: detect() usa PowerShell/sysinfo y no debe bloquear el hilo UI/IPC.
#[tauri::command]
pub async fn get_hardware_info() -> Result<HardwareInfo, String> {
    tokio::task::spawn_blocking(hardware::detect)
        .await
        .map_err(|e| format!("hardware task: {e}"))
}
