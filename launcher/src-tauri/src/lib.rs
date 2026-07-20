//! Paraguacraft Launcher - backend Rust.
//!
//! Arquitectura modular (cero espagueti):
//!   - `commands/` : handlers IPC finos expuestos a la UI via `invoke`.
//!   - `core/`     : logica de dominio por modulo (hardware, java, accounts, instances...).
//!   - `models/`   : structs compartidos con la UI (serde camelCase).
//!   - `config/`   : persistencia JSON atomica.
//!   - `state.rs`  : estado global (cliente HTTP reutilizable + caches perezosas).
//!
//! Eficiencia (objetivo del proyecto): no se lanzan hilos ni timers en segundo
//! plano; todo el trabajo (auth refresh, escaneos, descargas) es on-demand y
//! corre sobre el runtime tokio que ya provee Tauri. Esto mantiene el launcher
//! cerca de 0% CPU en idle y respeta el presupuesto de procesos para gaming.

mod commands;
mod config;
mod core;
mod error;
mod models;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    config::keys::load_env_files();
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .manage(AppState::default())
        .setup(|app| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.maximize();
            }
            let settings =
                config::read_json::<models::AppSettings>(&core::paths::config_file()).unwrap_or_default();
            if settings.discord_rpc {
                core::extras::discord_rpc::connect(true);
                if let Some(acc) = core::accounts::active_account() {
                    core::extras::discord_rpc::set_launcher_idle(&acc.username);
                }
            }
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(state) = handle.try_state::<AppState>() {
                    let http = state.client();
                    let _ = core::skins::pending_premium::flush_pending(&http).await;
                    state.shutdown_network();
                }
            });
            let _ = core::tray_lite::install(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if core::game_session::is_running() {
                    api.prevent_close();
                    core::tray_lite::hide_to_tray(window.app_handle());
                    return;
                }
                core::servers::stop_all_running();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Hardware
            commands::hardware::get_hardware_info,
            // Settings
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::cleanup::shutdown_background_services,
            commands::performance::optimize_minecraft_options,
            commands::performance::apply_recommended_performance,
            commands::compete::get_resource_budget,
            commands::compete::sync_pvp_config,
            commands::playbook::run_pre_launch_check,
            commands::playbook::get_instance_weight,
            commands::playbook::list_game_profiles,
            commands::playbook::launch_game_profile,
            commands::playbook::scan_mod_conflicts,
            // Extras
            commands::extras::get_extras_status,
            commands::extras::get_extras_panel_data,
            commands::extras::activate_game_mode,
            commands::extras::deactivate_game_mode,
            commands::extras::activate_turbo_mode,
            commands::extras::deactivate_turbo_mode,
            commands::extras::set_java_priority,
            commands::extras::get_cleanup_info,
            commands::extras::run_cleanup,
            commands::extras::sync_discord_rpc,
            commands::extras::set_discord_rpc_screen,
            // Java
            commands::java::detect_javas,
            commands::java::verify_java_path,
            commands::java::java_required_for_mc,
            commands::java::java_info_for_mc,
            commands::java::download_temurin,
            // Accounts
            commands::accounts::get_accounts,
            commands::accounts::set_active_account,
            commands::accounts::add_offline_account,
            commands::accounts::remove_account,
            commands::accounts::ms_login_url,
            commands::accounts::ms_login_complete_code,
            commands::accounts::ms_login_start,
            commands::accounts::ms_login_poll,
            commands::accounts::ensure_account_token,
            // Instances
            commands::instances::scan_instances,
            commands::instances::list_instances,
            commands::instances::create_instance,
            commands::instances::rename_instance,
            commands::instances::get_instance_icon_path,
            commands::instances::import_instance_icon,
            commands::instances::pick_and_import_instance_icon,
            commands::instances::set_instance_ram,
            commands::instances::duplicate_instance,
            commands::instances::delete_instance,
            commands::instances::import_instance,
            commands::instances::create_backup,
            commands::instances::list_backups,
            commands::instances::restore_backup,
            commands::instances::delete_backup,
            commands::instances::get_instance_meta,
            commands::instances::set_instance_config,
            commands::instances::set_instance_auto_managed,
            commands::instances::list_instance_content,
            commands::instances::toggle_instance_content,
            commands::instances::open_instance_folder,
            commands::instances::get_instance_folder_path,
            commands::instances::set_instance_loader,
            commands::instances::reinstall_instance_loader,
            commands::instances::repair_instance,
            commands::instances::get_instance_log,
            commands::instances::remove_instance_content,
            commands::instances::reveal_instance_content,
            commands::instances::pick_and_add_instance_content,
            commands::instances::pick_and_export_instance,
            // Versiones (Fase 3)
            commands::versions::list_minecraft_versions,
            commands::versions::install_minecraft_version,
            // Loaders (Fase 3)
            commands::loaders::list_loaders,
            commands::loaders::list_loader_versions,
            commands::loaders::install_loader,
            commands::loaders::install_fabric_iris_bundle,
            commands::loaders::install_pvp_bundle,
            commands::loaders::install_pvp_modern_bundle,
            commands::loaders::get_pvp_client_status,
            commands::loaders::get_pvp_modern_client_status,
            // Tienda (Fase 3)
            commands::store::store_search,
            commands::store::store_list_versions,
            commands::store::store_list_project_versions,
            commands::store::store_install,
            commands::store::store_install_version,
            commands::store::import_mrpack,
            commands::store::import_mrpack_version,
            commands::store::import_mrpack_version_to_server,
            commands::store::import_cfpack,
            commands::store::import_cfpack_version,
            commands::store::import_cfpack_version_to_server,
            commands::store::pick_and_import_cfpack_zip,
            commands::store::list_instance_worlds,
            commands::store::list_server_worlds,
            commands::store::update_instance_content,
            // Lanzamiento (Fase 3)
            commands::launch::launch_instance,
            commands::launch::sync_overlay_music,
            commands::favorites::list_favorite_servers,
            commands::favorites::add_favorite_server,
            commands::favorites::remove_favorite_server,
            commands::favorites::suggest_favorite_profile,
            commands::favorites::join_favorite_server,
            commands::favorites::add_favorite_from_server,
            commands::favorites::favorite_bedrock_address,
            commands::favorites::join_favorite_bedrock,
            commands::bedrock::get_bedrock_status,
            commands::bedrock::launch_bedrock,
            // Fase 4
            commands::diagnostics::diagnose_instance,
            commands::diagnostics::ai_assist,
            commands::diagnostics::ai_status,
            commands::diagnostics::save_groq_api_key,
            commands::servers::list_servers,
            commands::servers::create_server,
            commands::servers::update_server,
            commands::servers::delete_server,
            commands::servers::server_status,
            commands::servers::start_server,
            commands::servers::stop_server,
            commands::servers::stop_server_force,
            commands::servers::start_playit,
            commands::servers::stop_playit,
            commands::servers::mark_playit_claimed,
            commands::servers::prepare_server_jar,
            commands::servers::server_plugin_count,
            commands::servers::get_server_log,
            commands::servers::export_server_log,
            commands::servers::send_server_command,
            commands::servers::read_server_properties,
            commands::servers::write_server_properties,
            commands::servers::open_server_folder,
            commands::servers::get_server_folder_path,
            commands::servers::list_server_content,
            commands::servers::server_whitelist_list,
            commands::servers::server_whitelist_add,
            commands::servers::server_whitelist_remove,
            commands::servers::server_op_list,
            commands::servers::server_op_add,
            commands::servers::server_op_remove,
            commands::servers::server_ban_list,
            commands::servers::server_ban_add,
            commands::servers::server_ban_remove,
            commands::servers::hangar_search_plugins,
            commands::servers::hangar_install_plugin,
            commands::servers::server_backup_worlds,
            commands::servers::repair_server,
            commands::servers::list_server_backups,
            commands::servers::import_server_folder,
            commands::servers::pick_server_folder,
            commands::servers::set_playit_address,
            commands::skins::get_active_skin_local,
            commands::skins::get_active_skin,
            commands::skins::get_skin_for_account,
            commands::skins::get_offline_skin,
            commands::skins::has_offline_skin_file,
            commands::skins::get_offline_skin_path,
            commands::skins::pick_and_apply_offline_skin,
            commands::skins::apply_offline_skin_path,
            commands::skins::lookup_skin_player,
            commands::skins::skin_catalog_search,
            commands::skins::skin_preview_image,
            commands::skins::apply_skin_from_username,
            commands::skins::apply_skin_from_url,
            commands::skins::download_skin_file,
            commands::skins::get_skin_history,
            commands::skins::clear_skin_history,
            commands::skins::can_upload_premium_skin,
            commands::skins::has_pending_premium_skin,
            commands::skins::sync_pending_premium_skin,
            commands::skins::pick_skin_file_for_preview,
            commands::skins::apply_skin_file_with_variant,
            // Spotify
            commands::spotify::spotify_status,
            commands::spotify::spotify_save_credentials,
            commands::spotify::spotify_validate_app,
            commands::spotify::spotify_setup_info,
            commands::spotify::spotify_auth_start,
            commands::spotify::spotify_poll_auth,
            commands::spotify::spotify_oauth_hint,
            commands::spotify::spotify_open_dashboard,
            commands::spotify::spotify_connect,
            commands::spotify::spotify_try_autoconnect,
            commands::spotify::spotify_disconnect,
            commands::spotify::spotify_now_playing,
            commands::spotify::spotify_control,
            commands::spotify::spotify_shuffle,
            commands::spotify::spotify_repeat,
            commands::updater::check_launcher_update,
            commands::updater::download_and_install_launcher_update,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Paraguacraft Launcher");
}
