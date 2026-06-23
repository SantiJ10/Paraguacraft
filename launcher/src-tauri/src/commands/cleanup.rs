//! Limpieza al cerrar el launcher (servidores locales + playit).

use crate::core::servers;

#[tauri::command]
pub fn shutdown_background_services() {
    servers::stop_all_running();
}
