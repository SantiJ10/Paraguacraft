//! Comandos de Java: deteccion, verificacion, descarga de Temurin.

use std::path::Path;

use tauri::{AppHandle, State};

use crate::core::java;
use crate::error::AppResult;
use crate::models::JavaInstallation;
use crate::state::AppState;

/// Detecta Javas. Con `force_refresh` ignora la cache.
#[tauri::command]
pub fn detect_javas(state: State<'_, AppState>, force_refresh: Option<bool>) -> Vec<JavaInstallation> {
    if !force_refresh.unwrap_or(false) {
        let cache = state.java_cache.lock().unwrap();
        if let Some(found) = cache.as_ref() {
            return found.clone();
        }
    } else {
        *state.java_cache.lock().unwrap() = None;
    }
    let found = java::detect::detect_all();
    *state.java_cache.lock().unwrap() = Some(found.clone());
    found
}

/// Verifica una ruta de Java concreta (selector "elegir Java").
#[tauri::command]
pub fn verify_java_path(path: String) -> Option<JavaInstallation> {
    java::verify::verify(Path::new(&path), "custom")
}

#[tauri::command]
pub fn java_info_for_mc(version: String) -> serde_json::Value {
    let required = java::required_for_mc(&version);
    let components = java::mojang::component_candidates(&version);
    serde_json::json!({
        "requiredMajor": required,
        "mojangComponents": components,
    })
}

/// Major de Java requerido por una version de Minecraft.
#[tauri::command]
pub fn java_required_for_mc(version: String) -> u32 {
    java::required_for_mc(&version)
}

/// Descarga (o reutiliza) Temurin del major dado. Emite progreso via eventos.
#[tauri::command]
pub async fn download_temurin(
    app: AppHandle,
    state: State<'_, AppState>,
    major: u32,
    force: bool,
) -> AppResult<String> {
    // Invalidamos la cache antes del await (sin sostener el guard cruzando await).
    {
        *state.java_cache.lock().unwrap() = None;
    }
    // net_scope libera el cliente HTTP al terminar (Regla 3).
    let (http, _net) = state.net_scope();
    java::adoptium::download(&app, &http, major, force).await
}

/// Descarga (o reutiliza) Azul Zulu del major dado.
#[tauri::command]
pub async fn download_zulu(
    app: AppHandle,
    state: State<'_, AppState>,
    major: u32,
    force: bool,
) -> AppResult<String> {
    {
        *state.java_cache.lock().unwrap() = None;
    }
    let (http, _net) = state.net_scope();
    java::zulu::download(&app, &http, major, force).await
}

/// Runtimes oficiales de Mojang disponibles para este PC.
#[tauri::command]
pub async fn list_mojang_runtimes(
    state: State<'_, AppState>,
) -> AppResult<Vec<java::mojang::MojangRuntimeInfo>> {
    let (http, _net) = state.net_scope();
    java::mojang::list_runtimes(&http).await
}

/// Descarga (o reutiliza) un runtime Java de Mojang.
#[tauri::command]
pub async fn download_mojang_runtime(
    app: AppHandle,
    state: State<'_, AppState>,
    component: String,
) -> AppResult<String> {
    {
        *state.java_cache.lock().unwrap() = None;
    }
    let (http, _net) = state.net_scope();
    let path = java::mojang::ensure_runtime(&app, &http, &component).await?;
    Ok(path.to_string_lossy().to_string())
}

/// Diálogo para elegir `java.exe` / `javaw.exe` del sistema.
#[tauri::command]
pub async fn pick_java_executable(app: AppHandle) -> AppResult<JavaInstallation> {
    use tauri_plugin_dialog::DialogExt;

    let file = app
        .dialog()
        .file()
        .add_filter("Java", &["exe"])
        .set_title("Elegir Java")
        .blocking_pick_file();

    let Some(file) = file else {
        return Err(crate::error::AppError::msg("No se seleccionó ningún archivo"));
    };
    let path = file
        .into_path()
        .map_err(|e| crate::error::AppError::msg(format!("Ruta inválida: {e}")))?;
    java::verify::verify(&path, "custom").ok_or_else(|| {
        crate::error::AppError::msg("Ese archivo no es un Java válido")
    })
}
