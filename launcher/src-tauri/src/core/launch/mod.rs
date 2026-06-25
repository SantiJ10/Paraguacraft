//! Lanzamiento de Minecraft + SUSPENSION TOTAL (Regla 3).
//!
//! Construye el comando Java (classpath, natives, assets, args con sustitucion
//! de placeholders, soportando perfiles modernos y `minecraftArguments` legacy y
//! merges de loaders via `inheritsFrom`), arma los JVM args (RAM/GC estilo Aikar
//! como en `core.py`) y lanza `javaw` SIN consola.
//!
//! Tras lanzar, el launcher **libera toda la red** (`shutdown_network`) y vacia
//! caches: queda en ~0% CPU y RAM minima mientras el juego corre.

pub mod window_title;
pub mod pvp_jvm;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::core::instances;
use crate::core::paths;
use crate::core::versions;
use crate::error::{AppError, AppResult};

/// Datos de autenticacion resueltos para el lanzamiento.
pub struct AuthCtx {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    /// "msa" (Microsoft) | "legacy" (offline)
    pub user_type: String,
}

/// Config efectiva de JVM (ya resuelto el override del usuario).
pub struct JvmCtx {
    pub ram_mb: u32,
    pub gc: String,
    pub extra_args: Vec<String>,
    pub java_path: PathBuf,
    /// Major detectado (8, 17, 21…) para elegir flags compatibles.
    pub java_major: u32,
    pub mc_version: String,
    pub loader: String,
    /// RAM total del sistema (GB), para presets PvP por gama.
    pub system_ram_gb: f64,
}

fn cp_sep() -> &'static str {
    if cfg!(target_os = "windows") { ";" } else { ":" }
}

/// Resuelve el id vanilla raiz (ej. `1.21.11`) siguiendo `inheritsFrom`.
pub fn resolve_version_chain_base_id(id: &str) -> String {
    let mut current = id.to_string();
    loop {
        let Some(json) = versions::read_local_json(&current) else {
            return current;
        };
        if let Some(parent) = json["inheritsFrom"].as_str() {
            current = parent.to_string();
        } else {
            return current;
        }
    }
}

/// Carga el JSON de una version mergeando `inheritsFrom` recursivamente.
pub fn load_merged(id: &str) -> AppResult<Value> {
    let mut json = versions::read_local_json(id)
        .ok_or_else(|| AppError::msg(format!("Falta el perfil {id}; instalalo primero")))?;
    if let Some(parent_id) = json["inheritsFrom"].as_str().map(String::from) {
        let parent = load_merged(&parent_id)?;
        json = merge(parent, json);
    }
    Ok(json)
}

/// Merge perfil hijo (loader) sobre padre (vanilla). El hijo tiene prioridad en
/// libraries (van primero) y mainClass; el padre aporta assets/jar/assetIndex.
fn merge(parent: Value, child: Value) -> Value {
    let mut out = parent.clone();

    let mut libs = child["libraries"].as_array().cloned().unwrap_or_default();
    libs.extend(parent["libraries"].as_array().cloned().unwrap_or_default());
    out["libraries"] = Value::Array(libs);

    if let Some(mc) = child["mainClass"].as_str() {
        out["mainClass"] = Value::String(mc.to_string());
    }
    if let Some(mca) = child["minecraftArguments"].as_str() {
        out["minecraftArguments"] = Value::String(mca.to_string());
    }
    // arguments: concatenar game y jvm (padre primero, luego hijo).
    if child.get("arguments").is_some() || parent.get("arguments").is_some() {
        let mut game = parent["arguments"]["game"].as_array().cloned().unwrap_or_default();
        game.extend(child["arguments"]["game"].as_array().cloned().unwrap_or_default());
        let mut jvm = parent["arguments"]["jvm"].as_array().cloned().unwrap_or_default();
        jvm.extend(child["arguments"]["jvm"].as_array().cloned().unwrap_or_default());
        out["arguments"] = serde_json::json!({ "game": game, "jvm": jvm });
    }
    out
}

/// Evalua reglas de un argumento. Devuelve false si depende de features
/// opcionales (demo, quickPlay, resolucion): no las activamos.
fn arg_rules_allow(rules: &Value) -> bool {
    let Some(arr) = rules.as_array() else {
        return true;
    };
    let mut allowed = arr.is_empty();
    for rule in arr {
        if rule.get("features").is_some() {
            return false;
        }
        let action_allow = rule["action"].as_str() == Some("allow");
        if let Some(os) = rule.get("os").and_then(|o| o.get("name")).and_then(|n| n.as_str()) {
            if os != versions::os_name() {
                continue;
            }
        }
        allowed = action_allow;
    }
    allowed
}

