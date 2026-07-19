//! Discord Rich Presence (mismo APP ID que el launcher Python).

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::activity::{Activity, Timestamps};
use discord_rich_presence::DiscordIpc;
use discord_rich_presence::DiscordIpcClient;

use crate::core::loaders;

const APP_ID: &str = "1487516329631154206";

static CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);
static SESSION_START: Mutex<Option<i64>> = Mutex::new(None);

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn connect(enabled: bool) {
    if !enabled {
        disconnect();
        return;
    }
    let mut guard = CLIENT.lock().unwrap();
    if guard.is_some() {
        return;
    }
    let mut client = DiscordIpcClient::new(APP_ID);
    if client.connect().is_ok() {
        *SESSION_START.lock().unwrap() = Some(now_secs());
        *guard = Some(client);
    }
}

pub fn disconnect() {
    let mut guard = CLIENT.lock().unwrap();
    if let Some(mut c) = guard.take() {
        let _ = c.close();
    }
    *SESSION_START.lock().unwrap() = None;
}

pub fn set_launcher_idle(username: &str) {
    update(
        "Paraguacraft",
        &format!("Conectado como {username}"),
        true,
        true,
    );
}

pub fn set_exploring_settings(username: &str) {
    update(
        "Paraguacraft",
        &format!("Explorando Ajustes · {username}"),
        true,
        true,
    );
}

/// RPC al iniciar el juego (antes de detectar mundo/servidor).
pub fn set_playing(
    username: &str,
    mc_version: &str,
    loader: &str,
    show_version: bool,
    show_time: bool,
) {
    let loader_label = loaders::display_label(loader);
    let details = format!("Jugando como {username}");
    let state = if show_version {
        format!("{mc_version} - {loader_label}")
    } else {
        loader_label
    };
    update(&details, &state, show_time, show_version);
}

/// Formato in-game: `{user} - {version} - {loader}` + servidor/mundo/menu en state.
pub fn set_playing_session(
    username: &str,
    mc_version: &str,
    loader: &str,
    mode_line: Option<&str>,
    show_version: bool,
    show_time: bool,
) {
    let loader_label = loaders::display_label(loader);
    let details = if show_version {
        format!("{username} - {mc_version} - {loader_label}")
    } else {
        format!("{username} - {loader_label}")
    };
    let state = mode_line
        .filter(|s| !s.is_empty())
        .unwrap_or("En el menú")
        .to_string();
    update(&details, &state, show_time, show_version);
}

/// RPC al detectar el proceso Bedrock (antes de leer ventana).
pub fn set_bedrock_loading(username: &str, show_time: bool) {
    update(
        &format!("Jugando como {username}"),
        "Bedrock Edition",
        show_time,
        false,
    );
}

/// RPC in-game Bedrock: `{user} - Bedrock Edition` + menú/mundo en state.
pub fn set_bedrock_session(username: &str, mode_line: Option<&str>, show_time: bool) {
    let details = format!("{username} - Bedrock Edition");
    let state = mode_line
        .filter(|s| !s.is_empty())
        .unwrap_or("En el menú")
        .to_string();
    update(&details, &state, show_time, false);
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

fn update(details: &str, state: &str, show_time: bool, _show_version: bool) {
    let mut guard = CLIENT.lock().unwrap();
    let Some(client) = guard.as_mut() else {
        return;
    };
    let mut act = Activity::new().details(details).state(state);
    if show_time {
        if let Some(start) = *SESSION_START.lock().unwrap() {
            act = act.timestamps(Timestamps::new().start(start));
        }
    }
    let _ = client.set_activity(act);
}

pub fn clear_activity() {
    let mut guard = CLIENT.lock().unwrap();
    if let Some(client) = guard.as_mut() {
        let _ = client.clear_activity();
    }
}
