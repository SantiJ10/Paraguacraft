//! Limpieza al cerrar el launcher (servidores locales + playit).

use crate::core::extras::discord_rpc;
use crate::core::servers;

#[tauri::command]
pub fn shutdown_background_services() {
    discord_rpc::shutdown();
    std::thread::spawn(|| {
        servers::stop_all_running();
    });
}