fn subst(s: &str, map: &HashMap<&str, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in map {
        out = out.replace(&format!("${{{k}}}"), v);
    }
    out
}

fn collect_arg_array(arr: &Value, map: &HashMap<&str, String>) -> Vec<String> {
    let mut out = Vec::new();
    let Some(items) = arr.as_array() else {
        return out;
    };
    for it in items {
        if let Some(s) = it.as_str() {
            out.push(subst(s, map));
        } else if it.is_object() {
            if let Some(rules) = it.get("rules") {
                if !arg_rules_allow(rules) {
                    continue;
                }
            }
            match &it["value"] {
                Value::String(s) => out.push(subst(s, map)),
                Value::Array(a) => {
                    for v in a {
                        if let Some(s) = v.as_str() {
                            out.push(subst(s, map));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// group:artifact de un coordinate maven (para dedup de classpath).
fn ga_key(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() >= 2 {
        format!("{}:{}", parts[0], parts[1])
    } else {
        name.to_string()
    }
}

/// Construye el classpath (libs + client jar), deduplicando por group:artifact
/// (las libs del loader, que van primero, ganan).
fn build_classpath(merged: &Value, base_id: &str) -> Vec<PathBuf> {
    let libs_root = paths::default_minecraft_dir().join("libraries");
    let mut seen: HashSet<String> = HashSet::new();
    let mut cp: Vec<PathBuf> = Vec::new();

    if let Some(libs) = merged["libraries"].as_array() {
        for lib in libs {
            if let Some(rules) = lib.get("rules") {
                if !versions::rules_allow(rules) {
                    continue;
                }
            }
            let name = lib["name"].as_str().unwrap_or("");
            // Las libs de natives puras no van al classpath.
            if name.contains(":natives-") {
                continue;
            }
            // natives map presente sin artifact => skip classpath.
            let has_artifact = lib["downloads"].get("artifact").is_some();
            if lib.get("natives").is_some() && !has_artifact {
                continue;
            }
            let key = if name.is_empty() { String::new() } else { ga_key(name) };
            if !key.is_empty() && !seen.insert(key) {
                continue;
            }
            if let Some(path) = lib["downloads"]["artifact"]["path"].as_str() {
                cp.push(libs_root.join(path));
            } else if !name.is_empty() {
                if let Some(rel) = versions::maven_to_path(name) {
                    cp.push(libs_root.join(rel));
                }
            }
        }
    }
    // Client jar de la base.
    cp.push(versions::jar_path(base_id));
    cp
}

/// Extrae los natives (DLL/SO/DYLIB) de las libs al directorio dado.
/// Extrae a carpeta temporal primero para no borrar natives válidos si falla la descarga.
fn extract_natives(merged: &Value, natives_dir: &Path) -> AppResult<()> {
    let temp = natives_dir.with_extension("tmp_extract");
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp)?;

    let Some(libs) = merged["libraries"].as_array() else {
        return Ok(());
    };
    let mut missing_jars = 0u32;
    for lib in libs {
        if let Some(jar) = versions::native_jar_path(lib) {
            if jar.is_file() {
                extract_native_jar(&jar, &temp)?;
            } else {
                missing_jars += 1;
            }
        }
    }

    if natives_dir_has_binaries(&temp) {
        if natives_dir.exists() {
            let _ = std::fs::remove_dir_all(natives_dir);
        }
        std::fs::rename(&temp, natives_dir).or_else(|_| {
            std::fs::create_dir_all(natives_dir)?;
            for entry in std::fs::read_dir(&temp)?.flatten() {
                let dest = natives_dir.join(entry.file_name());
                if entry.path().is_file() {
                    std::fs::copy(entry.path(), dest)?;
                }
            }
            let _ = std::fs::remove_dir_all(&temp);
            Ok::<(), std::io::Error>(())
        })?;
    } else {
        let _ = std::fs::remove_dir_all(&temp);
        if !natives_dir_has_binaries(natives_dir) && missing_jars > 0 {
            return Err(AppError::msg(format!(
                "Faltan JARs de natives en libraries/ ({missing_jars} ausentes). \
                 Usá Reparar instancia para volver a descargar las dependencias."
            )));
        }
    }
    Ok(())
}

fn natives_dir_has_binaries(dir: &Path) -> bool {
    let ext = if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x == ext)
        })
}

fn validate_natives(natives_dir: &Path, mc_base_id: &str) -> AppResult<()> {
    if !natives_dir_has_binaries(natives_dir) {
        return Err(AppError::msg(format!(
            "Faltan librerías nativas (LWJGL) para Minecraft {mc_base_id}. \
             Usá Reparar instancia o reinstalá la versión desde el launcher."
        )));
    }
    Ok(())
}

const NATIVE_JVM_PREFIXES: &[&str] = &[
    "-Djava.library.path=",
    "-Dorg.lwjgl.librarypath=",
    "-Dorg.lwjgl.system.SharedLibraryExtractPath=",
    "-Djna.tmpdir=",
    "-Dio.netty.native.workdir=",
];

fn is_native_jvm_arg(arg: &str) -> bool {
    NATIVE_JVM_PREFIXES.iter().any(|p| arg.starts_with(p))
}

fn push_native_jvm_args(cmd: &mut Vec<String>, natives_dir: &Path) {
    let p = natives_dir.to_string_lossy().to_string();
    cmd.push(format!("-Djava.library.path={p}"));
    cmd.push(format!("-Dorg.lwjgl.librarypath={p}"));
    cmd.push(format!("-Dorg.lwjgl.system.SharedLibraryExtractPath={p}"));
    cmd.push(format!("-Djna.tmpdir={p}"));
    cmd.push(format!("-Dio.netty.native.workdir={p}"));
}

fn natives_dir_from_args(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .find(|a| a.starts_with("-Djava.library.path="))
        .map(|a| PathBuf::from(a.trim_start_matches("-Djava.library.path=")))
}

fn extract_native_jar(jar: &Path, dest: &Path) -> AppResult<()> {
    let Ok(file) = std::fs::File::open(jar) else {
        return Ok(()); // si falta, seguimos (puede no aplicar a este SO)
    };
    let mut zip = zip::ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(name) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
            continue;
        };
        let lossy = name.to_string_lossy();
        if lossy.starts_with("META-INF") || entry.is_dir() {
            continue;
        }
        // Solo binarios nativos.
        let keep = lossy.ends_with(".dll")
            || lossy.ends_with(".so")
            || lossy.ends_with(".dylib")
            || lossy.ends_with(".jnilib");
        if !keep {
            continue;
        }
        let out = dest.join(name.file_name().unwrap_or_default());
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut buf)?;
        std::fs::write(out, buf)?;
    }
    Ok(())
}

