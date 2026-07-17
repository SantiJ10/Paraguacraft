//! Tier A: chequeo pre-lanzamiento, peso de instancia y perfiles de juego.

use tauri::{AppHandle, State};

use crate::core::{game_profiles, instance_weight, pre_launch};
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn run_pre_launch_check(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> AppResult<pre_launch::PreLaunchCheckReport> {
    let (http, _net) = state.net_scope();
    pre_launch::run(&app, &http, &instance_id).await
}

#[tauri::command]
pub fn get_instance_weight(instance_id: String) -> AppResult<instance_weight::InstanceWeight> {
    instance_weight::compute(&instance_id)
}

#[tauri::command]
pub fn list_game_profiles() -> Vec<game_profiles::GameProfile> {
    game_profiles::list()
}

#[tauri::command]
pub fn scan_mod_conflicts(instance_id: String) -> AppResult<Vec<crate::core::mod_conflicts::ModConflict>> {
    crate::core::mod_conflicts::scan(&instance_id)
}

#[tauri::command]
pub async fn launch_game_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<u32> {
    let profile = game_profiles::resolve(&profile_id)?;
    let instance_id = game_profiles::resolve_instance_id(&profile)?;
    crate::commands::launch::launch_instance(
        app,
        state,
        instance_id,
        profile.server_address.clone(),
        Some(profile.compete_mode),
    )
    .await
}
