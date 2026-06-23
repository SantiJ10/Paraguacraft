//! Comandos de instancias: escaneo/import, CRUD y backups.

use crate::core::instances::{self, backups, importers, profiles, scan, InstanceMeta};
use crate::error::AppResult;
use crate::models::{BackupInfo, Instance};

/// Escanea instancias locales + de otros launchers (Vanilla/Lunar/Prism/TLauncher/SK).
#[tauri::command]
pub fn scan_instances() -> Vec<Instance> {
    scan::scan_all()
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
) -> AppResult<InstanceMeta> {
    profiles::set_config(&id, ram_mb, jvm_args, gc, java_path)
}

/// Activa/desactiva la autogestion por hardware.
#[tauri::command]
pub fn set_instance_auto_managed(id: String, auto: bool) -> AppResult<InstanceMeta> {
    profiles::set_auto_managed(&id, auto)
}

#[tauri::command]
pub fn list_instance_content(id: String) -> AppResult<Vec<instances::content::InstanceContentItem>> {
    instances::content::list(&id)
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
    Ok(meta.into_instance(&id, &dir))
}