/// JVM args (RAM + GC). Java 8 no soporta flags Aikar/AlwaysPreTouch (Java 9+).
fn merge_extra_jvm_args(args: &mut Vec<String>, extra: &[String]) {
    for a in extra {
        if a.starts_with("-Xmx") || a.starts_with("-Xms") {
            args.retain(|x| !x.starts_with("-Xmx") && !x.starts_with("-Xms"));
        }
        args.push(a.clone());
    }
}

fn build_jvm_ram_gc(jvm: &JvmCtx) -> Vec<String> {
    if pvp_jvm::applies(&jvm.loader, &jvm.mc_version, jvm.java_major) {
        let mut args = pvp_jvm::build_jvm_args(jvm.system_ram_gb);
        merge_extra_jvm_args(&mut args, &jvm.extra_args);
        return args;
    }

    let xms = (jvm.ram_mb / 4).max(512).min(jvm.ram_mb);
    let mut args = vec![
        format!("-Xmx{}M", jvm.ram_mb),
        format!("-Xms{}M", xms),
    ];

    if jvm.java_major <= 8 {
        args.push("-XX:+UseG1GC".into());
        args.push("-XX:+UnlockExperimentalVMOptions".into());
        args.push("-XX:MaxGCPauseMillis=200".into());
        args.push("-XX:+DisableExplicitGC".into());
    } else {
        args.extend([
            "-XX:+UnlockExperimentalVMOptions".into(),
            "-XX:+DisableExplicitGC".into(),
            "-XX:+AlwaysPreTouch".into(),
        ]);
        match jvm.gc.as_str() {
            "ZGC" if jvm.java_major >= 15 => args.push("-XX:+UseZGC".into()),
            "Shenandoah" if jvm.java_major >= 12 => args.push("-XX:+UseShenandoahGC".into()),
            _ => args.extend(
                [
                    "-XX:+UseG1GC",
                    "-XX:G1NewSizePercent=30",
                    "-XX:G1MaxNewSizePercent=40",
                    "-XX:G1HeapRegionSize=8M",
                    "-XX:G1ReservePercent=20",
                    "-XX:G1HeapWastePercent=5",
                    "-XX:G1MixedGCCountTarget=4",
                    "-XX:InitiatingHeapOccupancyPercent=15",
                    "-XX:G1MixedGCLiveThresholdPercent=90",
                    "-XX:G1RSetUpdatingPauseTimePercent=5",
                    "-XX:SurvivorRatio=32",
                    "-XX:+PerfDisableSharedMem",
                    "-XX:MaxTenuringThreshold=1",
                ]
                .iter()
                .map(|s| s.to_string()),
            ),
        }
    }

    merge_extra_jvm_args(&mut args, &jvm.extra_args);
    args
}

