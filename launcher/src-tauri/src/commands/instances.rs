//! Comandos de instancias: escaneo/import, CRUD y backups.

use crate::core::instances::{self, backups, importers, profiles, InstanceMeta};
use crate::error::AppResult;
use crate::models::{BackupInfo, Instance};

/// Escanea instancias locales + de otros launchers (Vanilla/Lunar/Prism/TLauncher/SK).
#[tauri::command]
pub async fn scan_instances() -> Vec<Instance> {
    let local = instances::list_local();
    let external = tokio::task::spawn_blocking(instances::scan::scan_external)
        .await
        .unwrap_or_default();
    let mut out = local;
    out.extend(external);
    out
}

/// Solo las instancias del ecosistema Paraguacraft (mas rapido).
#[tauri::command]
pub fn list_instances() -> Vec<Instance> {
    instances::list_local()
}

#[tauri::command]
pub fn create_instance(
    name: String,
    mc_version: String,
    loader: String,
    loader_version: String,
    icon: String,
    ram_mb: u32,
) -> AppResult<Instance> {
    profiles::create(&name, &mc_version, &loader, &loader_version, &icon, ram_mb)
}

#[tauri::command]
pub fn rename_instance(id: String, name: String, icon: String) -> AppResult<Instance> {
    profiles::rename(&id, &name, &icon)
}

#[tauri::command]
pub fn set_instance_ram(id: String, ram_mb: u32) -> AppResult<Instance> {
    profiles::set_ram(&id, ram_mb)
}

#[tauri::command]
pub fn duplicate_instance(id: String, new_name: String) -> AppResult<Instance> {
    profiles::duplicate(&id, &new_name)
}

#[tauri::command]
pub fn delete_instance(id: String) -> AppResult<()> {
    profiles::delete(&id)
}

#[tauri::command]
pub fn import_instance(id: String) -> AppResult<Instance> {
    importers::import(&id)
}

#[tauri::command]
pub fn create_backup(id: String) -> AppResult<BackupInfo> {
    backups::create(&id)
}

#[tauri::command]
pub fn list_backups(id: String) -> Vec<BackupInfo> {
    backups::list(&id)
}

#[tauri::command]
pub fn restore_backup(id: String, name: String) -> AppResult<()> {
    backups::restore(&id, &name)
}

#[tauri::command]
pub fn delete_backup(id: String, name: String) -> AppResult<()> {
    backups::delete(&id, &name)
}

// ── Override de configuracion por instancia (Regla 2) ───────────────────────

/// Devuelve la metadata completa (incluye ram/jvm/gc/java_path + auto_managed).
#[tauri::command]
pub fn get_instance_meta(id: String) -> AppResult<InstanceMeta> {
    if id.starts_with("ext::") {
        return instances::resolve_external_meta(&id)
            .ok_or_else(|| crate::error::AppError::msg("Instancia no encontrada"));
    }
    instances::ensure_meta(&id)
}

/// Guarda overrides de JVM (marca auto_managed=false). Cualquier campo None se
/// limpia salvo ram_mb (que se conserva si no se envia).
#[tauri::command]
pub fn set_instance_config(
    id: String,
    ram_mb: Option<u32>,
    jvm_args: Option<String>,
    gc: Option<String>,
    java_path: Option<String>,
    performance_tier: Option<String>,
    show_game_console: Option<String>,
) -> AppResult<InstanceMeta> {
    profiles::set_config(
        &id,
        ram_mb,
        jvm_args,
        gc,
        java_path,
        performance_tier,
        show_game_console,
    )
}

/// Activa/desactiva la autogestion por hardware.
#[tauri::command]
pub fn set_instance_auto_managed(id: String, auto: bool) -> AppResult<InstanceMeta> {
    profiles::set_auto_managed(&id, auto)
}

