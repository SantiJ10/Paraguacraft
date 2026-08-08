//! Servidores locales + Playit.gg invisible (CREATE_NO_WINDOW).

use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config;
use crate::core::paths;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerProfile {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    /// paper | paper-geyser | fabric | fabric-geyser | forge
    pub server_type: String,
    pub ram_mb: u32,
    pub port: u16,
    pub created_at: u64,
    pub playit_address: Option<String>,
    /// Carpeta externa importada (ruta absoluta). Si es None, usa `{data}/servers/{id}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_folder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub id: String,
    pub running: bool,
    pub playit_running: bool,
    pub playit_address: Option<String>,
    pub pid: Option<u32>,
    /// El usuario confirmó haber vinculado el agente Playit a su cuenta
    /// (asistente primera vez). Persistido en `_paragua_srv.json`.
    pub playit_claimed: bool,
    /// Última línea `…/claim` vista en el log de Playit (para mostrar el link).
    pub playit_claim_hint: Option<String>,
    /// `true` si hay jar del plugin playit en `plugins/` (túnel nativo Paper).
    #[serde(default)]
    pub playit_plugin_mode: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct ServersFile {
    #[serde(default)]
    servers: Vec<ServerProfile>,
}

fn servers_file() -> PathBuf {
    paths::data_dir().join("servers.json")
}

fn server_dir(id: &str) -> PathBuf {
    paths::data_dir().join("servers").join(id)
}

pub fn profile_by_id(id: &str) -> AppResult<ServerProfile> {
    load_all()
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| AppError::msg("Servidor no encontrado"))
}

pub fn folder_for(prof: &ServerProfile) -> PathBuf {
    prof.custom_folder
        .as_ref()
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .unwrap_or_else(|| server_dir(&prof.id))
}

pub fn folder_for_id(id: &str) -> AppResult<PathBuf> {
    Ok(folder_for(&profile_by_id(id)?))
}

fn load_all() -> Vec<ServerProfile> {
    config::read_json::<ServersFile>(&servers_file())
        .unwrap_or_default()
        .servers
}

fn save_all(list: &[ServerProfile]) -> AppResult<()> {
    config::write_json_atomic(&servers_file(), &ServersFile { servers: list.to_vec() })
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

static PROCESSES: Mutex<Option<HashMap<String, ServerProcs>>> = Mutex::new(None);

struct McProcess {
    child: Child,
    stdin: Mutex<Option<std::process::ChildStdin>>,
}

struct ServerProcs {
    mc: Option<McProcess>,
    playit: Option<Child>,
}

fn procs() -> std::sync::MutexGuard<'static, Option<HashMap<String, ServerProcs>>> {
    let mut g = PROCESSES.lock().unwrap();
    if g.is_none() {
        *g = Some(HashMap::new());
    }
    g
}

pub fn list() -> Vec<ServerProfile> {
    load_all()
}

/// Carpetas en disco de todos los servidores registrados.
pub fn list_server_dirs() -> Vec<PathBuf> {
    load_all()
        .into_iter()
        .map(|s| folder_for(&s))
        .filter(|p| p.is_dir())
        .collect()
}

/// Valores de RAM permitidos para servidores (MB).
pub const RAM_PRESETS_MB: [u32; 6] = [2048, 4096, 6144, 8192, 12288, 16384];

pub fn normalize_ram_mb(ram_mb: u32) -> AppResult<u32> {
    if ram_mb == 0 {
        return Ok(4096);
    }
    if RAM_PRESETS_MB.contains(&ram_mb) {
        return Ok(ram_mb);
    }
    Err(AppError::msg(
        "RAM invalida. Valores permitidos: 2, 4, 6, 8, 12 o 16 GB.",
    ))
}

pub fn create(name: &str, mc_version: &str, server_type: &str, ram_mb: u32) -> AppResult<ServerProfile> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::msg("Nombre vacio"));
    }
    let server_type = crate::core::server_setup::normalize_server_type(server_type)?;
    let id = format!("srv-{}", uuid::Uuid::new_v4());
    let dir = server_dir(&id);
    std::fs::create_dir_all(dir.join("plugins"))?;
    std::fs::create_dir_all(dir.join("mods"))?;
    std::fs::create_dir_all(dir.join("world"))?;
    ensure_eula(&dir)?;
    let profile = ServerProfile {
        id: id.clone(),
        name: name.to_string(),
        mc_version: mc_version.to_string(),
        server_type,
        ram_mb: normalize_ram_mb(ram_mb)?,
        port: 25565,
        created_at: now(),
        playit_address: crate::core::playit_shared::shared_address(),
        custom_folder: None,
    };
    // Prefill plugin secret si ya hay uno en la cuenta local del launcher.
    let _ = crate::core::playit_shared::apply_to_server(&dir);
    let _ = ensure_server_properties(&dir, 25565);
    let mut all = load_all();
    all.push(profile.clone());
    save_all(&all)?;
    write_server_meta(&dir, &profile)?;
    Ok(profile)
}

/// Indica si falta preparar el JAR del servidor.
pub fn needs_prepare(dir: &Path, server_type: &str) -> bool {
    let Ok(kind) = crate::core::server_setup::normalize_server_type(server_type) else {
        return true;
    };
    if crate::core::server_setup::is_forge_style(&kind) {
        return !dir.join("run.bat").is_file()
            && !dir.join("run.sh").is_file()
            && crate::core::server_setup::find_server_jar(dir, &kind).is_none();
    }
    if kind.starts_with("fabric") {
        return !dir.join("fabric-server-launch.jar").is_file() && !dir.join("server.jar").is_file();
    }
    !dir.join("server.jar").is_file()
}

pub fn update_profile(
    id: &str,
    name: Option<&str>,
    mc_version: Option<&str>,
    ram_mb: Option<u32>,
    port: Option<u16>,
) -> AppResult<ServerProfile> {
    let mut all = load_all();
    let idx = all
        .iter()
        .position(|s| s.id == id)
        .ok_or_else(|| AppError::msg("Servidor no encontrado"))?;
    if let Some(n) = name.map(str::trim).filter(|s| !s.is_empty()) {
        all[idx].name = n.to_string();
    }
    if let Some(mc) = mc_version.map(str::trim).filter(|s| !s.is_empty()) {
        all[idx].mc_version = mc.to_string();
    }
    if let Some(ram) = ram_mb {
        all[idx].ram_mb = normalize_ram_mb(ram)?;
    }
    if let Some(p) = port {
        all[idx].port = p;
    }
    let prof = all[idx].clone();
    save_all(&all)?;
    let dir = folder_for(&prof);
    write_server_meta(&dir, &prof)?;
    Ok(prof)
}