/// Construye la lista completa de argumentos para `java`.
pub fn build_command(
    launch_id: &str,
    instance_dir: &Path,
    auth: &AuthCtx,
    jvm: &JvmCtx,
    resolution: Option<(u32, u32)>,
) -> AppResult<(Vec<String>, PathBuf)> {
    let merged = load_merged(launch_id)?;
    let base_id = resolve_version_chain_base_id(launch_id);

    let cp = build_classpath(&merged, &base_id);
    let cp_str = cp
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(cp_sep());

    let natives_dir = versions::versions_dir().join(&base_id).join("natives");
    extract_natives(&merged, &natives_dir)?;
    validate_natives(&natives_dir, &base_id)?;

    let assets_root = paths::default_minecraft_dir().join("assets");
    let asset_index = merged["assetIndex"]["id"]
        .as_str()
        .or_else(|| merged["assets"].as_str())
        .unwrap_or("legacy")
        .to_string();
    let version_type = merged["type"].as_str().unwrap_or("release").to_string();
    let main_class = merged["mainClass"]
        .as_str()
        .ok_or_else(|| AppError::msg("El perfil no tiene mainClass"))?
        .to_string();

    let mut map: HashMap<&str, String> = HashMap::new();
    map.insert("auth_player_name", auth.username.clone());
    map.insert("version_name", launch_id.to_string());
    map.insert("game_directory", instance_dir.to_string_lossy().to_string());
    map.insert("assets_root", assets_root.to_string_lossy().to_string());
    map.insert("game_assets", assets_root.join("virtual").join("legacy").to_string_lossy().to_string());
    map.insert("assets_index_name", asset_index);
    map.insert("auth_uuid", auth.uuid.clone());
    map.insert("auth_access_token", auth.access_token.clone());
    map.insert("auth_session", format!("token:{}", auth.access_token));
    map.insert("clientid", String::new());
    map.insert("auth_xuid", String::new());
    map.insert("user_type", auth.user_type.clone());
    map.insert("version_type", version_type);
    map.insert("user_properties", "{}".into());
    map.insert("natives_directory", natives_dir.to_string_lossy().to_string());
    map.insert("launcher_name", "ParaguacraftLauncher".into());
    map.insert("launcher_version", "2.0".into());
    map.insert("classpath", cp_str.clone());
    map.insert("classpath_separator", cp_sep().to_string());
    map.insert(
        "library_directory",
        paths::default_minecraft_dir().join("libraries").to_string_lossy().to_string(),
    );

    let mut cmd: Vec<String> = Vec::new();
    // 1) JVM RAM/GC propios.
    cmd.extend(build_jvm_ram_gc(jvm));

    // 2) Natives antes que Fabric/Sodium (preLaunch carga LWJGL al iniciar).
    push_native_jvm_args(&mut cmd, &natives_dir);

    // 3) JVM args del perfil (modernos) o defaults (legacy).
    let pvp_jvm_preset = pvp_jvm::applies(&jvm.loader, &jvm.mc_version, jvm.java_major);
    if merged.get("arguments").and_then(|a| a.get("jvm")).is_some() {
        for arg in collect_arg_array(&merged["arguments"]["jvm"], &map) {
            if is_native_jvm_arg(&arg) {
                continue;
            }
            // Evitar -Xmx/-Xms duplicados del perfil Forge cuando ya aplicó preset PvP.
            if pvp_jvm_preset && (arg.starts_with("-Xmx") || arg.starts_with("-Xms")) {
                continue;
            }
            cmd.push(arg);
        }
    } else {
        cmd.push("-cp".into());
        cmd.push(cp_str.clone());
    }

    // 4) Main class.
    cmd.push(main_class);

    // 5) Game args.
    if let Some(mca) = merged["minecraftArguments"].as_str() {
        for tok in mca.split_whitespace() {
            cmd.push(subst(tok, &map));
        }
    } else if merged.get("arguments").and_then(|a| a.get("game")).is_some() {
        cmd.extend(collect_arg_array(&merged["arguments"]["game"], &map));
    }

    if let Some((w, h)) = resolution {
        cmd.push("--width".into());
        cmd.push(w.to_string());
        cmd.push("--height".into());
        cmd.push(h.to_string());
    }

    let java = jvm.java_path.clone();
    Ok((cmd, java))
}

