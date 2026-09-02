//! Comandos de servidores locales + Playit.gg.

use std::collections::HashMap;

use tauri::{AppHandle, State};

use crate::core::server_admin;
use crate::core::server_backups;
use crate::core::server_hangar;
use crate::core::server_properties;
use crate::core::server_repair;
use crate::core::servers::{self, ServerContentItem, ServerProfile, ServerStatus};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub fn list_servers() -> Vec<ServerProfile> {
    servers::list()
}

#[tauri::command]
pub fn create_server(
    name: String,
    mc_version: String,
    server_type: String,
    ram_mb: u32,
) -> AppResult<ServerProfile> {
    servers::create(&name, &mc_version, &server_type, ram_mb)
}

#[tauri::command]
pub fn delete_server(id: String) -> AppResult<()> {
    servers::delete(&id)
}

#[tauri::command]
pub fn server_status(id: String) -> AppResult<ServerStatus> {
    servers::status(&id)
}

#[tauri::command]
pub fn list_running_servers() -> Vec<servers::RunningServerInfo> {
    servers::list_running()
}

#[tauri::command]
pub fn update_server(
    id: String,
    name: Option<String>,
    mc_version: Option<String>,
    ram_mb: Option<u32>,
    port: Option<u16>,
) -> AppResult<ServerProfile> {
    servers::update_profile(
        &id,
        name.as_deref(),
        mc_version.as_deref(),
        ram_mb,
        port,
    )
}

#[tauri::command]
pub async fn start_server(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<u32> {
    let prof = servers::profile_by_id(&id)?;
    let dir = servers::folder_for(&prof);
    {
        let (http, _net) = state.net_scope();
        if servers::needs_prepare(&dir, &prof.server_type) {
            servers::ensure_server_jar(&app, &http, &id).await?;
        } else {
            // No bloquear el arranque: deps ausentes se rellenan en paralelo.
            let app2 = app.clone();
            let http2 = http.clone();
            let id2 = id.clone();
            tauri::async_runtime::spawn(async move {
                let _ = servers::soft_ensure_deps(&app2, &http2, &id2).await;
            });
        }
    }
    let _ = crate::core::java::resolve::ensure_server_java(&app, &state, &prof.mc_version).await?;
    let pid = servers::start_mc(&id)?;
    // Túnel siempre en playit.exe (Rust), no en el plugin de Java.
    servers::disable_playit_plugin_jars(&id);
    if servers::playit_available(&id) {
        let id_playit = id.clone();
        std::thread::Builder::new()
            .name("playit-autostart".into())
            .spawn(move || match servers::start_playit(&id_playit) {
                Ok(_) => crate::core::server_console::append(
                    &id_playit,
                    "[playit] Túnel playit.gg iniciado automáticamente.",
                ),
                Err(e) => crate::core::server_console::append(
                    &id_playit,
                    &format!("[playit] No se pudo auto-iniciar el túnel: {e}"),
                ),
            })
            .ok();
    }
    Ok(pid)
}

/// Async: `stop_mc_graceful` espera hasta 45 s. En el hilo de IPC Windows marca
/// el launcher como «No responde».
#[tauri::command]
pub async fn stop_server(id: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || servers::stop_mc_graceful(&id))
        .await
        .map_err(|e| AppError::msg(format!("stop_server: {e}")))?
}

#[tauri::command]
pub async fn stop_server_force(id: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || servers::stop(&id))
        .await
        .map_err(|e| AppError::msg(format!("stop_server_force: {e}")))?
}

#[tauri::command]
pub fn start_playit(id: String) -> AppResult<String> {
    servers::start_playit(&id)
}

#[tauri::command]
pub fn stop_playit(id: String) -> AppResult<()> {
    servers::stop_playit(&id)
}

#[tauri::command]
pub fn reset_playit(id: String) -> AppResult<String> {
    servers::reset_playit(&id)
}

#[tauri::command]
pub fn reset_playit_full(id: String) -> AppResult<String> {
    servers::reset_playit_ex(&id, true)
}