fn write_server_meta(dir: &Path, prof: &ServerProfile) -> AppResult<()> {
    let meta = serde_json::json!({
        "tipo": prof.server_type,
        "version": prof.mc_version,
        "ram_gb": prof.ram_mb / 1024,
    });
    config::write_json_atomic(&dir.join("_paragua_srv.json"), &meta)
}

fn ensure_server_properties(dir: &Path, port: u16) -> AppResult<()> {
    let path = dir.join("server.properties");
    if !path.is_file() {
        std::fs::write(
            path,
            format!(
                "online-mode=false\nserver-port={port}\nmotd=Paraguacraft Server\nmax-players=20\nview-distance=10\nsimulation-distance=8\n"
            ),
        )?;
        return Ok(());
    }
    // Unificar puerto con el perfil (el túnel Playit apunta a un solo puerto local).
    let mut updates = std::collections::HashMap::new();
    updates.insert("server-port".to_string(), port.to_string());
    if crate::core::server_properties::read(dir)
        .ok()
        .and_then(|p| p.get("online-mode").cloned())
        .is_none()
    {
        updates.insert("online-mode".to_string(), "false".to_string());
    }
    let _ = crate::core::server_properties::write(dir, &updates);
    Ok(())
}

/// Otro servidor MC del launcher en ejecución (id, nombre).
fn other_running_mc(except_id: &str) -> Option<(String, String)> {
    for p in load_all() {
        if p.id == except_id {
            continue;
        }
        if is_mc_running(&p.id) {
            return Some((p.id.clone(), p.name.clone()));
        }
    }
    None
}

fn resolve_server_java(mc: &str) -> AppResult<PathBuf> {
    use crate::config;
    use crate::models::AppSettings;

    let settings: AppSettings = config::read_json(&paths::config_file()).unwrap_or_default();
    crate::core::java::resolve::resolve_sync(
        mc,
        Some(mc),
        settings.java_path.as_deref(),
        crate::core::java::resolve::JavaRole::Installer,
    )
}

fn resolve_server_jar(dir: &Path, server_type: &str) -> AppResult<PathBuf> {
    let kind = crate::core::server_setup::normalize_server_type(server_type)?;
    if crate::core::server_setup::is_forge_style(&kind) {
        return crate::core::server_setup::find_server_jar(dir, &kind).ok_or_else(|| {
            AppError::msg(format!(
                "{} no preparado. Usa «Preparar servidor» primero.",
                crate::core::server_setup::type_label(&kind)
            ))
        });
    }
    let jar = dir.join("server.jar");
    if jar.is_file() {
        return Ok(jar);
    }
    Err(AppError::msg(
        "server.jar no encontrado. Usa «Preparar servidor» primero.",
    ))
}

fn aikar_flags(ram_mb: u32) -> Vec<&'static str> {
    let xmx_gb = ram_mb / 1024;
    if xmx_gb >= 4 {
        vec![
            "-XX:+UseG1GC",
            "-XX:+ParallelRefProcEnabled",
            "-XX:MaxGCPauseMillis=200",
            "-XX:+UnlockExperimentalVMOptions",
            "-XX:+DisableExplicitGC",
            "-XX:+AlwaysPreTouch",
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
            "-Dusing.aikars.flags=https://mcflags.emc.gs",
            "-Daikars.new.flags=true",
        ]
    } else {
        vec![
            "-XX:+UseG1GC",
            "-XX:+ParallelRefProcEnabled",
            "-XX:MaxGCPauseMillis=200",
            "-XX:+UnlockExperimentalVMOptions",
            "-XX:+DisableExplicitGC",
            "-XX:+AlwaysPreTouch",
            "-XX:G1HeapRegionSize=4M",
            "-XX:InitiatingHeapOccupancyPercent=20",
            "-XX:+PerfDisableSharedMem",
        ]
    }
}

fn is_mc_running(id: &str) -> bool {
    let mut g = procs();
    if let Some(map) = g.as_mut() {
        if let Some(sp) = map.get_mut(id) {
            if let Some(mc) = &mut sp.mc {
                return mc.child.try_wait().ok().flatten().is_none();
            }
        }
    }
    false
}

