//! Handle de Tauri para emitir eventos desde hilos de dominio (Playit, etc.).

use std::sync::OnceLock;

use tauri::{AppHandle, Emitter};
use serde::Serialize;

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn set(app: AppHandle) {
    let _ = APP.set(app);
}

pub fn emit(event: &str, payload: impl Serialize + Clone) {
    if let Some(app) = APP.get() {
        let _ = app.emit(event, payload);
    }
}
