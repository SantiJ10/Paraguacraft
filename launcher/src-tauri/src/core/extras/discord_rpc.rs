//! Discord Rich Presence (mismo APP ID que el launcher Python).

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::activity::{Activity, Timestamps};
use discord_rich_presence::DiscordIpc;
use discord_rich_presence::DiscordIpcClient;

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
        drop(guard);
        set_launcher_idle("Paraguacraft");
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
        "En el launcher",
        &format!("Conectado como {username}"),
        true,
        true,
    );
}

pub fn set_browsing(username: &str) {
    update(
        "Explorando el launcher",
        &format!("Conectado como {username}"),
        true,
        true,
    );
}

pub fn set_playing(username: &str, mc_version: &str, loader: &str, show_version: bool, show_time: bool) {
    let state = if show_version {
        format!("{mc_version} · {loader}")
    } else {
        "Jugando Minecraft".into()
    };
    update(
        &state,
        &format!("Jugando como {username}"),
        show_time,
        show_version,
    );
}

pub fn set_bedrock_playing(username: &str, show_time: bool) {
    update(
        "Paraguacraft Bedrock",
        &format!("Jugando como {username}"),
        show_time,
        false,
    );
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