/// Inicia el proceso del servidor MC (Java headless, sin ventana).
pub fn start_mc(id: &str) -> AppResult<u32> {
    let st = status(id)?;
    if st.running {
        return Err(AppError::msg("El servidor ya está en ejecución"));
    }
    // Un solo Paper/Java a la vez: el túnel Playit compartido solo enruta al puerto local
    // del proceso activo.
    if let Some((_oid, name)) = other_running_mc(id) {
        return Err(AppError::msg(format!(
            "«{name}» sigue corriendo. Con la IP Playit compartida solo puede haber un servidor a la vez — detenelo y volvé a iniciar."
        )));
    }

    let prof = profile_by_id(id)?;
    let dir = folder_for(&prof);
    ensure_eula(&dir)?;
    let port = if prof.port > 0 { prof.port } else { 25565 };
    ensure_server_properties(&dir, port)?;

    // Secret/IP compartidos → misma dirección en todos los mundos / servers del launcher.
    match crate::core::playit_shared::apply_to_server(&dir) {
        Ok(true) => {
            crate::core::server_console::clear(id);
            crate::core::server_console::append(
                id,
                "[playit] Secret compartido aplicado (misma IP para todos tus servers). No se crea un agente nuevo.",
            );
            if let Some(addr) = crate::core::playit_shared::shared_address() {
                let _ = set_playit_address(id, &addr);
                crate::core::server_console::append(
                    id,
                    &format!("[playit] 💾 IP compartida: {addr}"),
                );
            }
        }
        Ok(false) => {
            crate::core::server_console::clear(id);
            crate::core::server_console::append(
                id,
                "[playit] Primera vez: claim en la consola vincula el agente; se reutilizará para todos los servers.",
            );
        }
        Err(e) => {
            crate::core::server_console::clear(id);
            crate::core::server_console::append(
                id,
                &format!("[playit] ⚠ No se pudo aplicar secret compartido: {e}"),
            );
        }
    }

    let java = resolve_server_java(&prof.mc_version)?;
    let java_home = java
        .parent()
        .and_then(|bin| bin.parent())
        .map(|h| h.to_path_buf());
    let ram = prof.ram_mb;
    let kind = crate::core::server_setup::normalize_server_type(&prof.server_type)?;

    // clear ya se hizo en apply_to_server path; si falló early puede no - safe re-append only
    if crate::core::server_console::get_lines(id, 1).is_empty() {
        crate::core::server_console::clear(id);
    }
    crate::core::server_console::append(id, "[launcher] Iniciando servidor…");

    let run_bat = dir.join("run.bat");
    let mut child = if crate::core::server_setup::is_forge_style(&kind) && run_bat.is_file() {
        crate::core::server_console::append(
            id,
            &format!(
                "[launcher] {}: cmd /C call run.bat nogui",
                crate::core::server_setup::type_label(&kind)
            ),
        );
        let (stdin, stdout, stderr) = crate::core::server_console::pipe_stdio();
        let mut cmd = Command::new("cmd");
        cmd.current_dir(&dir).args(["/C", "call", "run.bat", "nogui"]);
        if let Some(ref home) = java_home {
            cmd.env("JAVA_HOME", home);
        }
        cmd.stdin(stdin).stdout(stdout).stderr(stderr);
        hide_console(&mut cmd);
        cmd.spawn()
            .map_err(|e| AppError::msg(format!("No se pudo iniciar run.bat: {e}")))?
    } else {
        let jar = resolve_start_jar(&dir, &prof.server_type)?;
        let xms = (ram / 4).max(512);
        let jar_disp = jar.to_string_lossy();
        crate::core::server_console::append(
            id,
            &format!(
                "[launcher] {} -Xmx{ram}M -Xms{xms}M -jar {jar_disp} nogui",
                java.to_string_lossy()
            ),
        );
        let mut cmd = Command::new(&java);
        cmd.current_dir(&dir)
            .arg(format!("-Xmx{ram}M"))
            .arg(format!("-Xms{xms}M"));
        if let Some(ref home) = java_home {
            cmd.env("JAVA_HOME", home);
        }
        for flag in aikar_flags(ram) {
            cmd.arg(flag);
        }
        cmd.arg("-jar").arg(&jar).arg("nogui");
        let (stdin, stdout, stderr) = crate::core::server_console::pipe_stdio();
        cmd.stdin(stdin).stdout(stdout).stderr(stderr);
        hide_console(&mut cmd);
        cmd.spawn()
            .map_err(|e| AppError::msg(format!("No se pudo iniciar el servidor: {e}")))?
    };

    let pid = child.id();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdin = child.stdin.take();

    if let Some(out) = stdout {
        crate::core::server_console::spawn_stdout_reader(id.to_string(), out);
    }
    if let Some(err) = stderr {
        crate::core::server_console::spawn_stderr_reader(id.to_string(), err);
    }

    let id_tail = id.to_string();
    let dir_tail = dir.clone();
    crate::core::server_console::spawn_log_tail(id_tail.clone(), dir_tail, move || is_mc_running(&id_tail));

    let mc = McProcess {
        child,
        stdin: Mutex::new(stdin),
    };

    let mut g = procs();
    let map = g.as_mut().unwrap();
    map.entry(id.to_string())
        .or_insert(ServerProcs {
            mc: None,
            playit: None,
        })
        .mc = Some(mc);

    spawn_mc_lifecycle_watcher(id.to_string());
    crate::core::server_console::append(id, &format!("[launcher] PID {pid} — consola integrada (copiá desde la pestaña Consola)"));
    Ok(pid)
}

fn spawn_mc_lifecycle_watcher(id: String) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let mut g = procs();
            let Some(map) = g.as_mut() else {
                break;
            };
            let Some(sp) = map.get_mut(&id) else {
                break;
            };
            let Some(mc) = &mut sp.mc else {
                break;
            };
            match mc.child.try_wait() {
                Ok(Some(status)) => {
                    drop(g);
                    crate::core::server_console::append(
                        &id,
                        &format!(
                            "[launcher] El servidor se detuvo (código {:?}). Revisá el log arriba o logs/latest.log.",
                            status.code()
                        ),
                    );
                    let mut g2 = procs();
                    if let Some(map2) = g2.as_mut() {
                        if let Some(sp2) = map2.get_mut(&id) {
                            sp2.mc = None;
                        }
                    }
                    break;
                }
                Ok(None) => {}
                Err(e) => {
                    crate::core::server_console::append(
                        &id,
                        &format!("[launcher] Error comprobando proceso: {e}"),
                    );
                    break;
                }
            }
        }
    });
}

fn resolve_start_jar(dir: &Path, server_type: &str) -> AppResult<PathBuf> {
    let fabric = dir.join("fabric-server-launch.jar");
    if fabric.is_file() {
        return Ok(fabric);
    }
    resolve_server_jar(dir, server_type)
}

