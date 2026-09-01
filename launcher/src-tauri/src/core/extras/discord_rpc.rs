//! Discord Rich Presence (mismo APP ID que el launcher Python).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use discord_rich_presence::activity::{Activity, ActivityType, Assets, Button, Timestamps};
use discord_rich_presence::DiscordIpc;
use discord_rich_presence::DiscordIpcClient;
use uuid::Uuid;

use crate::core::extras::server_assets::{self, RpcArt};
use crate::core::loaders;

const APP_ID: &str = "1487516329631154206";
const DOWNLOAD_URL: &str = "https://paraguacraft.pages.dev";

static CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);
static SESSION_START: Mutex<Option<i64>> = Mutex::new(None);
static WATCHDOG: AtomicBool = AtomicBool::new(false);
static SHUTDOWN: AtomicBool = AtomicBool::new(false);
static GAME_PID: AtomicU32 = AtomicU32::new(0);
static LAST: Mutex<Option<PresenceSnap>> = Mutex::new(None);

#[derive(Clone)]
struct PresenceSnap {
    details: String,
    state: String,
    show_time: bool,
    large_image: Option<String>,
    large_text: Option<String>,
    small_image: Option<String>,
    small_text: Option<String>,
}

/// Vincula el RPC / overlay de Discord al PID de `javaw` (no al del launcher).
pub fn bind_game_pid(pid: u32) {
    GAME_PID.store(pid, Ordering::SeqCst);
}

pub fn clear_game_pid() {
    GAME_PID.store(0, Ordering::SeqCst);
}