#[tauri::command]
pub fn mark_playit_claimed(id: String) -> AppResult<()> {
    servers::mark_playit_claimed(&id)
}

#[tauri::command]
pub async fn prepare_server_jar(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<String> {
    let (http, _net) = state.net_scope();
    let jar = servers::ensure_server_jar(&app, &http, &id).await?;
    Ok(jar.to_string_lossy().to_string())
}

#[tauri::command]
pub fn server_plugin_count(id: String) -> u32 {
    servers::plugin_count(&id)
}

#[tauri::command]
pub fn get_server_log(id: String, max_lines: Option<usize>) -> Vec<String> {
    servers::get_log(&id, max_lines.unwrap_or(200))
}

#[tauri::command]
pub fn export_server_log(id: String) -> AppResult<String> {
    let dir = servers::folder_for_id(&id)?;
    crate::core::server_console::export_to_file(&id, &dir)
}

#[tauri::command]
pub fn send_server_command(id: String, command: String) -> AppResult<()> {
    servers::send_command(&id, &command)
}

#[tauri::command]
pub fn read_server_properties(id: String) -> AppResult<HashMap<String, String>> {
    let dir = servers::folder_for_id(&id)?;
    server_properties::read(&dir)
}

#[tauri::command]
pub fn write_server_properties(id: String, props: HashMap<String, String>) -> AppResult<()> {
    let dir = servers::folder_for_id(&id)?;
    server_properties::write(&dir, &props)
}

#[tauri::command]
pub fn open_server_folder(id: String) -> AppResult<()> {
    servers::open_folder(&id)
}

#[tauri::command]
pub fn get_server_folder_path(id: String) -> AppResult<String> {
    Ok(servers::folder_for_id(&id)?
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
pub async fn list_server_content(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Vec<ServerContentItem>> {
    let prof = servers::profile_by_id(&id)?;
    let mut items = servers::list_content(&id)?;
    let loader = if prof.server_type.starts_with("fabric") {
        "fabric"
    } else if prof.server_type.contains("neoforge") {
        "neoforge"
    } else if prof.server_type.starts_with("forge") {
        "forge"
    } else {
        "paper"
    };
    let (http, _) = state.net_scope();
    let _ = servers::enrich_content_modrinth(&http, &prof.mc_version, loader, &mut items).await;
    Ok(items)
}

#[tauri::command]
pub fn server_whitelist_list(id: String) -> AppResult<Vec<String>> {
    let dir = servers::folder_for_id(&id)?;
    Ok(server_admin::whitelist_names(&dir))
}

#[tauri::command]
pub fn server_whitelist_add(id: String, name: String) -> AppResult<()> {
    let dir = servers::folder_for_id(&id)?;
    if servers::status(&id)?.running {
        servers::send_command(&id, &format!("whitelist add {}", name.trim()))?;
        servers::send_command(&id, "whitelist reload")?;
    } else {
        server_admin::whitelist_add_offline(&dir, &name)?;
    }
    Ok(())
}

#[tauri::command]
pub fn server_whitelist_remove(id: String, name: String) -> AppResult<()> {
    let dir = servers::folder_for_id(&id)?;
    if servers::status(&id)?.running {
        servers::send_command(&id, &format!("whitelist remove {}", name.trim()))?;
        servers::send_command(&id, "whitelist reload")?;
    } else {
        server_admin::whitelist_remove_offline(&dir, &name)?;
    }
    Ok(())
}

#[tauri::command]
pub fn server_op_list(id: String) -> AppResult<Vec<String>> {
    let dir = servers::folder_for_id(&id)?;
    Ok(server_admin::op_names(&dir))
}

#[tauri::command]
pub fn server_op_add(id: String, name: String) -> AppResult<()> {
    let dir = servers::folder_for_id(&id)?;
    if servers::status(&id)?.running {
        servers::send_command(&id, &format!("op {}", name.trim()))?;
    } else {
        server_admin::op_add_offline(&dir, &name)?;
    }
    Ok(())
}

#[tauri::command]
pub fn server_op_remove(id: String, name: String) -> AppResult<()> {
    let dir = servers::folder_for_id(&id)?;
    if servers::status(&id)?.running {
        servers::send_command(&id, &format!("deop {}", name.trim()))?;
    } else {
        server_admin::op_remove_offline(&dir, &name)?;
    }
    Ok(())
}

#[tauri::command]
pub fn server_ban_list(id: String) -> AppResult<Vec<String>> {
    let dir = servers::folder_for_id(&id)?;
    Ok(server_admin::ban_names(&dir))
}

#[tauri::command]
pub fn server_ban_add(id: String, name: String) -> AppResult<()> {
    let dir = servers::folder_for_id(&id)?;
    if servers::status(&id)?.running {
        servers::send_command(&id, &format!("ban {}", name.trim()))?;
    } else {
        server_admin::ban_add_offline(&dir, &name)?;
    }
    Ok(())
}

#[tauri::command]
pub fn server_ban_remove(id: String, name: String) -> AppResult<()> {
    let dir = servers::folder_for_id(&id)?;
    if servers::status(&id)?.running {
        servers::send_command(&id, &format!("pardon {}", name.trim()))?;
    } else {
        server_admin::ban_remove_offline(&dir, &name)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn hangar_search_plugins(
    state: State<'_, AppState>,
    query: String,
) -> AppResult<Vec<server_hangar::HangarPlugin>> {
    let (http, _net) = state.net_scope();
    server_hangar::search(&http, &query).await
}

#[tauri::command]
pub async fn hangar_install_plugin(
    state: State<'_, AppState>,
    id: String,
    owner: String,
    slug: String,
) -> AppResult<String> {
    let prof = servers::profile_by_id(&id)?;
    let dir = servers::folder_for(&prof);
    let is_fabric = prof.server_type.starts_with("fabric");
    let (http, _net) = state.net_scope();
    server_hangar::install_plugin(&http, &dir, &owner, &slug, &prof.mc_version, is_fabric).await
}

#[tauri::command]
pub fn server_backup_worlds(id: String) -> AppResult<server_backups::ServerBackupResult> {
    let dir = servers::folder_for_id(&id)?;
    server_backups::backup_worlds(&id, &dir)
}

#[tauri::command]
pub fn repair_server(id: String) -> AppResult<server_repair::ServerRepairReport> {
    server_repair::repair(&id)
}

#[tauri::command]
pub fn list_server_backups(id: String) -> Vec<server_backups::ServerBackupResult> {
    server_backups::list_backups(&id)
}

#[tauri::command]
pub fn import_server_folder(path: String, name: Option<String>) -> AppResult<ServerProfile> {
    servers::import_folder(&path, name.as_deref())
}

#[tauri::command]
pub async fn pick_server_folder(app: AppHandle) -> AppResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().blocking_pick_folder();
    let Some(file) = picked else {
        return Ok(None);
    };
    let path = file
        .into_path()
        .map_err(|e| AppError::msg(format!("Ruta inválida: {e}")))?;
    Ok(Some(path.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn set_playit_address(id: String, address: String) -> AppResult<()> {
    let addr = address.trim();
    if addr.is_empty() {
        return Err(AppError::msg("Dirección vacía"));
    }
    let dir = servers::folder_for_id(&id)?;
    let meta_path = dir.join("_paragua_srv.json");
    let mut meta: serde_json::Value =
        crate::config::read_json(&meta_path).unwrap_or(serde_json::json!({}));
    if let Some(obj) = meta.as_object_mut() {
        obj.insert(
            "java_address".into(),
            serde_json::Value::String(addr.to_string()),
        );
    }
    crate::config::write_json_atomic(&meta_path, &meta)?;

    #[derive(serde::Deserialize, serde::Serialize, Default)]
    struct ServersFile {
        #[serde(default)]
        servers: Vec<ServerProfile>,
    }
    let path = crate::core::paths::data_dir().join("servers.json");
    let mut file: ServersFile = crate::config::read_json(&path).unwrap_or_default();
    if let Some(p) = file.servers.iter_mut().find(|s| s.id == id) {
        p.playit_address = Some(addr.to_string());
    }
    crate::config::write_json_atomic(&path, &file)?;
    Ok(())
}