pub fn delete(id: &str) -> AppResult<()> {
    stop(id)?;
    let prof = profile_by_id(id)?;
    let mut all = load_all();
    all.retain(|s| s.id != id);
    save_all(&all)?;
    if prof.custom_folder.is_none() {
        let dir = server_dir(id);
        if dir.exists() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
    crate::core::server_console::clear(id);
    Ok(())
}

pub fn status(id: &str) -> AppResult<ServerStatus> {
    let prof = profile_by_id(id)?;
    let mut running = false;
    let mut playit_running = false;
    let mut pid = None;
    let mut g = procs();
    if let Some(map) = g.as_mut() {
        if let Some(sp) = map.get_mut(id) {
            if let Some(mc) = &mut sp.mc {
                running = mc.child.try_wait().ok().flatten().is_none();
                pid = if running { Some(mc.child.id()) } else { None };
            }
            if let Some(p) = &mut sp.playit {
                playit_running = p.try_wait().ok().flatten().is_none();
            }
        }
    }
    let dir = folder_for(&prof);
    let shared_addr = crate::core::playit_shared::shared_address();
    Ok(ServerStatus {
        id: id.to_string(),
        running,
        playit_running,
        playit_address: prof
            .playit_address
            .clone()
            .or_else(|| playit_address_from_meta(&dir))
            .or(shared_addr),
        pid,
        playit_claimed: playit_claimed_from_meta(&dir)
            || crate::core::playit_shared::load().claimed
            || crate::core::playit_shared::has_agent_secret(),
        playit_claim_hint: playit_claim_hint_from_meta(&dir),
        playit_plugin_mode: playit_plugin_present(id),
    })
}

fn playit_address_from_meta(dir: &Path) -> Option<String> {
    let path = dir.join("_paragua_srv.json");
    let v: serde_json::Value = config::read_json(&path)?;
    v.get("java_address")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn playit_claimed_from_meta(dir: &Path) -> bool {
    let path = dir.join("_paragua_srv.json");
    let Some(v) = config::read_json::<serde_json::Value>(&path) else {
        return false;
    };
    v.get("playit_claimed").and_then(|x| x.as_bool()).unwrap_or(false)
}

fn playit_claim_hint_from_meta(dir: &Path) -> Option<String> {
    let path = dir.join("_paragua_srv.json");
    let v: serde_json::Value = config::read_json(&path)?;
    v.get("playit_claim_hint")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// Marca el asistente Playit primera vez como completado (el usuario confirmó
/// que vinculó el agente a su cuenta playit.gg).
pub fn mark_playit_claimed(id: &str) -> AppResult<()> {
    let dir = folder_for_id(id)?;
    let meta_path = dir.join("_paragua_srv.json");
    let mut meta: serde_json::Value = config::read_json(&meta_path).unwrap_or(serde_json::json!({}));
    if let Some(obj) = meta.as_object_mut() {
        obj.insert("playit_claimed".into(), serde_json::Value::Bool(true));
    }
    config::write_json_atomic(&meta_path, &meta)
}

fn save_playit_claim_hint(dir: &Path, line: &str) {
    let meta_path = dir.join("_paragua_srv.json");
    let mut meta: serde_json::Value = config::read_json(&meta_path).unwrap_or(serde_json::json!({}));
    if let Some(obj) = meta.as_object_mut() {
        obj.insert(
            "playit_claim_hint".into(),
            serde_json::Value::String(line.to_string()),
        );
    }
    let _ = config::write_json_atomic(&meta_path, &meta);
}

fn set_playit_address(id: &str, address: &str) -> AppResult<()> {
    let dir = folder_for_id(id)?;
    let meta_path = dir.join("_paragua_srv.json");
    let mut meta: serde_json::Value = config::read_json(&meta_path).unwrap_or(serde_json::json!({}));
    let same_local = meta.get("java_address").and_then(|x| x.as_str()) == Some(address);

    if let Some(obj) = meta.as_object_mut() {
        obj.insert(
            "java_address".into(),
            serde_json::Value::String(address.to_string()),
        );
        // Si el secret del plugin ya existía / se acaba de claimar, cosechar al store global.
        if let Some(s) = crate::core::playit_shared::read_plugin_secret_from_server(&dir) {
            let _ = crate::core::playit_shared::set_agent_secret(&s, Some(address));
            obj.insert("playit_claimed".into(), serde_json::Value::Bool(true));
        }
    }
    config::write_json_atomic(&meta_path, &meta)?;

    // Store + perfil de TODOS los servers → misma IP en la lista del launcher.
    let _ = crate::core::playit_shared::set_address(address);
    let _ = crate::core::playit_shared::harvest_from_server(&dir);
    let _ = crate::core::playit_shared::harvest_agent_toml_from_server(&dir);

    let mut all = load_all();
    for p in all.iter_mut() {
        p.playit_address = Some(address.to_string());
    }
    save_all(&all)?;
    if !same_local {
        // ya
    }
    Ok(())
}

fn parse_playit_address(line: &str) -> Option<String> {
    const SUFFIXES: &[&str] = &[
        "ply.gg",
        "joinmc.link",
        "auto.playit.gg",
        "playit.gg",
    ];
    for suffix in SUFFIXES {
        if let Some(suf_idx) = line.find(suffix) {
            let host_end = suf_idx + suffix.len();
            let port_end = line[host_end..]
                .find(|c: char| !c.is_ascii_digit() && c != ':')
                .map(|i| host_end + i)
                .unwrap_or(line.len());
            let start = line[..suf_idx]
                .rfind(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '.')
                .map(|i| i + 1)
                .unwrap_or(0);
            let addr = line[start..port_end].trim();
            if addr.contains('.') && !addr.starts_with(|c: char| c.is_ascii_digit()) {
                return Some(addr.to_string());
            }
        }
    }
    None
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc.is_ascii_alphabetic() || nc == '@' {
                        break;
                    }
                }
            } else {
                chars.next();
            }
            continue;
        }
        out.push(c);
    }
    out
}

