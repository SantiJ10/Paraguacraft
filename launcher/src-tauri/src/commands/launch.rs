//! Comando de lanzamiento + suspension total (Regla 3).

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, State};

use crate::config;
use crate::core::launch::{self, AuthCtx, JvmCtx};
use crate::core::paths;
use crate::core::{accounts, instances, loaders, modern_pvp, versions};
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use crate::state::AppState;

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

async fn resolve_auth(http: &reqwest::Client) -> AppResult<AuthCtx> {
    let account = accounts::active_account()
        .ok_or_else(|| AppError::msg("No hay cuenta activa. Agrega una en Ajustes."))?;
    if account.kind == "microsoft" {
        let tok = accounts::ensure_valid_token(http, &account.id).await?;
        Ok(AuthCtx {
            username: account.username,
            uuid: account.uuid,
            access_token: tok.mc_access_token,
            user_type: "msa".into(),
        })
    } else {
        Ok(AuthCtx {
            username: account.username,
            uuid: account.uuid,
            access_token: "0".into(),
            user_type: "legacy".into(),
        })
    }
}

async fn resolve_launch_id(
    app: &AppHandle,
    http: &reqwest::Client,
    mc: &str,
    loader: &str,
    loader_version: &str,
    version_hint: Option<&str>,
    meta: &mut instances::InstanceMeta,
    instance_id: &str,
) -> AppResult<String> {
    let loader = loaders::normalize(loader);
    // Siempre asegurar vanilla base (idempotente; corrige libraries/ incompletas).
    versions::install_vanilla(app, http, mc).await?;
    if let Some(v) = meta.version_id.clone() {
        let profile_ok = versions::read_local_json(&v).is_some()
            && loaders::version_id_matches_loader(&loader, &v, mc);
        if profile_ok {
            if loader == "vanilla" && !versions::jar_path(&v).is_file() {
                versions::install_vanilla(app, http, mc).await?;
            } else {
                return Ok(v);
            }
        } else {
            meta.version_id = None;
        }
    }
    if loader != "vanilla" {
        if let Some(vid) = loaders::find_version_id_for_loader(mc, &loader) {
            if versions::read_local_json(&vid).is_some() {
                meta.version_id = Some(vid.clone());
                if meta.loader_version.is_empty() {
                    if let Some(lv) =
                        loaders::loader_version_from_version_id(&loader, &vid, mc)
                    {
                        meta.loader_version = lv;
                    }
                }
                if !instance_id.starts_with("ext::") {
                    let _ = instances::write_meta(instance_id, meta);
                }
                return Ok(vid);
            }
        }
    }
    if loader == "vanilla" {
        if let Some(hint) = version_hint.filter(|h| versions::read_local_json(h).is_some()) {
            meta.version_id = Some(hint.to_string());
            if !instance_id.starts_with("ext::") {
                let _ = instances::write_meta(instance_id, meta);
            }
            return Ok(hint.to_string());
        }
        if versions::read_local_json(mc).is_some() {
            meta.version_id = Some(mc.to_string());
            if !instance_id.starts_with("ext::") {
                let _ = instances::write_meta(instance_id, meta);
            }
            return Ok(mc.to_string());
        }
    }
    let id = loaders::install_loader(app, http, mc, &loader, loader_version).await?;
    meta.version_id = Some(id.clone());
    if !instance_id.starts_with("ext::") {
        let _ = instances::write_meta(instance_id, meta);
    }
    Ok(id)
}

