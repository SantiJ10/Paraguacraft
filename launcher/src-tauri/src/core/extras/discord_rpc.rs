//! Discord Rich Presence (mismo APP ID que el launcher Python).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use discord_rich_presence::activity::{Activity, ActivityType, Assets, Button, Timestamps};
use discord_rich_presence::DiscordIpc;
use discord_rich_presence::DiscordIpcClient;

use crate::core::loaders;

const APP_ID: &str = "1487516329631154206";
const DOWNLOAD_URL: &str = "https://paraguacraft.pages.dev";

static CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);
static SESSION_START: Mutex<Option<i64>> = Mutex::new(None);
static WATCHDOG: AtomicBool = AtomicBool::new(false);
static LAST: Mutex<Option<PresenceSnap>> = Mutex::new(None);

#[derive(Clone)]
struct PresenceSnap {
    details: String,
    state: String,
    show_time: bool,
    small_image: Option<String>,
    small_text: Option<String>,
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
            let settings: crate::models::AppSettings =
                crate::config::read_json(&crate::core::paths::config_file()).unwrap_or_default();
            if !settings.discord_rpc {
                continue;
            }
            let disconnected = CLIENT.lock().unwrap().is_none();
            if disconnected {
                connect(true);
                if let Some(snap) = LAST.lock().unwrap().clone() {
                    apply(&snap);
                } else if let Some(acc) = crate::core::accounts::active_account() {
                    set_launcher_idle(&acc.username.replace(" [PREMIUM]", ""));
                }
            }
        })
        .ok();
}

pub fn connect(enabled: bool) {
    if !enabled {
        disconnect();
        return;
    }
    spawn_watchdog();
    let mut guard = CLIENT.lock().unwrap();
    if guard.is_some() {
        return;
    }
    let mut client = DiscordIpcClient::new(APP_ID);
    if client.connect().is_ok() {
        if SESSION_START.lock().unwrap().is_none() {
            *SESSION_START.lock().unwrap() = Some(now_secs());
        }
        *guard = Some(client);
    }
}

pub fn disconnect() {
    let mut guard = CLIENT.lock().unwrap();
    if let Some(mut c) = guard.take() {
        let _ = c.clear_activity();
        let _ = c.close();
    }
    *SESSION_START.lock().unwrap() = None;
    *LAST.lock().unwrap() = None;
}

pub fn set_launcher_idle(_username: &str) {
    update(
        "Navegando por el launcher",
        "Preparando la configuración",
        true,
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
    let small = small_for_loader(loader);
    update(&details, "En el menú", show_time, small.0, small.1);
}

/// Formato in-game: `{user} - {version} ({loader}) - {perfil}` + servidor/mundo/menu en state.
pub fn set_playing_session(
    username: &str,
    mc_version: &str,
    loader: &str,
    profile: &str,
    mode_line: Option<&str>,
    show_version: bool,
    show_time: bool,
) {
    let details = playing_details(username, mc_version, loader, profile, show_version);
    let state = mode_line
        .filter(|s| !s.is_empty())
        .unwrap_or("En el menú")
        .to_string();
    let hosting = {
        let s = state.to_lowercase();
        s.contains("hosteando") || s.contains("abierto a amigos") || s.contains(" lan")
    };
    let (img, txt) = if hosting {
        (Some("hosting"), Some("Hosteando"))
    } else {
        small_for_loader(loader)
    };
    update(&details, &state, show_time, img, txt);
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
    update(
        &format!("{username} - Bedrock Edition"),
        "En el menú",
        show_time,
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
    update(
        &format!("{username} - Bedrock Edition"),
        &state,
        show_time,
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
    small_image: Option<&str>,
    small_text: Option<&str>,
) {
    let snap = PresenceSnap {
        details: details.to_string(),
        state: state.to_string(),
        show_time,
        small_image: small_image.map(|s| s.to_string()),
        small_text: small_text.map(|s| s.to_string()),
    };
    *LAST.lock().unwrap() = Some(snap.clone());
    if !apply(&snap) {
        let mut guard = CLIENT.lock().unwrap();
        *guard = None;
        drop(guard);
        connect(true);
        let _ = apply(&snap);
    }
}

fn apply(snap: &PresenceSnap) -> bool {
    let mut guard = CLIENT.lock().unwrap();
    let Some(client) = guard.as_mut() else {
        return false;
    };
    let mut assets = Assets::new().large_text("Paraguacraft");
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
        if let Some(start) = *SESSION_START.lock().unwrap() {
            act = act.timestamps(Timestamps::new().start(start));
        }
    }
    client.set_activity(act).is_ok()
}

pub fn clear_activity() {
    let mut guard = CLIENT.lock().unwrap();
    if let Some(client) = guard.as_mut() {
        let _ = client.clear_activity();
    }
    *LAST.lock().unwrap() = None;
}