fn process_playit_log_chunk(id: &str, chunk: &str, seen: &mut HashSet<String>) {
    for raw in chunk.split('\n') {
        for part in raw.split('\r') {
            let line = strip_ansi(part).trim().to_string();
            if line.is_empty() || seen.contains(&line) {
                continue;
            }
            if seen.len() > 400 {
                seen.clear();
            }
            seen.insert(line.clone());
            crate::core::server_console::append(id, &format!("[playit] {line}"));
            if let Some(addr) = parse_playit_address(&line) {
                let _ = set_playit_address(id, &addr);
            }
            if line.contains("InvalidAgentKey")
                || line.contains("no longer valid")
                || line.contains("agent key is not valid")
            {
                crate::core::server_console::append(
                    id,
                    "[playit] ⚠ Clave de agente inválida o expirada. Usá «Resetear Playit» o borra playit-agent.toml y vuelve a claim.",
                );
                let _ = wipe_playit_local_secrets(id);
            }
            if line.contains("frontend secret provisioning")
                || line.contains("Waiting for frontend secret")
            {
                crate::core::server_console::append(
                    id,
                    "[playit] ⚠ El agente desktop se quedó esperando un secret por IPC (modo incorrecto). Deteniendo y limpiando secret local…",
                );
                let _ = stop_playit_only(id);
                let _ = wipe_playit_local_secrets(id);
                crate::core::server_console::append(
                    id,
                    "[playit] Con Paper usá el **plugin** (arranque del servidor → claim en consola). No mezcles playit.exe si ya está el jar en plugins/.",
                );
            }
            if line.contains("max agents")
                || line.contains("too many agents")
                || line.contains("agent limit")
                || line.contains("Maximum number of agents")
                || line.contains("upgrade your account")
            {
                crate::core::server_console::append(
                    id,
                    "[playit] ⚠ Límite de agentes/túneles en playit.gg. Entrá a https://playit.gg/account/agents y borrá agentes viejos; luego «Resetear Playit» y reiniciá el servidor.",
                );
            }
            if line.contains("playit.gg/claim") {
                crate::core::server_console::append(
                    id,
                    "[playit] ⚠ Vinculá tu agente en el enlace claim de arriba (cuenta playit.gg).",
                );
                if let Ok(dir) = folder_for_id(id) {
                    save_playit_claim_hint(&dir, &line);
                }
            }
        }
    }
}

/// Escanea líneas de la consola Paper (plugin playit-gg) para IP y claim.
/// No re-procesa nuestras propias líneas `[playit]` / `[launcher]`.
pub fn on_mc_console_line(id: &str, line: &str) {
    let line = strip_ansi(line);
    let t = line.trim();
    if t.is_empty() {
        return;
    }
    if t.starts_with("[playit]") || t.starts_with("[launcher]") {
        return;
    }
    // claim del plugin, ej: please visit: https://playit.gg/claim/xxxx
    if t.contains("playit.gg/claim") {
        if let Ok(dir) = folder_for_id(id) {
            save_playit_claim_hint(&dir, t);
        }
        // no append extra: la línea del MC ya está en consola
    }
    // IP: "playit.gg: katherine-momentum.tun.ply.gg" o "found minecraft java tunnel: …"
    if let Some(addr) = parse_playit_address(t) {
        if let Ok(dir) = folder_for_id(id) {
            let prev = playit_address_from_meta(&dir);
            if prev.as_deref() != Some(addr.as_str()) {
                // set_playit_address + aviso una sola vez
                if set_playit_address(id, &addr).is_ok() {
                    crate::core::server_console::append(
                        id,
                        &format!("[playit] ✅ Dirección del túnel: {addr}"),
                    );
                }
            } else {
                let _ = set_playit_address(id, &addr);
            }
        }
    }
    if t.contains("failed to send data to tunnel") {
        crate::core::server_console::append(
            id,
            "[playit] ⚠ Se cortó el túnel de un jugador (red/playit inestable). El servidor puede seguir; el amigo puede reconectar.",
        );
    }
    if t.contains("secret key not set") || t.contains("generate claim code") {
        // Si ya tenemos secret compartido, reintentar escribirlo (plugin puede haber reiniciado vacío)
        if let Ok(dir) = folder_for_id(id) {
            if crate::core::playit_shared::has_agent_secret() {
                let _ = crate::core::playit_shared::apply_to_server(&dir);
                crate::core::server_console::append(
                    id,
                    "[playit] ⚠ El plugin no vio el secret. Reaplicé el compartido — reiniciá el servidor una vez.",
                );
            } else {
                crate::core::server_console::append(
                    id,
                    "[playit] Generando enlace claim del plugin — abrí el link playit.gg/claim en la consola y vinculá tu cuenta (se reutiliza en todos los servers).",
                );
            }
        }
    }
    if t.contains("secret verified")
        || t.contains("keys setup complete")
        || t.contains("tunnel setup")
        || t.contains("found minecraft java tunnel")
    {
        if let Ok(dir) = folder_for_id(id) {
            let had = crate::core::playit_shared::has_agent_secret();
            if let Some(_) = crate::core::playit_shared::harvest_from_server(&dir) {
                if !had {
                    crate::core::server_console::append(
                        id,
                        "[playit] ✅ Secret guardado de forma compartida: el próximo server usará la misma IP.",
                    );
                }
            }
        }
    }
}

/// Borra secrets locales (agente desktop + config del plugin) sin borrar el jar.
fn wipe_playit_local_secrets(id: &str) -> AppResult<()> {
    let dir = folder_for_id(id)?;
    let secret = playit_secret_path(&dir);
    if secret.is_file() {
        let _ = std::fs::remove_file(&secret);
    }
    // Datos típicos del plugin playit-gg
    for sub in [
        "plugins/playit-gg",
        "plugins/Playit",
        "plugins/playit",
    ] {
        let p = dir.join(sub);
        if p.is_dir() {
            for name in ["config.yml", "config.yaml", "secret.key", "secret", "playit.toml"] {
                let f = p.join(name);
                if f.is_file() {
                    let _ = std::fs::remove_file(f);
                }
            }
            let _ = std::fs::remove_file(p.join("secret.key"));
        }
    }
    let meta = dir.join("_paragua_srv.json");
    if let Some(mut v) = config::read_json::<serde_json::Value>(&meta) {
        if let Some(obj) = v.as_object_mut() {
            obj.remove("playit_claim_hint");
            obj.insert("playit_claimed".into(), serde_json::Value::Bool(false));
            let _ = config::write_json_atomic(&meta, &v);
        }
    }
    Ok(())
}

/// Reset Playit.
/// - `wipe_shared = false` (default): limpia esta carpeta y **reaplica** el secret global (misma IP).
/// - `wipe_shared = true`: también borra el store global → claim nuevo para todos.
pub fn reset_playit(id: &str) -> AppResult<String> {
    reset_playit_ex(id, false)
}

