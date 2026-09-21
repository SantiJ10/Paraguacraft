//! Control de la cola de descargas (pausa / cancelar / abrir carpeta).

use std::path::PathBuf;

use crate::core::instances;
use crate::core::net;
use crate::error::AppResult;

#[tauri::command]
pub fn download_control(id: String, action: String) -> AppResult<()> {
    net::control_group(&id, &action)
}

#[tauri::command]
pub fn reveal_download_path(path: String) -> AppResult<()> {
    let p = PathBuf::from(path.trim());
    if p.as_os_str().is_empty() {
        return Err(crate::error::AppError::msg("Sin carpeta para abrir"));
    }
    if p.is_file() {
        instances::content::reveal_abs(&p)
    } else {
        instances::content::open_abs(&p)
    }
}
