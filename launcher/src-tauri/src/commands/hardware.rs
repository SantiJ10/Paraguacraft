//! Comando IPC de hardware. Delega en `core::hardware`.

use crate::core::hardware;
use crate::models::{HardwareInfo, SystemSpecs};

/// Async + spawn_blocking: detect() usa PowerShell/sysinfo y no debe bloquear el hilo UI/IPC.
#[tauri::command]
pub async fn get_hardware_info() -> Result<HardwareInfo, String> {
    tokio::task::spawn_blocking(hardware::detect)
        .await
        .map_err(|e| format!("hardware task: {e}"))
}

/// RAM / CPU / OS para validar requisitos de un modpack en la tienda.
#[tauri::command]
pub async fn get_system_specs() -> Result<SystemSpecs, String> {
    let hw = tokio::task::spawn_blocking(hardware::detect)
        .await
        .map_err(|e| format!("hardware task: {e}"))?;
    Ok(SystemSpecs {
        ram_gb: hw.ram_gb,
        cpu_threads: hw.cpu_threads,
        cpu_cores: hw.cpu_cores,
        os: hw.os,
    })
}