pub fn reset_playit_ex(id: &str, wipe_shared: bool) -> AppResult<String> {
    stop_playit_only(id)?;
    wipe_playit_local_secrets(id)?;
    if wipe_shared {
        let _ = crate::core::playit_shared::wipe_shared();
        // Limpia IP cacheada en perfiles
        let mut all = load_all();
        for p in all.iter_mut() {
            p.playit_address = None;
        }
        let _ = save_all(&all);
        crate::core::server_console::append(
            id,
            "[playit] Secrets locales + compartidos borrados. En playit.gg/account/agents borrá agentes huérfanos si el cupo free está lleno.",
        );
        crate::core::server_console::append(
            id,
            "[playit] Reiniciá el servidor para un claim NUEVO (una sola vez sirve para todos los mundos).",
        );
        return Ok(
            "Playit reseteado por completo. Borrá agentes viejos en playit.gg si hacía falta y reiniciá el servidor."
                .into(),
        );
    }
    // Restaurar secret compartido en esta carpeta
    if let Ok(dir) = folder_for_id(id) {
        let applied = crate::core::playit_shared::apply_to_server(&dir).unwrap_or(false);
        if applied {
            if let Some(addr) = crate::core::playit_shared::shared_address() {
                let _ = set_playit_address(id, &addr);
            }
            crate::core::server_console::append(
                id,
                "[playit] Carpeta limpia y secret COMPARTIDO restaurado (misma IP). Reiniciá el server.",
            );
            return Ok(
                "Playit de este server restaurado con el secret compartido (misma IP). Reiniciá el servidor."
                    .into(),
            );
        }
    }
    crate::core::server_console::append(
        id,
        "[playit] No había secret compartido. Reiniciá el servidor y hacé el claim una vez.",
    );
    Ok(
        "Secrets locales borrados. Si no tenés secret compartido, claim una vez al reiniciar; si falla por cupo, usá reseteo total."
            .into(),
    )
}

fn playit_process_running(id: &str) -> bool {
    let mut g = procs();
    let Some(map) = g.as_mut() else {
        return false;
    };
    let Some(sp) = map.get_mut(id) else {
        return false;
    };
    let Some(p) = &mut sp.playit else {
        return false;
    };
    p.try_wait().ok().flatten().is_none()
}

/// Lee `.playit-launcher.log` (evita buffering de pipes; el agente usa spinner `\r`).
fn spawn_playit_log_tail(id: String, log_path: PathBuf) {
    std::thread::spawn(move || {
        let mut pos = 0u64;
        let mut seen = HashSet::new();
        while playit_process_running(&id) {
            std::thread::sleep(Duration::from_millis(800));
            let Ok(mut f) = std::fs::File::open(&log_path) else {
                continue;
            };
            if f.seek(SeekFrom::Start(pos)).is_err() {
                continue;
            }
            let mut buf = String::new();
            if f.read_to_string(&mut buf).is_err() {
                continue;
            }
            pos += buf.len() as u64;
            if !buf.is_empty() {
                process_playit_log_chunk(&id, &buf, &mut seen);
            }
        }
        if let Ok(rest) = std::fs::read_to_string(&log_path) {
            if (rest.len() as u64) > pos {
                process_playit_log_chunk(&id, &rest[pos as usize..], &mut seen);
            }
        }
        let _ = std::fs::remove_file(&log_path);
        crate::core::server_console::append(&id, "[playit] Proceso playit finalizado.");
    });
}

fn spawn_playit_early_exit_watch(id: String, log_path: PathBuf) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(3));
        if playit_process_running(&id) {
            return;
        }
        match std::fs::read_to_string(&log_path) {
            Ok(content) if content.trim().is_empty() => {
                crate::core::server_console::append(
                    &id,
                    "[playit] El proceso terminó al instante sin logs. Ejecutá playit.exe manualmente en la carpeta del servidor o usá «Preparar servidor».",
                );
            }
            Ok(content) => {
                let mut seen = HashSet::new();
                process_playit_log_chunk(&id, &content, &mut seen);
            }
            Err(_) => {
                crate::core::server_console::append(
                    &id,
                    "[playit] No se pudo leer el log del agente.",
                );
            }
        }
    });
}

pub fn stop(id: &str) -> AppResult<()> {
    stop_mc(id)?;
    stop_playit_only(id)?;
    let mut g = procs();
    if let Some(map) = g.as_mut() {
        map.remove(id);
    }
    Ok(())
}

/// Detiene todos los servidores locales y túneles playit (al cerrar el launcher).
pub fn stop_all_running() {
    for prof in load_all() {
        let _ = stop_mc_graceful(&prof.id);
        let _ = stop_playit_only(&prof.id);
    }
    let mut g = procs();
    if let Some(map) = g.as_mut() {
        for sp in map.values_mut() {
            if let Some(mut mc) = sp.mc.take() {
                let _ = mc.child.kill();
            }
            if let Some(mut p) = sp.playit.take() {
                let _ = p.kill();
            }
        }
        map.clear();
    }
}

pub fn stop_mc(id: &str) -> AppResult<()> {
    let mut g = procs();
    if let Some(map) = g.as_mut() {
        if let Some(sp) = map.get_mut(id) {
            if let Some(mut mc) = sp.mc.take() {
                let _ = mc.child.kill();
            }
        }
    }
    Ok(())
}

/// Envía `stop` por stdin y espera hasta 45 s antes de forzar kill.
pub fn stop_mc_graceful(id: &str) -> AppResult<()> {
    let _ = send_command(id, "stop");
    for _ in 0..45 {
        if !is_mc_running(id) {
            let mut g = procs();
            if let Some(map) = g.as_mut() {
                if let Some(sp) = map.get_mut(id) {
                    sp.mc = None;
                }
            }
            crate::core::server_console::append(id, "[launcher] Servidor detenido correctamente.");
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    stop_mc(id)
}

pub fn send_command(id: &str, cmd: &str) -> AppResult<()> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return Err(AppError::msg("Comando vacío"));
    }
    let mut g = procs();
    let map = g.as_mut().ok_or_else(|| AppError::msg("Sin procesos"))?;
    let sp = map.get_mut(id).ok_or_else(|| AppError::msg("Servidor no en ejecución"))?;
    let mc = sp.mc.as_mut().ok_or_else(|| AppError::msg("Servidor no en ejecución"))?;
    if mc.child.try_wait().ok().flatten().is_some() {
        return Err(AppError::msg("Servidor no en ejecución"));
    }
    let stdin = mc.stdin.get_mut().unwrap();
    let Some(stdin) = stdin.as_mut() else {
        return Err(AppError::msg("Consola no disponible (Forge run.bat)"));
    };
    use std::io::Write;
    stdin
        .write_all(format!("{cmd}\n").as_bytes())
        .map_err(|e| AppError::msg(format!("No se pudo enviar comando: {e}")))?;
    stdin.flush().ok();
    crate::core::server_console::append(id, &format!("> {cmd}"));
    Ok(())
}

