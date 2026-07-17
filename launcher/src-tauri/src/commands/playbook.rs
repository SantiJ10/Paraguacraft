//! Tier A: chequeo pre-lanzamiento, peso de instancia y perfiles de juego.

use tauri::{AppHandle, State};

use crate::core::{compete_mode, game_profiles, hardware, instance_weight, instances, modern_pvp, pre_launch};
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
    destination_id: Option<String>,
    favorite_id: Option<String>,
) -> AppResult<u32> {
    let dest = destination_id.as_deref().unwrap_or("menu");
    let server = game_profiles::resolve_server(&profile_id, dest, favorite_id.as_deref())?;

    if profile_id == "fabric-iris-chill" {
        let profile = game_profiles::resolve(&profile_id)?;
        let instance_id = game_profiles::resolve_instance_id(&profile)?;
        return crate::commands::launch::launch_instance(
            app,
            state,
            instance_id,
            server,
            Some(false),
        )
        .await;
    }

    if profile_id == "modern-pvp" {
        let instance_id = {
            let (http, _guard) = state.net_scope();
            let id = modern_pvp::ensure_instance(&app, &http, &state).await?;
            modern_pvp::apply_hardware_profile(&id)?;
            let tier = hardware::detect().perfil_sugerido;
            let _ = modern_pvp::apply_launch_properties(&id, &tier)?;
            let _ = modern_pvp::apply_performance_profile(&id, &tier)?;
            if dest == "training" {
                let _ = modern_pvp::apply_training_profile(&id, &tier, true)?;
            }
            let _ = modern_pvp::sync_hud_mods(&app, &http, &id).await?;
            let _ = modern_pvp::sync_instance_content(&app, &http, &id).await?;
            id
        };
        return crate::commands::launch::launch_instance(
            app,
            state,
            instance_id,
            server,
            Some(false),
        )
        .await;
    }

    let profile = game_profiles::resolve(&profile_id)?;
    let instance_id = game_profiles::resolve_instance_id(&profile)?;
    if profile_id == "pvp-compete" {
        let tier = hardware::detect().perfil_sugerido;
        let inst_dir = instances::instance_dir(&instance_id);
        match dest {
            "training" => compete_mode::apply_training_profile(&inst_dir, &tier, true)?,
            "menu" => compete_mode::apply_training_profile(&inst_dir, &tier, false)?,
            _ => {}
        }
    }
    let use_compete = profile_id == "pvp-compete" && matches!(dest, "hypixel" | "favorite");
    crate::commands::launch::launch_instance(
        app,
        state,
        instance_id,
        server,
        Some(use_compete),
    )
    .await
}
