//! Comandos de configuracion (persistencia real en `launcher_config.json`).

use tauri::AppHandle;

use crate::config;
use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;

#[tauri::command]
pub fn get_settings() -> AppSettings {
    use crate::core::performance;

    let path = paths::config_file();
    let exists = path.is_file();
    let mut settings: AppSettings = config::read_json(&path).unwrap_or_default();
    if !settings.hardware_defaults_applied {
        if !exists || (settings.ram_mb == 4096 && settings.gc_type == "Auto") {
            let _ = performance::apply_hardware_defaults(&mut settings);
        } else {
            settings.hardware_defaults_applied = true;
        }
        let _ = config::write_json_atomic(&path, &settings);
    }
    settings
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> AppResult<()> {
    // La key de CF vive en app_secrets.json (no en el config publico si viene de .env).
    if let Some(ref k) = settings.curseforge_api_key {
        if !k.trim().is_empty() {
            config::keys::save_curseforge_api_key(k)?;
        }
    }
    config::write_json_atomic(&paths::config_file(), &settings)?;
    if settings.discord_rpc {
        crate::core::extras::discord_rpc::connect(true);
        if let Some(acc) = crate::core::accounts::active_account() {
            let user = acc.username.replace(" [PREMIUM]", "");
            crate::core::extras::discord_rpc::set_launcher_idle(&user);
        }
    } else {
        crate::core::extras::discord_rpc::disconnect();
    }
    Ok(())
}

fn safe_css_name(name: &str) -> AppResult<String> {
    let name = name.trim().replace('\\', "/");
    let base = name.rsplit('/').next().unwrap_or("");
    if base.is_empty()
        || base.contains("..")
        || !base.to_ascii_lowercase().ends_with(".css")
    {
        return Err(crate::error::AppError::msg("Nombre de tema inválido"));
    }
    Ok(base.to_string())
}

fn list_css_in(dir: &std::path::Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            if n.to_ascii_lowercase().ends_with(".css") {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

fn write_theme_readme(dir: &std::path::Path) {
    let readme = dir.join("LEEME.txt");
    if readme.is_file() {
        return;
    }
    let _ = std::fs::write(
        readme,
        "Poné archivos .css acá y recargá desde Ajustes → Apariencia.\n\
         El CSS se aplica sobre el launcher (variables --surface-* y --pc-accent).\n",
    );
}

/// Temas CSS del usuario + packs de iconos.
#[tauri::command]
pub fn list_custom_themes() -> serde_json::Value {
    let themes = paths::themes_dir();
    let icons = paths::icon_themes_dir();
    write_theme_readme(&themes);
    write_theme_readme(&icons);
    serde_json::json!({
        "themes": list_css_in(&themes),
        "iconThemes": list_css_in(&icons),
        "themesDir": themes.to_string_lossy(),
        "iconThemesDir": icons.to_string_lossy(),
    })
}

#[tauri::command]
pub fn read_custom_theme_css(kind: String, name: String) -> AppResult<String> {
    let file = safe_css_name(&name)?;
    let dir = if kind == "icons" {
        paths::icon_themes_dir()
    } else {
        paths::themes_dir()
    };
    let path = dir.join(&file);
    if !path.is_file() {
        return Err(crate::error::AppError::msg("Tema no encontrado"));
    }
    let bytes = std::fs::read(&path)?;
    if bytes.len() > 256 * 1024 {
        return Err(crate::error::AppError::msg("El CSS supera 256 KB"));
    }
    String::from_utf8(bytes).map_err(|_| crate::error::AppError::msg("El CSS no es UTF-8"))
}

#[tauri::command]
pub fn open_custom_themes_folder(kind: String) -> AppResult<()> {
    let dir = if kind == "icons" {
        paths::icon_themes_dir()
    } else {
        paths::themes_dir()
    };
    write_theme_readme(&dir);
    crate::core::instances::content::open_abs(&dir)
}

#[tauri::command]
pub async fn pick_wallpaper_file(app: AppHandle) -> AppResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let file = app
        .dialog()
        .file()
        .add_filter("Imagen", &["png", "jpg", "jpeg", "webp"])
        .blocking_pick_file();
    let Some(file) = file else {
        return Ok(None);
    };
    let path = file
        .into_path()
        .map_err(|e| AppError::msg(format!("Ruta inválida: {e}")))?;
    Ok(Some(path.to_string_lossy().to_string()))
}