#[tauri::command]
pub async fn list_instance_content(
    state: tauri::State<'_, crate::state::AppState>,
    id: String,
) -> AppResult<Vec<instances::content::InstanceContentItem>> {
    let mut items = instances::content::list(&id)?;
    if let Some(base) = instances::game_dir_for(&id) {
        let (http, _guard) = state.net_scope();
        let _ = instances::content_metadata::enrich(&http, &id, &base, &mut items).await;
    }
    Ok(items)
}

#[tauri::command]
pub fn toggle_instance_content(id: String, path: String, enabled: bool) -> AppResult<()> {
    instances::content::toggle(&id, &path, enabled)
}

#[tauri::command]
pub fn open_instance_folder(id: String) -> AppResult<()> {
    instances::content::open_folder(&id)
}

#[tauri::command]
pub fn get_instance_folder_path(id: String) -> AppResult<String> {
    Ok(instances::content::folder_path(&id)?
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
pub fn set_instance_loader(id: String, loader: String, loader_version: String) -> AppResult<InstanceMeta> {
    profiles::set_loader(&id, &loader, &loader_version)
}

#[tauri::command]
pub fn get_instance_icon_path(icon: String) -> Option<String> {
    instances::icons::resolve_path(&icon).map(|p| p.to_string_lossy().to_string())
}

/// Preferido para UI: data URL (no depende del asset protocol de Tauri).
#[tauri::command]
pub fn get_instance_icon_data(icon: String) -> Option<String> {
    instances::icons::as_data_url(&icon)
}

#[tauri::command]
pub fn import_instance_icon(source_path: String) -> AppResult<instances::icons::ImportIconResult> {
    instances::icons::import_from_path(std::path::Path::new(&source_path))
}

#[tauri::command]
pub async fn pick_and_import_instance_icon(
    app: tauri::AppHandle,
) -> AppResult<instances::icons::ImportIconResult> {
    use tauri_plugin_dialog::DialogExt;

    let file = app
        .dialog()
        .file()
        .add_filter("Imagen", &["png", "jpg", "jpeg", "webp"])
        .blocking_pick_file();

    let Some(file) = file else {
        return Err(crate::error::AppError::msg("No se seleccionó ningún archivo"));
    };

    let path = file
        .into_path()
        .map_err(|e| crate::error::AppError::msg(format!("Ruta inválida: {e}")))?;

    instances::icons::import_from_path(&path)
}

#[tauri::command]
pub async fn reinstall_instance_loader(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
    id: String,
) -> AppResult<Instance> {
    let meta = instances::read_meta(&id)
        .or_else(|| instances::resolve_meta(&id))
        .ok_or_else(|| crate::error::AppError::msg("Instancia no encontrada"))?;
    let (http, _net) = state.net_scope();
    let version_id = crate::core::loaders::install_loader(
        &app,
        &http,
        &meta.mc_version,
        &meta.loader,
        &meta.loader_version,
    )
    .await?;
    profiles::set_version_id(&id, &version_id)?;
    let meta = instances::ensure_meta(&id)?;
    let dir = instances::instance_dir(&id);
    let loader = crate::core::loaders::normalize(&meta.loader);
    if loader == "paraguacraft-optimized" || loader == "paraguacraft-optimized-neoforge" {
        let _ = crate::core::loaders::optimized::install_bundle_for_launch(
            &app,
            &http,
            &meta.mc_version,
            &loader,
            &dir,
        )
        .await;
    } else if loader == "fabric-iris" {
        let _ = crate::core::loaders::fabric_iris::install_bundle(
            &app,
            &http,
            &meta.mc_version,
            &dir,
        )
        .await;
    } else if loader == "paraguacraft-pvp" {
        let _ = crate::core::loaders::pvp::install_bundle(&app, &http, &dir).await;
    } else if loader == "paraguacraft-pvp-modern" {
        let _ = crate::core::modern_pvp::sync_instance_bundles(&app, &http, &id).await;
    }
    let meta = instances::ensure_meta(&id)?;
    Ok(meta.into_instance(&id, &dir))
}

#[tauri::command]
pub async fn repair_instance(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
    id: String,
) -> AppResult<crate::core::server_repair::ServerRepairReport> {
    let (http, _net) = state.net_scope();
    crate::core::instance_repair::repair(&app, &http, &id).await
}

#[tauri::command]
pub fn get_instance_log(id: String, max_lines: Option<usize>) -> AppResult<Vec<String>> {
    crate::core::instance_repair::read_log_lines(&id, max_lines.unwrap_or(200))
}

/// Consola del cliente en vivo (buffer del tail de latest.log mientras jugás).
/// Si no hay sesión, devuelve las últimas líneas del archivo.
#[tauri::command]
pub fn get_client_console(id: String, max_lines: Option<usize>) -> AppResult<Vec<String>> {
    let max = max_lines.unwrap_or(500);
    let live = crate::core::client_console::get_lines(&id, max);
    if !live.is_empty() {
        return Ok(live);
    }
    crate::core::instance_repair::read_log_lines(&id, max)
}

#[tauri::command]
pub fn export_client_console(id: String) -> AppResult<String> {
    let dir = instances::content::folder_path(&id)?;
    crate::core::client_console::export_to_file(&id, &dir)
}

/// Abre en el explorador: `log` → latest.log, `crashes` → crash-reports/, `folder` → raíz.
#[tauri::command]
pub fn open_instance_path(id: String, kind: String) -> AppResult<()> {
    let base = instances::content::folder_path(&id)?;
    match kind.as_str() {
        "log" => {
            let path = base.join("logs").join("latest.log");
            if path.is_file() {
                instances::content::reveal_abs(&path)
            } else {
                let logs = base.join("logs");
                if logs.is_dir() {
                    instances::content::open_abs(&logs)
                } else {
                    instances::content::open_folder(&id)
                }
            }
        }
        "crashes" => {
            let crash = base.join("crash-reports");
            if crash.is_dir() {
                instances::content::open_abs(&crash)
            } else {
                std::fs::create_dir_all(&crash)?;
                instances::content::open_abs(&crash)
            }
        }
        _ => instances::content::open_folder(&id),
    }
}

#[tauri::command]
pub fn remove_instance_content(id: String, path: String) -> AppResult<()> {
    instances::content::remove(&id, &path)
}

#[tauri::command]
pub fn reveal_instance_content(id: String, path: String) -> AppResult<()> {
    instances::content::reveal(&id, &path)
}

#[tauri::command]
pub async fn pick_and_add_instance_content(
    app: tauri::AppHandle,
    id: String,
    folder: String,
) -> AppResult<u32> {
    use tauri_plugin_dialog::DialogExt;

    let picked = app
        .dialog()
        .file()
        .add_filter("Mods / packs", &["jar", "zip"])
        .blocking_pick_files();

    let Some(files) = picked else {
        return Err(crate::error::AppError::msg("No se seleccionó ningún archivo"));
    };

    let paths: Vec<std::path::PathBuf> = files
        .into_iter()
        .filter_map(|f| f.into_path().ok())
        .collect();
    instances::content::add_files(&id, &folder, &paths)
}

#[tauri::command]
pub async fn pick_and_export_instance(
    app: tauri::AppHandle,
    id: String,
) -> AppResult<String> {
    use tauri_plugin_dialog::DialogExt;

    let meta = instances::ensure_meta(&id)?;
    let safe_name = meta
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    let default = format!("{safe_name}_export.zip");

    let picked = app
        .dialog()
        .file()
        .set_file_name(&default)
        .add_filter("Zip", &["zip"])
        .blocking_save_file();

    let Some(file) = picked else {
        return Err(crate::error::AppError::msg("Exportación cancelada"));
    };
    let path = file
        .into_path()
        .map_err(|e| crate::error::AppError::msg(format!("Ruta inválida: {e}")))?;
    instances::export::export_to(&id, &path)
}
