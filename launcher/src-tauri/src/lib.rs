//! Paraguacraft Launcher - backend Rust.
//!
//! Arquitectura modular (cero espagueti):
//!   - `commands/`  : handlers IPC finos expuestos a la UI via `invoke`.
//!   - `core/`      : logica de dominio por modulo (hardware, java, instances...).
//!   - `models/`    : structs compartidos con la UI (serde camelCase).
//!   - `state.rs`   : estado global de la app.
//!
//! Fase 1: solo `get_hardware_info` esta implementado de verdad para validar
//! el puente Tauri <-> Vue. El resto de los modulos son esqueletos listos para
//! las Fases 2-4.

mod commands;
mod core;
mod models;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![commands::hardware::get_hardware_info])
        .run(tauri::generate_context!())
        .expect("error al iniciar Paraguacraft Launcher");
}