pub fn get_log(id: &str, max_lines: usize) -> Vec<String> {
    crate::core::server_console::get_lines(id, max_lines)
}

pub fn stop_playit(id: &str) -> AppResult<()> {
    stop_playit_only(id)
}

/// Inicia playit.gg sin ventana CMD (Windows: CREATE_NO_WINDOW).
pub fn start_playit(id: &str) -> AppResult<String> {
    // Con el plugin de Paper el túnel ya corre dentro de Java: NO hay que lanzar playit.exe
    // (si se lanzan los dos, el daemon se cuelga en "Waiting for frontend secret provisioning").
    if playit_plugin_present(id) {
        crate::core::server_console::append(
            id,
            "[playit] Plugin en plugins/: el túnel arranca con el servidor Paper. Revisá la consola por playit.gg/claim y la IP *.tun.ply.gg. No uses el agente desktop en este modo.",
        );
        return Err(AppError::msg(
            "Este servidor usa el plugin playit-gg. Iniciá el servidor y mirá la consola (claim + IP). Si falló el cupo de agentes, usá «Resetear Playit» y borrá agentes en playit.gg.",
        ));
    }

    let prof = profile_by_id(id)?;
    let dir = folder_for(&prof);
    let playit = find_playit_exe(&dir);
    let playit = playit.ok_or_else(|| {
        AppError::msg("playit.exe no encontrado. Usa «Preparar servidor» para descargarlo.")
    })?;
    stop_playit_only(id)?;
    // Secret/agent.toml compartidos (evita claim nuevo por server).
    let _ = crate::core::playit_shared::apply_to_server(&dir);
    crate::core::server_console::append(id, "[playit] Iniciando túnel playit.gg…");

    let saved_addr = prof
        .playit_address
        .clone()
        .or_else(|| playit_address_from_meta(&dir));
    if let Some(ref saved) = saved_addr {
        crate::core::server_console::append(
            id,
            &format!("[playit] 💾 IP anterior guardada: {saved} (se actualizará si el túnel cambia)"),
        );
    }
    crate::core::server_console::append(
        id,
        "[playit] Esperando conexión con el servidor de playit…",
    );

    // Redirigir stdout+stderr a archivo (pipes bufferizan; playit usa spinner ANSI con \r).
    let log_path = dir.join(".playit-launcher.log");
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .map_err(|e| AppError::msg(format!("No se pudo crear log de playit: {e}")))?;
    let stderr_file = log_file
        .try_clone()
        .map_err(|e| AppError::msg(format!("No se pudo duplicar log de playit: {e}")))?;

    let secret_path = playit_secret_path(&dir);
    let mut cmd = Command::new(&playit);
    cmd.current_dir(&dir)
        .arg("--secret-path")
        .arg(&secret_path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(stderr_file));
    hide_console(&mut cmd);
    let child = cmd
        .spawn()
        .map_err(|e| AppError::msg(format!("No se pudo iniciar playit: {e}")))?;

    let id_tail = id.to_string();
    spawn_playit_log_tail(id_tail.clone(), log_path.clone());
    spawn_playit_early_exit_watch(id_tail, log_path);

    let mut g = procs();
    let map = g.as_mut().unwrap();
    map.entry(id.to_string())
        .or_insert(ServerProcs {
            mc: None,
            playit: None,
        })
        .playit = Some(child);

    Ok("Playit iniciado. Mirá la consola — la dirección aparece en unos segundos.".into())
}

fn stop_playit_only(id: &str) -> AppResult<()> {
    let mut g = procs();
    if let Some(map) = g.as_mut() {
        if let Some(sp) = map.get_mut(id) {
            if let Some(mut p) = sp.playit.take() {
                let _ = p.kill();
            }
        }
    }
    Ok(())
}

fn find_playit_exe(dir: &Path) -> Option<PathBuf> {
    let local = dir.join("playit.exe");
    if local.is_file() {
        return Some(local);
    }
    let global = paths::data_dir().join("playit.exe");
    if global.is_file() {
        return Some(global);
    }
    None
}

fn playit_secret_path(dir: &Path) -> PathBuf {
    dir.join("playit-agent.toml")
}

pub fn playit_available(id: &str) -> bool {
    profile_by_id(id)
        .ok()
        .map(|p| find_playit_exe(&folder_for(&p)).is_some())
        .unwrap_or(false)
}

/// True si el jar del plugin Playit está en `plugins/` (túnel nativo paper).
pub fn playit_plugin_present(id: &str) -> bool {
    let Ok(prof) = profile_by_id(id) else {
        return false;
    };
    let plugins = folder_for(&prof).join("plugins");
    let Ok(rd) = std::fs::read_dir(plugins) else {
        return false;
    };
    rd.flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_ascii_lowercase();
        n.ends_with(".jar") && n.contains("playit")
    })
}

/// Prepara el servidor (jar + mods/plugins según tipo).
pub async fn ensure_server_jar(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    id: &str,
) -> AppResult<PathBuf> {
    let prof = profile_by_id(id)?;
    let dir = folder_for(&prof);
    crate::core::server_setup::prepare(app, client, &prof, &dir).await
}

/// Solo dependencias de launcher ausentes (Playit plugin, ViaVersion…).
/// No re-descarga el jar si ya existe; preserva configs del usuario.
pub async fn soft_ensure_deps(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    id: &str,
) -> AppResult<()> {
    let prof = profile_by_id(id)?;
    let dir = folder_for(&prof);
    crate::core::server_setup::soft_ensure_launcher_deps(client, app, &prof, &dir).await
}

/// Auto-update de plugins: escanea carpeta plugins/ y reporta cuantos hay.
pub fn plugin_count(id: &str) -> u32 {
    let Ok(dir) = folder_for_id(id) else {
        return 0;
    };
    let plugins = dir.join("plugins");
    std::fs::read_dir(plugins)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("jar"))
                .count() as u32
        })
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerContentItem {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub kind: String,
}