fn activity_pid() -> u32 {
    let p = GAME_PID.load(Ordering::SeqCst);
    if p == 0 {
        std::process::id()
    } else {
        p
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn spawn_watchdog() {
    if WATCHDOG.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::Builder::new()
        .name("discord-rpc-watch".into())
        .spawn(|| loop {
            std::thread::sleep(Duration::from_secs(40));
            if SHUTDOWN.load(Ordering::SeqCst) {
                return;
            }
            let settings: crate::models::AppSettings =
                crate::config::read_json(&crate::core::paths::config_file()).unwrap_or_default();
            if !settings.discord_rpc {
                continue;
            }
            let disconnected = CLIENT.lock().map(|g| g.is_none()).unwrap_or(true);
            if disconnected {
                connect(true);
                if let Some(snap) = LAST.lock().ok().and_then(|g| g.clone()) {
                    apply(&snap);
                } else if let Some(acc) = crate::core::accounts::active_account() {
                    set_launcher_idle(&acc.username.replace(" [PREMIUM]", ""));
                }
            }
        })
        .ok();
}

pub fn connect(enabled: bool) {
    if SHUTDOWN.load(Ordering::SeqCst) {
        return;
    }
    if !enabled {
        disconnect();
        return;
    }
    spawn_watchdog();
    {
        let guard = match CLIENT.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if guard.is_some() {
            return;
        }
    }
    let mut client = DiscordIpcClient::new(APP_ID);
    if client.connect().is_err() {
        return;
    }
    let mut guard = match CLIENT.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if guard.is_some() {
        drop(guard);
        std::thread::spawn(move || {
            let _ = client.close();
        });
        return;
    }
    if let Ok(mut start) = SESSION_START.lock() {
        if start.is_none() {
            *start = Some(now_secs());
        }
    }
    *guard = Some(client);
}

pub fn disconnect() {
    take_client_and_close();
    if let Ok(mut start) = SESSION_START.lock() {
        *start = None;
    }
    if let Ok(mut last) = LAST.lock() {
        *last = None;
    }
}

/// Cierra Discord IPC sin bloquear el hilo de la UI (el pipe puede colgarse).
pub fn shutdown() {
    SHUTDOWN.store(true, Ordering::SeqCst);
    take_client_and_close();
}

fn take_client_and_close() {
    let client = match CLIENT.try_lock() {
        Ok(mut g) => g.take(),
        Err(_) => {
            std::thread::sleep(Duration::from_millis(50));
            CLIENT.try_lock().ok().and_then(|mut g| g.take())
        }
    };
    if let Some(mut c) = client {
        std::thread::spawn(move || {
            let _ = c.clear_activity();
            let _ = c.close();
        });
    }
}

pub fn set_launcher_idle(_username: &str) {
    update(
        "Navegando por el launcher",
        "Preparando la configuración",
        true,
        None,
        None,
        None,
        None,
    );
}

pub fn set_exploring_settings(username: &str) {
    update(
        "Navegando por el launcher",
        &format!("En ajustes · {username}"),
        true,
        None,
        None,
        None,
        None,
    );
}

/// RPC al iniciar el juego (antes de detectar mundo/servidor).
pub fn set_playing(
    username: &str,
    mc_version: &str,
    loader: &str,
    profile: &str,
    show_version: bool,
    show_time: bool,
) {
    let details = playing_details(username, mc_version, loader, profile, show_version);
    let art = server_assets::art_for_host(None);
    let (small, small_txt) = small_for_loader(loader);
    update(
        &details,
        "En el menú",
        show_time,
        Some(&art.large_image),
        Some(&art.large_text),
        small.or(art.small_image.as_deref()),
        small_txt.or(art.small_text.as_deref()),
    );
}

/// Formato in-game: `{user} - {version} ({loader}) - {perfil}` + servidor/mundo/menu en state.
pub fn set_playing_session(
    username: &str,
    mc_version: &str,
    loader: &str,
    profile: &str,
    mode_line: Option<&str>,
    host: Option<&str>,
    show_version: bool,
    show_time: bool,
) {
    let details = playing_details(username, mc_version, loader, profile, show_version);
    let state = mode_line
        .filter(|s| !s.is_empty())
        .unwrap_or("En el menú")
        .to_string();
    let art = art_for_session(&state, host, loader);
    update(
        &details,
        &state,
        show_time,
        Some(&art.large_image),
        Some(&art.large_text),
        art.small_image.as_deref(),
        art.small_text.as_deref(),
    );
}

fn art_for_session(state: &str, host: Option<&str>, loader: &str) -> RpcArt {
    let s = state.to_lowercase();
    let hosting = s.contains("hosteando") || s.contains("abierto a amigos") || s.contains(" lan");
    if let Some(host) = host.filter(|h| !h.is_empty()) {
        let mut art = server_assets::art_for_host(Some(host));
        if art.small_image.is_none() {
            let (img, txt) = small_for_loader(loader);
            art.small_image = img.map(|s| s.to_string());
            art.small_text = txt.map(|s| s.to_string());
        }
        return art;
    }
    let mut art = server_assets::art_for_host(None);
    if hosting {
        art.small_image = Some("hosting".into());
        art.small_text = Some("Hosteando".into());
    } else {
        let (img, txt) = small_for_loader(loader);
        art.small_image = img.map(|s| s.to_string());
        art.small_text = txt.map(|s| s.to_string());
    }
    art
}

fn playing_details(
    username: &str,
    mc_version: &str,
    loader: &str,
    profile: &str,
    show_version: bool,
) -> String {
    let loader_label = loaders::display_label(loader);
    let profile = profile.trim();
    if show_version {
        if profile.is_empty() {
            format!("{username} - {mc_version} ({loader_label})")
        } else {
            format!("{username} - {mc_version} ({loader_label}) - {profile}")
        }
    } else if profile.is_empty() {
        format!("{username} - {loader_label}")
    } else {
        format!("{username} - {loader_label} - {profile}")
    }
}

fn small_for_loader(loader: &str) -> (Option<&'static str>, Option<&'static str>) {
    let l = loader.to_ascii_lowercase();
    if l.contains("pvp") {
        (Some("pvp"), Some("PvP"))
    } else {
        (Some("play"), Some("En juego"))
    }
}

/// RPC al detectar el proceso Bedrock (antes de leer ventana).
pub fn set_bedrock_loading(username: &str, show_time: bool) {
    let art = server_assets::art_for_host(None);
    update(
        &format!("{username} - Bedrock Edition"),
        "En el menú",
        show_time,
        Some(&art.large_image),
        Some(&art.large_text),
        Some("play"),
        Some("Bedrock"),
    );
}

/// RPC in-game Bedrock: `{user} - Bedrock Edition` + menú/mundo en state.
pub fn set_bedrock_session(username: &str, mode_line: Option<&str>, show_time: bool) {
    let state = mode_line
        .filter(|s| !s.is_empty())
        .unwrap_or("En el menú")
        .to_string();
    let art = server_assets::art_for_host(None);
    update(
        &format!("{username} - Bedrock Edition"),
        &state,
        show_time,
        Some(&art.large_image),
        Some(&art.large_text),
        Some("play"),
        Some("Bedrock"),
    );
}

/// Actualiza RPC según pantalla del launcher (idle / settings). No pisa juego activo.
pub fn set_discord_rpc_screen(screen: &str) {
    if crate::core::game_session::is_running() {
        return;
    }
    let settings: crate::models::AppSettings =
        crate::config::read_json(&crate::core::paths::config_file()).unwrap_or_default();
    if !settings.discord_rpc {
        return;
    }
    connect(true);
    let user = crate::core::accounts::active_account()
        .map(|a| a.username.replace(" [PREMIUM]", ""))
        .unwrap_or_default();
    if screen == "settings" {
        set_exploring_settings(&user);
    } else {
        set_launcher_idle(&user);
    }
}

fn update(
    details: &str,
    state: &str,
    show_time: bool,
    large_image: Option<&str>,
    large_text: Option<&str>,
    small_image: Option<&str>,
    small_text: Option<&str>,
) {
    if SHUTDOWN.load(Ordering::SeqCst) {
        return;
    }
    let snap = PresenceSnap {
        details: details.to_string(),
        state: state.to_string(),
        show_time,
        large_image: large_image.map(|s| s.to_string()),
        large_text: large_text.map(|s| s.to_string()),
        small_image: small_image.map(|s| s.to_string()),
        small_text: small_text.map(|s| s.to_string()),
    };
    if let Ok(mut last) = LAST.lock() {
        *last = Some(snap.clone());
    }
    if !apply(&snap) {
        if let Ok(mut guard) = CLIENT.lock() {
            *guard = None;
        }
        connect(true);
        let _ = apply(&snap);
    }
}

fn apply(snap: &PresenceSnap) -> bool {
    if SHUTDOWN.load(Ordering::SeqCst) {
        return false;
    }
    let mut guard = match CLIENT.lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    let Some(client) = guard.as_mut() else {
        return false;
    };
    let mut assets = Assets::new();
    if let Some(img) = snap.large_image.as_deref() {
        assets = assets.large_image(img);
    }
    if let Some(txt) = snap.large_text.as_deref() {
        assets = assets.large_text(txt);
    } else {
        assets = assets.large_text("Paraguacraft");
    }
    if let (Some(img), Some(txt)) = (snap.small_image.as_deref(), snap.small_text.as_deref()) {
        assets = assets.small_image(img).small_text(txt);
    }
    let mut act = Activity::new()
        .activity_type(ActivityType::Playing)
        .details(snap.details.as_str())
        .state(snap.state.as_str())
        .assets(assets)
        .buttons(vec![Button::new("Descargar Launcher", DOWNLOAD_URL)]);
    if snap.show_time {
        if let Ok(start) = SESSION_START.lock() {
            if let Some(start) = *start {
                act = act.timestamps(Timestamps::new().start(start));
            }
        }
    }
    set_activity_with_pid(client, act, activity_pid())
}

/// `discord-rich-presence` manda `std::process::id()` (el launcher). El overlay
/// de Discord engancha el OpenGL de `javaw` solo si el PID del payload es el del juego.
fn set_activity_with_pid(client: &mut DiscordIpcClient, activity: Activity<'_>, pid: u32) -> bool {
    let data = serde_json::json!({
        "cmd": "SET_ACTIVITY",
        "args": {
            "pid": pid,
            "activity": activity
        },
        "nonce": Uuid::new_v4().to_string()
    });
    client.send(data, 1).is_ok()
}

pub fn clear_activity() {
    if let Ok(mut guard) = CLIENT.lock() {
        if let Some(client) = guard.as_mut() {
            let data = serde_json::json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": activity_pid(),
                    "activity": null
                },
                "nonce": Uuid::new_v4().to_string()
            });
            let _ = client.send(data, 1);
        }
    }
    if let Ok(mut last) = LAST.lock() {
        *last = None;
    }
}