/// Agrega `--server` / `--port` para conectar directo a un servidor multijugador.
pub fn append_server_join(args: &mut Vec<String>, address: &str) {
    let (host, port) = crate::core::favorites::parse_address(address);
    args.push("--server".into());
    args.push(host);
    if let Some(p) = port {
        args.push("--port".into());
        args.push(p.to_string());
    }
}

/// Lanza el proceso sin consola. Devuelve el handle (para esperar su salida).
///
/// En Windows, `@args.txt` solo funciona desde Java 9+. Minecraft 1.8–1.16 usa Java 8:
/// ahí hay que pasar los argumentos en la línea de comandos.
pub fn spawn_game(
    java: &Path,
    args: &[String],
    instance_dir: &Path,
    extra_env: &[(&str, &str)],
    java_major: u32,
) -> AppResult<std::process::Child> {
    std::fs::create_dir_all(instance_dir)?;
    let mut cmd = Command::new(java);
    cmd.current_dir(instance_dir);

    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    if let Some(natives) = natives_dir_from_args(args) {
        prepend_native_path_env(&mut cmd, &natives);
    }

    // Siempre guardamos el argfile para depuración (Paraguabot / soporte).
    let arg_file = instance_dir.join(".paraguacraft-java-args.txt");
    let _ = write_java_arg_file(&arg_file, args);

    #[cfg(target_os = "windows")]
    let use_arg_file = java_major >= 9;
    #[cfg(not(target_os = "windows"))]
    let use_arg_file = false;

    if use_arg_file {
        let at = arg_file.to_string_lossy();
        if at.contains(' ') {
            cmd.arg(format!("@\"{at}\""));
        } else {
            cmd.arg(format!("@{at}"));
        }
    } else {
        cmd.args(args);
    }

    no_console(&mut cmd);
    cmd.spawn()
        .map_err(|e| {
            let preview: String = args.iter().take(8).cloned().collect::<Vec<_>>().join(" ");
            AppError::msg(format!(
                "No se pudo iniciar Java ({}): {e}. Args: {preview}…",
                java.display()
            ))
        })
}

#[cfg(target_os = "windows")]
fn write_java_arg_file(path: &Path, args: &[String]) -> AppResult<()> {
    let mut buf = String::new();
    for arg in args {
        if arg.contains([' ', '\t', '"']) {
            let escaped = arg.replace('\\', "\\\\").replace('"', "\\\"");
            buf.push('"');
            buf.push_str(&escaped);
            buf.push('"');
        } else {
            buf.push_str(arg);
        }
        buf.push('\n');
    }
    std::fs::write(path, buf)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn write_java_arg_file(_path: &Path, _args: &[String]) -> AppResult<()> {
    Ok(())
}

fn prepend_native_path_env(cmd: &mut Command, natives_dir: &Path) {
    let native = natives_dir.to_string_lossy();
    let path = std::env::var("PATH").unwrap_or_default();
    #[cfg(target_os = "windows")]
    let merged = format!("{native};{path}");
    #[cfg(not(target_os = "windows"))]
    let merged = format!("{native}:{path}");
    cmd.env("PATH", merged);
}

/// Instancia con mod Paraguacraft PvP (RPC in-game toma el control).
pub fn has_paraguacraft_pvp_mod(game_dir: &Path) -> bool {
    let mods = game_dir.join("mods");
    let Ok(entries) = std::fs::read_dir(&mods) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.starts_with("paraguacraftpvp") && name.ends_with(".jar") {
            return true;
        }
    }
    false
}

/// Emite `game://started`.
pub fn emit_started(app: &AppHandle, instance_id: &str, pid: u32) {
    let _ = app.emit(
        "game://started",
        serde_json::json!({ "instanceId": instance_id, "pid": pid }),
    );
}