async fn spawn_for_instance(
    app: &AppHandle,
    state: &AppState,
    instance_id: &str,
    meta: &mut instances::InstanceMeta,
    game_dir: PathBuf,
    auth: AuthCtx,
    launch_id: String,
    mc: String,
    loader: String,
    settings: &AppSettings,
    server_address: Option<String>,
    compete: Option<crate::core::compete_mode::CompeteLaunchPlan>,
) -> AppResult<u32> {
    // Motor de optimizacion dinamica: limpieza + perfil de graficos diferenciado por
    // gama de PC (Baja/Media/Alta) y por loader (1.8.9 Forge+OptiFine, 1.21.11
    // Fabric+Sodium+Iris, o generico), en vez del preset fijo que se aplicaba antes.
    let hw_tier = crate::core::hardware::detect().perfil_sugerido;
    crate::core::launch::optimizer::apply_pre_launch(&game_dir, &loader, &hw_tier, settings);

    if crate::core::branding::should_apply(&loader) {
        let _ = crate::core::branding::inject_logos(
            &game_dir,
            &mc,
            settings.optimize_graphics,
        );
    }

    if loaders::normalize(&loader) == "paraguacraft-pvp"
        || loaders::normalize(&loader) == "paraguacraft-pvp-modern"
    {
        let http = state.client();
        let _ = crate::core::pvp_packs::prepare_launch(
            app,
            &http,
            &game_dir,
            &loader,
            &mc,
        )
        .await;
    }

    if crate::core::skins::offline::has_global_skin() {
        let _ = crate::core::skins::offline::ensure_for_launch(&game_dir, &mc);
    }

    if settings.backup_auto_hours > 0 && !instance_id.starts_with("ext::") {
        let _ = crate::core::extras::maintenance::auto_backup_worlds(instance_id);
    }

    let ram = if let Some(ref plan) = compete {
        plan.ram_mb
    } else if meta.ram_mb > 0 {
        meta.ram_mb
    } else {
        settings.ram_mb
    };
    let gc = meta
        .gc
        .clone()
        .unwrap_or_else(|| settings.gc_type.clone());
    let extra_args: Vec<String> = meta
        .jvm_args
        .clone()
        .unwrap_or_default()
        .split_whitespace()
        .map(String::from)
        .collect();
    let java_path = crate::core::java::resolve::ensure_launch_java(
        app,
        state,
        &mc,
        &launch_id,
        meta.java_path.as_deref(),
        settings.java_path.as_deref(),
    )
    .await?;
    let java_major = crate::core::java::verify::verify(&java_path, "launch")
        .map(|j| j.version_major)
        .unwrap_or_else(|| crate::core::java::required_for_mc(&mc));
    let hw = crate::core::hardware::detect();
    let jvm = JvmCtx {
        ram_mb: ram,
        gc,
        extra_args,
        java_path,
        java_major,
        mc_version: mc.clone(),
        loader: loader.clone(),
        system_ram_gb: hw.ram_gb,
    };

    let resolution = if settings.papa_mode {
        Some((800u32, 600u32))
    } else {
        None
    };

    let (args, java) = launch::build_command(&launch_id, &game_dir, &auth, &jvm, resolution)?;
    let mut args = args;
    if let Some(addr) = server_address.as_deref().filter(|s| !s.trim().is_empty()) {
        launch::append_server_join(&mut args, addr.trim());
    }
    let has_pvp_mod = launch::has_paraguacraft_pvp_mod(&game_dir);
    let has_pg_rpc = settings.discord_rpc && has_pvp_mod;
    let overlay_ipc = compete
        .as_ref()
        .map(|p| p.overlay_ipc)
        .unwrap_or_else(|| crate::core::compete_mode::overlay_ipc_needed(&game_dir));
    let ipc_path_owned = crate::core::overlay_ipc::ipc_path().to_string_lossy().into_owned();
    let mut launch_env_owned: Vec<(String, String)> = Vec::new();
    if has_pg_rpc {
        launch_env_owned.push(("PARAGUACRAFT_LAUNCHER_RPC".into(), "1".into()));
        launch_env_owned.push(("PARAGUACRAFT_RPC_USER".into(), auth.username.clone()));
        launch_env_owned.push(("PARAGUACRAFT_RPC_MC".into(), mc.clone()));
        launch_env_owned.push(("PARAGUACRAFT_RPC_LOADER".into(), loaders::display_label(&loader)));
    }
    if has_pvp_mod && overlay_ipc {
        launch_env_owned.push(("PARAGUACRAFT_OVERLAY_IPC".into(), ipc_path_owned));
    }
    let launch_env: Vec<(&str, &str)> = launch_env_owned
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let child = launch::spawn_game(&java, &args, &game_dir, &launch_env, java_major)?;
    let pid = child.id();

    let java_priority = compete
        .as_ref()
        .map(|p| p.java_priority.as_str())
        .unwrap_or_else(|| {
            if settings.java_priority.is_empty() {
                "high"
            } else {
                &settings.java_priority
            }
        });
    let _ = crate::core::extras::java_priority::set_level(java_priority);

    if settings.discord_rpc {
        if has_pg_rpc {
            crate::core::extras::discord_rpc::disconnect();
        } else {
            crate::core::extras::discord_rpc::set_playing(
                &auth.username,
                &mc,
                &loader,
                settings.discord_rpc_version,
                settings.discord_rpc_time,
            );
        }
    }

    launch::emit_started(app, instance_id, pid);
    crate::core::game_session::set_running(true);
    let close_on_launch = compete
        .as_ref()
        .map(|p| p.close_on_launch)
        .unwrap_or(settings.close_on_launch);
    launch::watch_exit(
        app.clone(),
        instance_id.to_string(),
        child,
        mc.clone(),
        auth.username.clone(),
        loader.clone(),
        game_dir.clone(),
        server_address.clone(),
        settings.clone(),
        has_pg_rpc,
        overlay_ipc,
        compete.is_some(),
        compete.is_some(),
    );

    state.shutdown_network();
    *state.java_cache.lock().unwrap() = None;

    launch::apply_launch_window(app, close_on_launch, compete.is_some());

    if !instance_id.starts_with("ext::") {
        meta.last_played = Some(now_secs().to_string());
        let _ = instances::write_meta(instance_id, meta);
    }

    Ok(pid)
}