pub fn list_content(id: &str) -> AppResult<Vec<ServerContentItem>> {
    let prof = profile_by_id(id)?;
    let dir = folder_for(&prof);
    let is_fabric = prof.server_type.starts_with("fabric");
    let sub = if is_fabric { "mods" } else { "plugins" };
    let folder = dir.join(sub);
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&folder) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("jar") {
                continue;
            }
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            out.push(ServerContentItem {
                name: e.file_name().to_string_lossy().to_string(),
                path: p.to_string_lossy().to_string(),
                size_bytes: size,
                kind: sub.to_string(),
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn open_folder(id: &str) -> AppResult<()> {
    let dir = folder_for_id(id)?;
    open_in_file_manager(&dir)
}

fn open_in_file_manager(path: &Path) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    Ok(())
}

/// Importa una carpeta de servidor existente sin sobrescribir configs del usuario.
///
/// Detecta paper/fabric/forge/neoforge, respeta `server.properties` y datos de
/// plugins/mods, y solo rellena dependencias ausentes al preparar después.
pub fn import_folder(path: &str, name: Option<&str>) -> AppResult<ServerProfile> {
    let path = path.trim();
    let dir = PathBuf::from(path);
    if !dir.is_dir() {
        return Err(AppError::msg("Ruta inválida o no es una carpeta."));
    }
    let has_paper = dir.join("server.jar").is_file();
    let has_fabric = dir.join("fabric-server-launch.jar").is_file();
    let has_forge = dir.join("run.bat").is_file()
        || dir.join("libraries").is_dir()
        || dir
            .join("unix_args.txt")
            .is_file()
        || std::fs::read_dir(&dir)
            .map(|rd| {
                rd.flatten().any(|e| {
                    let n = e.file_name().to_string_lossy().to_ascii_lowercase();
                    n.ends_with(".jar")
                        && (n.contains("forge") || n.contains("neoforge") || n.contains("minecraft_server"))
                })
            })
            .unwrap_or(false);
    if !has_paper && !has_fabric && !has_forge {
        return Err(AppError::msg(
            "No parece un servidor: se espera server.jar, fabric-server-launch.jar o un setup Forge/NeoForge.",
        ));
    }
    let server_type = detect_server_type(&dir);
    let mc_version = detect_mc_version(&dir).unwrap_or_else(|| "?".into());
    let display_name = name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| dir.file_name().unwrap_or_default().to_string_lossy().to_string());

    let id = format!("srv-{}", uuid::Uuid::new_v4());
    let profile = ServerProfile {
        id: id.clone(),
        name: display_name,
        mc_version,
        server_type,
        ram_mb: 4096,
        port: read_port_from_props(&dir).unwrap_or(25565),
        created_at: now(),
        playit_address: playit_address_from_meta(&dir),
        custom_folder: Some(dir.to_string_lossy().to_string()),
    };

    // No tocar configs existentes: solo eula y meta del launcher.
    ensure_eula(&dir)?;
    write_server_meta(&dir, &profile)?;
    let _ = std::fs::create_dir_all(dir.join("plugins"));
    let _ = std::fs::create_dir_all(dir.join("mods"));
    // Marker: la carpeta ya venía lista; prepare debe ser non-destructive.
    let _ = std::fs::write(
        dir.join(".paraguacraft_imported"),
        "imported=1\npreserve_configs=1\n",
    );

    let mut all = load_all();
    all.push(profile.clone());
    save_all(&all)?;
    Ok(profile)
}

fn ensure_eula(dir: &Path) -> AppResult<()> {
    let eula = dir.join("eula.txt");
    if eula.is_file() {
        let content = std::fs::read_to_string(&eula).unwrap_or_default();
        if content.to_lowercase().contains("eula=true") {
            return Ok(());
        }
    }
    std::fs::write(eula, "eula=true\n")?;
    Ok(())
}

fn read_port_from_props(dir: &Path) -> Option<u16> {
    crate::core::server_properties::read(dir)
        .ok()
        .and_then(|p| p.get("server-port").cloned())
        .and_then(|p| p.parse().ok())
}

fn detect_server_type(dir: &Path) -> String {
    if let Some(meta) = config::read_json::<serde_json::Value>(&dir.join("_paragua_srv.json")) {
        if let Some(t) = meta.get("tipo").and_then(|x| x.as_str()) {
            return t.to_string();
        }
    }
    let has_geyser = ["plugins", "mods"].iter().any(|sub| {
        dir.join(sub).is_dir()
            && std::fs::read_dir(dir.join(sub))
                .map(|rd| {
                    rd.flatten().any(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .to_lowercase()
                            .contains("geyser")
                    })
                })
                .unwrap_or(false)
    });
    let fabric = dir.join("fabric-server-launch.jar").is_file();
    if fabric && has_geyser {
        "fabric-geyser".into()
    } else if fabric {
        "fabric".into()
    } else if has_geyser {
        "paper-geyser".into()
    } else if dir.join("run.bat").is_file() || dir.join("libraries").is_dir() {
        // No hay metadata propia (carpeta importada a mano): heurística por nombre de
        // librerías/jars para no confundir Forge con NeoForge (cambia el maven usado
        // al reparar/re-preparar el servidor).
        let is_neoforge = dir.join("libraries").join("net").join("neoforged").is_dir()
            || std::fs::read_dir(&dir)
                .map(|rd| {
                    rd.flatten().any(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .to_lowercase()
                            .contains("neoforge")
                    })
                })
                .unwrap_or(false);
        if is_neoforge { "neoforge".into() } else { "forge".into() }
    } else {
        "paper".into()
    }
}

fn detect_mc_version(dir: &Path) -> Option<String> {
    if let Some(meta) = config::read_json::<serde_json::Value>(&dir.join("_paragua_srv.json")) {
        if let Some(v) = meta.get("version").and_then(|x| x.as_str()) {
            if !v.is_empty() && v != "?" {
                return Some(v.to_string());
            }
        }
    }
    let props = crate::core::server_properties::read(dir).ok()?;
    props.get("version").cloned()
}

#[cfg(target_os = "windows")]
fn hide_console(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_console(_cmd: &mut Command) {}