/// Espera (en hilo BLOQUEADO = 0% CPU, sin red) a que el juego cierre, suma el
/// tiempo jugado y emite `game://exited`. No hay polling ni busy-loop.
pub fn watch_exit(
    app: AppHandle,
    instance_id: String,
    mut child: std::process::Child,
    mc_version: String,
    username: String,
    loader: String,
    game_dir: PathBuf,
    launch_server: Option<String>,
    settings: crate::models::AppSettings,
    game_rpc_handoff: bool,
) {
    let started = std::time::Instant::now();
    let pid = child.id();
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    window_title::watch_window_title(pid, &mc_version, &loader, stop.clone());

    if settings.discord_rpc && !game_rpc_handoff {
        crate::core::extras::game_presence::watch(
            crate::core::extras::game_presence::PresenceCtx {
                username: username.clone(),
                mc_version: mc_version.clone(),
                loader: loader.clone(),
                game_dir: game_dir.clone(),
                launch_server: launch_server.clone(),
                settings: settings.clone(),
            },
            stop.clone(),
        );
    }

    tauri::async_runtime::spawn_blocking(move || {
        let status = child.wait().ok();
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut exit_code = status.and_then(|s| s.code()).unwrap_or(-1);
        let minutes = (started.elapsed().as_secs() / 60) as u64;
        if let Some(mut meta) = instances::read_meta(&instance_id) {
            meta.total_play_minutes = meta.total_play_minutes.saturating_add(minutes);
            let _ = instances::write_meta(&instance_id, &meta);
        }
        let diagnosis = if exit_code != 0 {
            crate::core::diagnostics::analyze_instance(&instance_id, exit_code).ok()
        } else {
            None
        };
        // Forge 1.8.9 suele devolver 1 al cerrar normalmente; no marcar como crash.
        if exit_code != 0 {
            if let Some(ref d) = diagnosis {
                if d.category == "clean_exit" {
                    exit_code = 0;
                }
            }
        }
        let payload = serde_json::json!({
            "instanceId": instance_id,
            "minutes": minutes,
            "exitCode": exit_code,
            "diagnosis": diagnosis,
        });
        let _ = app.emit("game://exited", payload.clone());
        if exit_code != 0 {
            let _ = app.emit("game://crashed", payload);
        }
        let settings = crate::config::read_json::<crate::models::AppSettings>(
            &crate::core::paths::config_file(),
        )
        .unwrap_or_default();
        if settings.discord_rpc {
            crate::core::extras::discord_rpc::set_launcher_idle(&username);
        }
    });
}

#[cfg(target_os = "windows")]
fn no_console(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
}

#[cfg(not(target_os = "windows"))]
fn no_console(_cmd: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn fabric_launch_sets_natives_library_path() {
        let launch_id = "fabric-loader-0.19.3-1.21.11";
        if versions::read_local_json(launch_id).is_none() {
            eprintln!("skip: fabric profile not installed");
            return;
        }
        let natives = versions::versions_dir()
            .join("1.21.11")
            .join("natives");
        let auth = AuthCtx {
            username: "test".into(),
            uuid: "0".repeat(32),
            access_token: "0".into(),
            user_type: "legacy".into(),
        };
        let jvm = JvmCtx {
            ram_mb: 2048,
            gc: "Auto".into(),
            extra_args: vec![],
            java_path: PathBuf::from("java"),
            java_major: 21,
            mc_version: "1.21.11".into(),
            loader: "fabric".into(),
            system_ram_gb: 16.0,
        };
        let inst = paths::instances_dir().join("Paraguacraft_1.21.11_fabric-iris");
        let (args, _) = build_command(launch_id, &inst, &auth, &jvm, None).expect("build_command");
        let lib_args: Vec<_> = args
            .iter()
            .filter(|a| a.starts_with("-Djava.library.path="))
            .collect();
        assert_eq!(
            lib_args.len(),
            1,
            "expected one library path, got {lib_args:?} in {} args",
            args.len()
        );
        let lwjgl_path = args
            .iter()
            .find(|a| a.starts_with("-Dorg.lwjgl.librarypath="))
            .expect("org.lwjgl.librarypath");
        assert_eq!(
            lib_args[0].trim_start_matches("-Djava.library.path="),
            lwjgl_path.trim_start_matches("-Dorg.lwjgl.librarypath=")
        );
        let expected = natives.to_string_lossy().to_string();
        assert!(
            lib_args[0].ends_with(&expected.replace('/', "\\")) || lib_args[0].contains("1.21.11\\natives"),
            "bad library path: {}",
            lib_args[0]
        );
        eprintln!("library path arg: {}", lib_args[0]);
        eprintln!("first 15 args: {:?}", &args[..args.len().min(15)]);
    }
}