async fn launch_external(
    app: &AppHandle,
    state: &State<'_, AppState>,
    instance_id: &str,
) -> AppResult<u32> {
    let mut meta = instances::resolve_external_meta(instance_id)
        .ok_or_else(|| AppError::msg("Instancia externa no encontrada"))?;
    let game_dir = instances::game_dir_for(instance_id)
        .ok_or_else(|| AppError::msg("Sin carpeta de juego para esta instancia"))?;
    let mc = meta.mc_version.clone();
    let loader = loaders::normalize(&meta.loader);
    let version_hint = instance_id.rsplit("::").next();

    let settings = config::read_json::<AppSettings>(&paths::config_file()).unwrap_or_default();

    let loader_version = meta.loader_version.clone();
    let (auth, launch_id) = {
        let (http, _net) = state.net_scope();
        let launch_id = resolve_launch_id(
            app,
            &http,
            &mc,
            &loader,
            &loader_version,
            version_hint,
            &mut meta,
            instance_id,
        )
        .await?;
        let merged = launch::load_merged(&launch_id)?;
        versions::ensure_merged_libraries(
            app,
            &http,
            &merged,
            &format!("Dependencias {mc}"),
        )
        .await?;
        let auth = resolve_auth(&http).await?;
        (auth, launch_id)
    };

    spawn_for_instance(
        app,
        state,
        instance_id,
        &mut meta,
        game_dir,
        auth,
        launch_id,
        mc,
        loader,
        &settings,
        None,
        None,
    )
    .await
}

/// Lanza la instancia. Asegura instalacion (vanilla+loader), valida el token,
/// arma el comando y lanza `javaw` sin consola; luego libera red y caches.
/// Devuelve el PID del juego.
#[tauri::command]
pub async fn launch_instance(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    server_address: Option<String>,
    compete_mode: Option<bool>,
) -> AppResult<u32> {
    if instance_id.starts_with("ext::") {
        return launch_external(&app, &state, &instance_id).await;
    }

    let mut meta = instances::ensure_meta(&instance_id)?;
    let mc = meta.mc_version.clone();
    let loader = loaders::normalize(&meta.loader);

    let settings = config::read_json::<AppSettings>(&paths::config_file()).unwrap_or_default();
    let use_compete = compete_mode.unwrap_or(false);
    let inst_dir = instances::instance_dir(&instance_id);

    let compete_plan = if use_compete {
        Some(crate::core::compete_mode::apply_pre_launch(
            &inst_dir,
            &loader,
            &meta,
            settings.compete_turbo,
        )?)
    } else {
        None
    };

    let loader_version = meta.loader_version.clone();
    let (auth, launch_id) = {
        let (http, _net) = state.net_scope();

        let launch_id = resolve_launch_id(
            &app,
            &http,
            &mc,
            &loader,
            &loader_version,
            None,
            &mut meta,
            &instance_id,
        )
        .await?;

        let merged = launch::load_merged(&launch_id)?;
        versions::ensure_merged_libraries(
            &app,
            &http,
            &merged,
            &format!("Dependencias {mc}"),
        )
        .await?;

        if loader == "fabric-iris" {
            let inst_dir = instances::instance_dir(&instance_id);
            loaders::fabric_iris::install_bundle(&app, &http, &mc, &inst_dir).await?;
        }
        if loader == "paraguacraft-pvp-modern" {
            let inst_dir = instances::instance_dir(&instance_id);
            loaders::pvp_modern::sync_instance(&app, &http, &mc, &inst_dir).await?;
            let _ = modern_pvp::sync_hud_mods(&app, &http, &instance_id).await;
            let tier = crate::core::hardware::detect().perfil_sugerido;
            let _ = modern_pvp::ensure_launch_defaults(&instance_id, &tier);
            let _ = modern_pvp::sync_instance_content(&app, &http, &instance_id).await;
        }
        if loader == "paraguacraft-pvp" {
            loaders::pvp::install_bundle_for_launch(
                &app,
                &http,
                &inst_dir,
                &instance_id,
                use_compete,
            )
            .await?;
        }

        let auth = resolve_auth(&http).await?;
        (auth, launch_id)
    };

    let game_dir = instances::instance_dir(&instance_id);
    spawn_for_instance(
        &app,
        &state,
        &instance_id,
        &mut meta,
        game_dir,
        auth,
        launch_id,
        mc,
        loader,
        &settings,
        server_address,
        compete_plan,
    )
    .await
}

/// Sincroniza titulo/artista de musica al HUD in-game (IPC overlay).
#[tauri::command]
pub fn sync_overlay_music(playing: bool, title: String, artist: String, image_url: String) {
    crate::core::overlay_ipc::set_music(playing, &title, &artist, &image_url);
}
