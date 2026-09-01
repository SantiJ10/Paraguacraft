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

/// Escribe `playStyle` en `paraguacraft_v2.properties` (PvP 1.8.9).
fn write_legacy_pvp_play_style(instance_id: &str, play_style: &str) -> AppResult<()> {
    let path = instances::instance_dir(instance_id).join("paraguacraft_v2.properties");
    let mut props: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if path.is_file() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    props.insert(k.trim().into(), v.trim().into());
                }
            }
        }
    }
    props.insert("playStyle".into(), play_style.into());
    props.insert("oldAnimations".into(), "true".into());
    props.insert("boostFps".into(), "true".into());
    let mut keys: Vec<_> = props.keys().cloned().collect();
    keys.sort();
    let body: Vec<String> = keys
        .into_iter()
        .map(|k| format!("{k}={}", props[&k]))
        .collect();
    std::fs::write(path, format!("{}\n", body.join("\n")))?;
    Ok(())
}

async fn resolve_auth(http: &reqwest::Client, offline: bool) -> AppResult<AuthCtx> {
    let account = accounts::active_account()
        .ok_or_else(|| AppError::msg("No hay cuenta activa. Agrega una en Ajustes."))?;
    if account.kind == "microsoft" {
        if offline {
            let tok = accounts::cached_token(&account.id).ok_or_else(|| {
                AppError::msg(
                    "Sin conexión: no hay una sesión Microsoft guardada. Conectate una vez para iniciar sesión.",
                )
            })?;
            return Ok(AuthCtx {
                username: account.username,
                uuid: account.uuid,
                access_token: tok.mc_access_token,
                user_type: "msa".into(),
            });
        }
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

fn local_profile_ready(version_id: &str) -> bool {
    versions::read_local_json(version_id).is_some()
}

const OFFLINE_MISSING: &str =
    "Sin conexión a internet. Esta instancia no está descargada por completo; conectate una vez para instalarla y después podés jugar offline.";

async fn resolve_launch_id(
    app: &AppHandle,
    http: &reqwest::Client,
    mc: &str,
    loader: &str,
    loader_version: &str,
    version_hint: Option<&str>,
    meta: &mut instances::InstanceMeta,
    instance_id: &str,
    offline: bool,
) -> AppResult<String> {
    let loader = loaders::normalize(loader);
    if !offline {
        // Siempre asegurar vanilla base (idempotente; corrige libraries/ incompletas).
        versions::install_vanilla(app, http, mc).await?;
    }
    if let Some(v) = meta.version_id.clone() {
        let profile_ok = local_profile_ready(&v) && loaders::version_id_matches_loader(&loader, &v, mc);
        if profile_ok {
            if loader == "vanilla" && !versions::jar_path(&v).is_file() {
                if offline {
                    return Err(AppError::msg(OFFLINE_MISSING));
                }
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
            if local_profile_ready(&vid) {
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
        if let Some(hint) = version_hint.filter(|h| local_profile_ready(h)) {
            meta.version_id = Some(hint.to_string());
            if !instance_id.starts_with("ext::") {
                let _ = instances::write_meta(instance_id, meta);
            }
            return Ok(hint.to_string());
        }
        if local_profile_ready(mc) {
            meta.version_id = Some(mc.to_string());
            if !instance_id.starts_with("ext::") {
                let _ = instances::write_meta(instance_id, meta);
            }
            return Ok(mc.to_string());
        }
    }
    if offline {
        return Err(AppError::msg(OFFLINE_MISSING));
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
    offline: bool,
) -> AppResult<u32> {
    // Motor de optimizacion dinamica: limpieza + perfil de graficos diferenciado por
    // gama de PC (Baja/Media/Alta) y por loader (1.8.9 Forge+OptiFine, 1.21.11
    // Fabric+Sodium+Iris, o generico), en vez del preset fijo que se aplicaba antes.
    let hw_tier = crate::core::performance::resolve_tier(
        settings,
        meta.performance_tier.as_deref(),
    );
    crate::core::launch::optimizer::apply_pre_launch(
        &game_dir,
        &loader,
        &hw_tier,
        &mc,
        settings,
    );
    if settings.papa_mode {
        let _ = crate::core::performance::apply_papa_profile(&game_dir);
        crate::core::launch::optimizer::inject_papa_mods(
            app,
            state,
            &game_dir,
            &loader,
            &mc,
            offline,
        )
        .await;
    }

    if crate::core::branding::should_apply(&loader) {
        let _ = crate::core::branding::inject_logos(
            &game_dir,
            &mc,
            settings.optimize_graphics,
        );
    }

    let loader_norm = loaders::normalize(&loader);
    // Skin offline → LocalSkin por nick + purga texturas Steve/Alex del brand pack
    // (antes se inyectaba en defaults y se veía LA MISMA skin en todos los jugadores).
    let _ = crate::core::skins::offline::ensure_for_launch(&game_dir, &mc);

    if loader_norm == "paraguacraft-pvp" || loader_norm == "paraguacraft-pvp-modern" {
        // Marker one-shot desde perfiles → Practica. Sin marker: limpia sticky
        // (Home/Play/Hypixel) para no reabrir el mundo flat.
        let training_marker = game_dir.join(".paraguacraft_launch_training");
        let launch_training = training_marker.is_file();
        if launch_training {
            let _ = std::fs::remove_file(&training_marker);
        } else {
            if loader_norm == "paraguacraft-pvp" {
                let _ = crate::core::compete_mode::clear_training_flags_189(&game_dir);
            } else {
                let _ = crate::core::compete_mode::clear_training_flags_modern(&game_dir);
            }
        }

        if !offline {
            let http = state.client();
            if let Err(e) = crate::core::pvp_packs::prepare_launch(
                app,
                &http,
                &game_dir,
                &loader,
                &mc,
            )
            .await
            {
                eprintln!("[paraguacraft] resource pack prepare_launch: {e}");
            }
        } else if let Err(e) = crate::core::pvp_packs::enable_local_only(&game_dir, &loader, &mc) {
            eprintln!("[paraguacraft] resource pack local: {e}");
        }
    }

    // Offline / no-premium: CustomSkinLoader + Ely.by para skins multiplayer.
    if !offline {
        let http = state.client();
        let _ = crate::core::skins::csl::ensure_for_offline_launch(
            app,
            &http,
            &game_dir,
            &mc,
            &loader,
            &auth.user_type,
        )
        .await;
    } else {
        crate::core::skins::csl::ensure_local_config(&game_dir, &loader, &auth.user_type);
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
    // Respeta la RAM elegida por el usuario; solo baja el tope si supera lo seguro del sistema
    // (deja ~1.5 GB para Windows/launcher) para evitar que el SO mate Java.
    let hw = crate::core::hardware::detect();
    let ram = {
        let total_mb = ((hw.ram_gb * 1024.0).round() as u32).max(2048);
        let max_safe = total_mb.saturating_sub(1536).max(1024);
        let ram = ram.max(1024).min(max_safe);
        if settings.papa_mode {
            ram.min(2048)
        } else {
            ram
        }
    };
    let gc = if settings.papa_mode {
        "G1".into()
    } else {
        meta.gc
            .clone()
            .unwrap_or_else(|| settings.gc_type.clone())
    };
    let extra_args: Vec<String> = {
        let mut out = Vec::new();
        for part in settings.global_jvm_args.split_whitespace() {
            out.push(part.to_string());
        }
        if let Some(inst) = meta.jvm_args.as_deref() {
            for part in inst.split_whitespace() {
                out.push(part.to_string());
            }
        }
        if settings.papa_mode {
            for flag in [
                "-XX:MaxGCPauseMillis=100",
                "-XX:ParallelGCThreads=2",
                "-XX:ConcGCThreads=1",
            ] {
                if !out.iter().any(|a| a.starts_with(flag.split('=').next().unwrap_or(flag))) {
                    out.push(flag.into());
                }
            }
        }
        out
    };
    let show_console = meta.show_game_console.unwrap_or(settings.show_game_console);
    let java_path = crate::core::java::resolve::ensure_launch_java(
        app,
        state,
        &mc,
        &launch_id,
        meta.java_path.as_deref(),
        settings.java_path.as_deref(),
        !offline,
    )
    .await?;
    // Consola OS: `java.exe` (con ventana). Por defecto `javaw` sin consola.
    let java_path = if show_console {
        PathBuf::from(crate::core::java::resolve::prefer_java_exe(
            &java_path.to_string_lossy(),
        ))
    } else {
        java_path
    };
    let java_major = crate::core::java::verify::verify(&java_path, "launch")
        .map(|j| j.version_major)
        .unwrap_or_else(|| crate::core::java::required_for_mc(&mc));
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
    } else if settings.game_width > 0 && settings.game_height > 0 {
        Some((settings.game_width, settings.game_height))
    } else {
        None
    };

    let (args, java) = launch::build_command(&launch_id, &game_dir, &auth, &jvm, resolution)?;
    let mut args = args;
    if let Some(addr) = server_address.as_deref().filter(|s| !s.trim().is_empty()) {
        launch::append_server_join(&mut args, addr.trim());
    }
    let has_pvp_mod = launch::has_paraguacraft_pvp_mod(&game_dir);
    let overlay_ipc = compete
        .as_ref()
        .map(|p| p.overlay_ipc)
        .unwrap_or_else(|| crate::core::compete_mode::overlay_ipc_needed(&game_dir));
    let ipc_path_owned = crate::core::overlay_ipc::ipc_path().to_string_lossy().into_owned();
    let mut launch_env_owned: Vec<(String, String)> = Vec::new();
    if has_pvp_mod {
        // El launcher mantiene el RPC; evita que el mod compita por el pipe de Discord.
        launch_env_owned.push(("PARAGUACRAFT_DISABLE_RPC".into(), "1".into()));
    }
    if has_pvp_mod && overlay_ipc {
        launch_env_owned.push(("PARAGUACRAFT_OVERLAY_IPC".into(), ipc_path_owned));
    }
    // Compat GPU (Mesa): requiere drivers Mesa en PATH; útil en PCs con GPU problemática.
    match settings.gpu_compat_mode.as_str() {
        "mesa-d3d12" => {
            launch_env_owned.push(("GALLIUM_DRIVER".into(), "d3d12".into()));
        }
        "mesa-llvmpipe" => {
            launch_env_owned.push(("GALLIUM_DRIVER".into(), "llvmpipe".into()));
            launch_env_owned.push(("LIBGL_ALWAYS_SOFTWARE".into(), "1".into()));
        }
        "mesa-zink" => {
            launch_env_owned.push(("GALLIUM_DRIVER".into(), "zink".into()));
        }
        _ => {}
    }
    let launch_env: Vec<(&str, &str)> = launch_env_owned
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let child = launch::spawn_game(
        &java,
        &args,
        &game_dir,
        &launch_env,
        java_major,
        show_console,
    )?;
    let pid = child.id();
    crate::core::extras::discord_rpc::bind_game_pid(pid);

    crate::core::game_session::set_last_launch_instance(instance_id);
    crate::core::client_console::begin_session(instance_id, &game_dir);

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
        crate::core::extras::discord_rpc::set_playing(
            &auth.username,
            &mc,
            &loader,
            &meta.name,
            settings.discord_rpc_version,
            settings.discord_rpc_time,
        );
    }

    launch::emit_started(app, instance_id, pid);
    crate::core::game_session::set_running(true);
    let close_on_launch = compete
        .as_ref()
        .map(|p| p.close_on_launch)
        .unwrap_or(settings.close_on_launch);
    // Con consola ON no matamos el proceso del launcher (soft → bandeja / restaurar al cerrar MC).
    let soft_close = compete.is_some() || show_console;
    launch::watch_exit(
        app.clone(),
        instance_id.to_string(),
        child,
        mc.clone(),
        auth.username.clone(),
        loader.clone(),
        meta.name.clone(),
        game_dir.clone(),
        server_address.clone(),
        settings.clone(),
        false,
        overlay_ipc,
        compete.is_some(),
        soft_close,
    );

    state.shutdown_network();
    *state.java_cache.lock().unwrap() = None;

    launch::apply_launch_window(app, close_on_launch, soft_close);

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
    let (auth, launch_id, offline) = {
        let (http, _net) = state.net_scope();
        let mut offline = !crate::core::net::is_online(&http).await;
        launch::emit_status(
            app,
            "preparing",
            if offline {
                "Sin conexión — usando archivos locales…"
            } else {
                "Resolviendo loader / perfil…"
            },
        );
        let prepared = async {
            let launch_id = resolve_launch_id(
                app,
                &http,
                &mc,
                &loader,
                &loader_version,
                version_hint,
                &mut meta,
                instance_id,
                offline,
            )
            .await?;
            if !offline {
                let merged = launch::load_merged(&launch_id)?;
                versions::ensure_merged_libraries(
                    app,
                    &http,
                    &merged,
                    &format!("Dependencias {mc}"),
                )
                .await?;
            }
            let auth = resolve_auth(&http, offline).await?;
            Ok::<_, AppError>((auth, launch_id))
        }
        .await;

        match prepared {
            Ok(pair) => (pair.0, pair.1, offline),
            Err(e) if !offline && e.is_connectivity() => {
                offline = true;
                launch::emit_status(app, "preparing", "Sin conexión — usando archivos locales…");
                let launch_id = resolve_launch_id(
                    app,
                    &http,
                    &mc,
                    &loader,
                    &loader_version,
                    version_hint,
                    &mut meta,
                    instance_id,
                    true,
                )
                .await?;
                let auth = resolve_auth(&http, true).await?;
                (auth, launch_id, true)
            }
            Err(e) => return Err(e),
        }
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
        offline,
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

    launch::emit_status(
        &app,
        "preparing",
        &format!("Preparando {}…", meta.name),
    );

    let compete_plan = if use_compete {
        launch::emit_status(&app, "preparing", "Modo Competir — aplicando perfil…");
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
    let (auth, launch_id, offline) = {
        let (http, _net) = state.net_scope();
        let mut offline = !crate::core::net::is_online(&http).await;

        launch::emit_status(
            &app,
            "preparing",
            if offline {
                "Sin conexión — usando archivos locales…"
            } else {
                "Resolviendo loader / perfil…"
            },
        );

        let prepared = async {
            let launch_id = resolve_launch_id(
                &app,
                &http,
                &mc,
                &loader,
                &loader_version,
                None,
                &mut meta,
                &instance_id,
                offline,
            )
            .await?;

            if !offline {
                launch::emit_status(&app, "downloading", &format!("Verificando librerías de {mc}…"));
                let merged = launch::load_merged(&launch_id)?;
                versions::ensure_merged_libraries(
                    &app,
                    &http,
                    &merged,
                    &format!("Dependencias {mc}"),
                )
                .await?;

                if loader == "fabric-iris" {
                    launch::emit_status(&app, "downloading", "Sincronizando Fabric + Iris…");
                    let inst_dir = instances::instance_dir(&instance_id);
                    loaders::fabric_iris::install_bundle(&app, &http, &mc, &inst_dir).await?;
                }
                if loader == "paraguacraft-optimized" || loader == "paraguacraft-optimized-neoforge" {
                    launch::emit_status(&app, "downloading", "Sincronizando pack Optimized…");
                    let inst_dir = instances::instance_dir(&instance_id);
                    loaders::optimized::install_bundle_for_launch(&app, &http, &mc, &loader, &inst_dir)
                        .await?;
                }
                if loader == "paraguacraft-pvp-modern" {
                    launch::emit_status(&app, "downloading", "Sincronizando cliente PvP…");
                    modern_pvp::sync_instance_bundles(&app, &http, &instance_id).await?;
                    let settings_tier = crate::core::performance::resolve_tier(
                        &settings,
                        meta.performance_tier.as_deref(),
                    );
                    let play_style = settings.pvp_play_style.as_str();
                    let _ = modern_pvp::ensure_launch_defaults(&instance_id, &settings_tier, play_style);
                    let _ = modern_pvp::sync_instance_content(&app, &http, &instance_id).await;
                }
                if loader == "paraguacraft-pvp" {
                    launch::emit_status(&app, "downloading", "Sincronizando cliente PvP 1.8.9…");
                    loaders::pvp::install_bundle_for_launch(
                        &app,
                        &http,
                        &inst_dir,
                        &instance_id,
                        use_compete,
                    )
                    .await?;
                    let _ = write_legacy_pvp_play_style(&instance_id, settings.pvp_play_style.as_str());
                }
                launch::emit_status(&app, "preparing", "Validando cuenta…");
            } else if loader == "paraguacraft-pvp" {
                let _ = write_legacy_pvp_play_style(&instance_id, settings.pvp_play_style.as_str());
            } else if loader == "paraguacraft-pvp-modern" {
                let settings_tier = crate::core::performance::resolve_tier(
                    &settings,
                    meta.performance_tier.as_deref(),
                );
                let play_style = settings.pvp_play_style.as_str();
                let _ = modern_pvp::ensure_launch_defaults(&instance_id, &settings_tier, play_style);
            }

            let auth = resolve_auth(&http, offline).await?;
            Ok::<_, AppError>((auth, launch_id))
        }
        .await;

        match prepared {
            Ok(pair) => (pair.0, pair.1, offline),
            Err(e) if !offline && e.is_connectivity() => {
                offline = true;
                launch::emit_status(&app, "preparing", "Sin conexión — usando archivos locales…");
                let launch_id = resolve_launch_id(
                    &app,
                    &http,
                    &mc,
                    &loader,
                    &loader_version,
                    None,
                    &mut meta,
                    &instance_id,
                    true,
                )
                .await?;
                if loader == "paraguacraft-pvp" {
                    let _ = write_legacy_pvp_play_style(&instance_id, settings.pvp_play_style.as_str());
                }
                if loader == "paraguacraft-pvp-modern" {
                    let settings_tier = crate::core::performance::resolve_tier(
                        &settings,
                        meta.performance_tier.as_deref(),
                    );
                    let _ = modern_pvp::ensure_launch_defaults(
                        &instance_id,
                        &settings_tier,
                        settings.pvp_play_style.as_str(),
                    );
                }
                let auth = resolve_auth(&http, true).await?;
                (auth, launch_id, true)
            }
            Err(e) => return Err(e),
        }
    };

    launch::emit_status(&app, "launching", "Iniciando Java…");
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
        offline,
    )
    .await
}

/// Args del último launch (o de una instancia concreta) para copiar / soporte.
#[tauri::command]
pub fn get_last_launch_args(instance_id: Option<String>) -> AppResult<LastLaunchArgsDto> {
    let id = instance_id
        .filter(|s| !s.trim().is_empty())
        .or_else(crate::core::game_session::last_launch_instance)
        .ok_or_else(|| {
            AppError::msg(
                "Todavía no hay un launch reciente. Jugá una partida y volvé a intentar.",
            )
        })?;
    let (path, text) = launch::read_last_args_file(&id)?;
    Ok(LastLaunchArgsDto {
        instance_id: id,
        path: path.to_string_lossy().into_owned(),
        args: text,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastLaunchArgsDto {
    pub instance_id: String,
    pub path: String,
    pub args: String,
}

/// Sincroniza titulo/artista de musica al HUD in-game (IPC overlay).
#[tauri::command]
pub fn sync_overlay_music(playing: bool, title: String, artist: String, image_url: String) {
    crate::core::overlay_ipc::set_music(playing, &title, &artist, &image_url);
}
