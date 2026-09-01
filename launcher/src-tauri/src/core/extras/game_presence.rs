//! Tail no bloqueante de `logs/latest.log` (cualquier loader) → Discord RPC.

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use regex::Regex;

use crate::core::extras::discord_rpc;
use crate::core::extras::server_assets;
use crate::models::AppSettings;

pub struct PresenceCtx {
    pub username: String,
    pub mc_version: String,
    pub loader: String,
    pub profile: String,
    pub game_dir: PathBuf,
    pub launch_server: Option<String>,
    pub settings: AppSettings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Session {
    Menu,
    Singleplayer(Option<String>),
    Lan(Option<String>),
    Friends(Option<String>),
    Remote(String),
    HostingLocal,
}

static CONNECTING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Connecting to (?:the server[, ]+)?([\w.-]+(?::\d+)?)").expect("connecting regex")
});
/// Solo cortes reales de sesión. `Reached end of stream` / `Lost connection`
/// salen en transfers de Hypixel y pisaban el `Connecting to` nuevo.
static DISCONNECT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:Stopping client|Disconnecting from|Quitting(?: the game)?|Stopping!|Connecting aborted)",
    )
    .expect("disconnect regex")
});

/// Tail asíncrono (tokio) del log mientras corre el juego.
pub fn watch(ctx: PresenceCtx, stop: Arc<AtomicBool>) {
    tauri::async_runtime::spawn(async move {
        let mut last_session: Option<Session> = None;
        let mut last_rev = 0u64;
        let mut revision = 0u64;
        let mut launch_hint = ctx.launch_server.clone();
        let mut session = Session::Menu;
        let mut pos = 0u64;
        let mut primed = false;
        let mut interval = tokio::time::interval(Duration::from_millis(750));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        while !stop.load(Ordering::Relaxed) {
            interval.tick().await;
            let log = ctx.game_dir.join("logs").join("latest.log");
            if !primed {
                pos = initial_read_pos(&log);
                primed = true;
            }
            let pos_now = pos;
            let chunk = tokio::task::spawn_blocking(move || read_new_bytes(&log, pos_now)).await;
            if let Ok(Some((new_pos, text))) = chunk {
                pos = new_pos;
                for line in text.lines() {
                    if apply_line(&mut session, line) {
                        revision = revision.wrapping_add(1);
                        launch_hint = None;
                    }
                }
            }
            if last_session.as_ref() != Some(&session) || last_rev != revision {
                last_session = Some(session.clone());
                last_rev = revision;
                let (mode, host) = session_line(&session, launch_hint.as_deref());
                if ctx.settings.discord_rpc {
                    discord_rpc::set_playing_session(
                        &ctx.username,
                        &ctx.mc_version,
                        &ctx.loader,
                        &ctx.profile,
                        Some(&mode),
                        host.as_deref(),
                        ctx.settings.discord_rpc_version,
                        ctx.settings.discord_rpc_time,
                    );
                }
            }
        }
    });
}

fn session_line(session: &Session, launch_hint: Option<&str>) -> (String, Option<String>) {
    match session {
        Session::Remote(host) => (
            format!("Jugando en {}", server_assets::pretty_name(host)),
            Some(host.clone()),
        ),
        Session::HostingLocal => ("Hosteando Servidor Local".into(), None),
        Session::Friends(world) => (with_world("Mundo abierto a amigos", world.as_deref()), None),
        Session::Lan(world) => (with_world("Hosteando LAN", world.as_deref()), None),
        Session::Singleplayer(world) => (with_world("Un jugador", world.as_deref()), None),
        Session::Menu => {
            if crate::core::servers::any_playit_running() {
                return ("Hosteando Servidor Local".into(), None);
            }
            if let Some(addr) = launch_hint.map(str::trim).filter(|s| !s.is_empty()) {
                let (host, _) = crate::core::favorites::parse_address(addr);
                if !host.is_empty() && !is_local_or_tunnel(&host) {
                    return (
                        format!("Conectando a {}…", server_assets::pretty_name(&host)),
                        Some(host),
                    );
                }
            }
            ("En el menú".into(), None)
        }
    }
}

fn with_world(prefix: &str, world: Option<&str>) -> String {
    match world {
        Some(w) => format!("{prefix}: {w}"),
        None => prefix.to_string(),
    }
}

/// `true` si hubo conexión nueva o corte de sesión: hay que pise el RPC entero.
fn apply_line(session: &mut Session, line: &str) -> bool {
    let low_line = line.to_lowercase();
    if low_line.contains("[chat]") {
        return false;
    }
    if let Some(host) = parse_connecting_to(line) {
        // Reset atómico: no arrastrar Minemen/Hypixel/local al destino nuevo.
        *session = Session::Menu;
        *session = if is_local_or_tunnel(&host) {
            Session::HostingLocal
        } else {
            Session::Remote(strip_port(&host))
        };
        return true;
    }

    if DISCONNECT.is_match(line) {
        *session = Session::Menu;
        return true;
    }

    if let Some(world) = world_name_in_line(line) {
        match session {
            Session::Remote(_) | Session::HostingLocal => {}
            Session::Friends(w) => *w = Some(world),
            Session::Lan(w) => *w = Some(world),
            Session::Singleplayer(w) => *w = Some(world),
            Session::Menu => *session = Session::Singleplayer(Some(world)),
        }
    }

    if matches!(session, Session::Remote(_) | Session::HostingLocal) {
        return false;
    }

    let low = line.to_lowercase();
    if is_friend_host_log(&low) {
        let world = session_world(session);
        *session = Session::Friends(world);
        return false;
    }
    if is_lan_host_log(&low) {
        let world = session_world(session);
        *session = Session::Lan(world);
        return false;
    }
    if is_integrated_server(&low) {
        let world = session_world(session);
        *session = Session::Singleplayer(world);
    }
    false
}

fn session_world(session: &Session) -> Option<String> {
    match session {
        Session::Singleplayer(w) | Session::Lan(w) | Session::Friends(w) => w.clone(),
        _ => None,
    }
}

fn parse_connecting_to(line: &str) -> Option<String> {
    let caps = CONNECTING.captures(line)?;
    let raw = caps.get(1)?.as_str().trim().trim_end_matches('.');
    if raw.is_empty() || raw.len() > 128 {
        return None;
    }
    Some(raw.to_string())
}

fn strip_port(host: &str) -> String {
    if let Some((h, _)) = host.rsplit_once(':') {
        if !h.contains(':') {
            return h.to_string();
        }
    }
    host.to_string()
}

fn is_local_or_tunnel(host: &str) -> bool {
    let h = host
        .trim()
        .trim_end_matches('.')
        .split(':')
        .next()
        .unwrap_or(host)
        .to_ascii_lowercase();
    if h == "localhost"
        || h == "127.0.0.1"
        || h == "0.0.0.0"
        || h == "::1"
        || h.contains("playit.gg")
        || h.contains("ply.gg")
        || h.contains("playit.io")
    {
        return true;
    }
    if h.starts_with("192.168.") || h.starts_with("10.") {
        return true;
    }
    if let Some(rest) = h.strip_prefix("172.") {
        if let Some(oct) = rest.split('.').next() {
            if let Ok(n) = oct.parse::<u16>() {
                return (16..=31).contains(&n);
            }
        }
    }
    false
}

fn is_friend_host_log(low: &str) -> bool {
    const MARKERS: &[&str] = &[
        "[essential]",
        "essential sps",
        "starting singleplayer server",
        "friends can now join",
        "invite friends to your world",
        "e4mc",
        "your server is now available at",
        "world-host",
        "worldhost",
        "lan world plug",
        "opened your world to friends",
        "mundo abierto a amigos",
    ];
    MARKERS.iter().any(|m| low.contains(m))
}

fn is_lan_host_log(low: &str) -> bool {
    const MARKERS: &[&str] = &[
        "started serving on",
        "local game hosted on",
        "hosted on port",
        "open to lan",
        "opened to lan",
        "started hosted world",
        "lan server",
        "mundo local abierto",
        "abierto a la lan",
        "iniciado el servidor en el puerto",
        "servidor lan",
    ];
    MARKERS.iter().any(|m| low.contains(m))
}

fn is_integrated_server(low: &str) -> bool {
    low.contains("starting integrated minecraft server")
        || low.contains("integrated server loaded")
        || low.contains("loading world")
}

fn world_name_in_line(line: &str) -> Option<String> {
    if let Some(idx) = line.find("ServerLevel[") {
        let rest = &line[idx + 12..];
        if let Some(end) = rest.find(']') {
            let name = rest[..end].trim();
            if is_plausible_world(name) {
                return Some(name.to_string());
            }
        }
    }
    if let Some(idx) = line.find("Joined world '") {
        let rest = &line[idx + 14..];
        if let Some(end) = rest.find('\'') {
            let name = rest[..end].trim();
            if is_plausible_world(name) {
                return Some(name.to_string());
            }
        }
    }
    if let Some(idx) = line.find("Saving chunks for level '") {
        let rest = &line[idx + 25..];
        if let Some(end) = rest.find('\'') {
            let name = rest[..end].trim();
            if is_plausible_world(name) {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn is_plausible_world(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty()
        && n.len() < 48
        && !n.contains("minecraft:")
        && !n.eq_ignore_ascii_case("overworld")
        && !n.eq_ignore_ascii_case("the_nether")
        && !n.eq_ignore_ascii_case("the_end")
}

fn initial_read_pos(path: &Path) -> u64 {
    let len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    const MAX_CATCHUP: u64 = 512 * 1024;
    if len > MAX_CATCHUP {
        len - MAX_CATCHUP
    } else {
        0
    }
}

/// Lectura compartida en Windows: MC mantiene `latest.log` abierto con write share.
fn open_log_shared(path: &Path) -> std::io::Result<std::fs::File> {
    let mut opts = OpenOptions::new();
    opts.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x1;
        const FILE_SHARE_WRITE: u32 = 0x2;
        const FILE_SHARE_DELETE: u32 = 0x4;
        opts.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE);
    }
    opts.open(path)
}

fn read_new_bytes(path: &Path, pos: u64) -> Option<(u64, String)> {
    let mut f = open_log_shared(path).ok()?;
    let len = f.metadata().ok()?.len();
    let mut pos = pos;
    if len < pos {
        pos = initial_read_pos(path);
    }
    if len == pos {
        return None;
    }
    f.seek(SeekFrom::Start(pos)).ok()?;
    let mut buf = String::new();
    f.read_to_string(&mut buf).ok()?;
    Some((len, buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replay(lines: &str) -> Session {
        let mut s = Session::Menu;
        for line in lines.lines() {
            apply_line(&mut s, line);
        }
        s
    }

    #[test]
    fn remote_hypixel_sticky_after_chat_spam() {
        let mut log = String::from("[Client thread/INFO]: Connecting to mc.hypixel.net, 25565\n");
        for i in 0..2000 {
            log.push_str(&format!("[Client thread/INFO]: [CHAT] lobby spam {i}\n"));
        }
        assert_eq!(replay(&log), Session::Remote("mc.hypixel.net".into()));
    }

    #[test]
    fn regex_connecting_host_port() {
        assert_eq!(
            parse_connecting_to("[Render thread/INFO]: Connecting to mc.hypixel.net:25565"),
            Some("mc.hypixel.net:25565".into())
        );
        assert_eq!(
            parse_connecting_to("[Client thread/INFO]: Connecting to mc.hypixel.net, 25565"),
            Some("mc.hypixel.net".into())
        );
    }

    #[test]
    fn playit_is_local_hosting() {
        let s = replay("[INFO]: Connecting to abc123.playit.gg, 25565\n");
        assert_eq!(s, Session::HostingLocal);
        let s = replay("[INFO]: Connecting to 127.0.0.1:25565\n");
        assert_eq!(s, Session::HostingLocal);
    }

    #[test]
    fn disconnect_returns_to_menu() {
        let log = "[Client thread/INFO]: Connecting to mc.hypixel.net, 25565\n\
                   [Client thread/INFO]: [CHAT] hi\n\
                   [Client thread/INFO]: Disconnecting from server\n";
        assert_eq!(replay(log), Session::Menu);
    }

    #[test]
    fn stream_end_does_not_drop_multiplayer() {
        let log = "[Client thread/INFO]: Connecting to mc.hypixel.net, 25565\n\
                   [Client thread/INFO]: Reached end of stream.\n";
        assert_eq!(replay(log), Session::Remote("mc.hypixel.net".into()));
    }

    #[test]
    fn hot_swap_minemen_to_hypixel_is_atomic() {
        let log = "[INFO]: Connecting to na.minemen.club, 25565\n\
                   [INFO]: Disconnecting from server\n\
                   [INFO]: Connecting to mc.hypixel.net, 25565\n\
                   [INFO]: Reached end of stream.\n\
                   [INFO]: Lost connection: Timed out\n";
        assert_eq!(replay(log), Session::Remote("mc.hypixel.net".into()));
    }

    #[test]
    fn connecting_overwrites_without_disconnect_line() {
        let log = "[INFO]: Connecting to na.minemen.club, 25565\n\
                   [INFO]: Connecting to mc.hypixel.net, 25565\n";
        assert_eq!(replay(log), Session::Remote("mc.hypixel.net".into()));
    }

    #[test]
    fn connecting_to_local_clears_remote() {
        let log = "[INFO]: Connecting to mc.hypixel.net, 25565\n\
                   [INFO]: Connecting to 127.0.0.1, 25565\n";
        assert_eq!(replay(log), Session::HostingLocal);
    }

    #[test]
    fn quitting_returns_to_menu() {
        let log = "[INFO]: Connecting to na.minemen.club, 25565\n[INFO]: Quitting\n";
        assert_eq!(replay(log), Session::Menu);
    }

    #[test]
    fn stopping_client_returns_to_menu() {
        let log = "[INFO]: Connecting to mc.hypixel.net, 25565\n[INFO]: Stopping client\n";
        assert_eq!(replay(log), Session::Menu);
    }

    #[test]
    fn lan_open() {
        let log = "[Server thread/INFO]: Started serving on 25565\n";
        assert_eq!(replay(log), Session::Lan(None));
    }

    #[test]
    fn essential_invite() {
        let log = "[Essential] Friends can now join your world\n";
        assert_eq!(replay(log), Session::Friends(None));
    }

    #[test]
    fn e4mc_link() {
        let log = "[e4mc] Your server is now available at abc.e4mc.link\n";
        assert_eq!(replay(log), Session::Friends(None));
    }

    #[test]
    fn singleplayer_world_name() {
        let log = "[Server thread/INFO]: Starting integrated minecraft server\n\
                   [Server thread/INFO]: Saving chunks for level 'Mi Mundo'/Overworld\n";
        assert_eq!(replay(log), Session::Singleplayer(Some("Mi Mundo".into())));
    }

    #[test]
    fn saving_chunks_does_not_drop_multiplayer() {
        let log = "[Client thread/INFO]: Connecting to mc.hypixel.net, 25565\n\
                   [Server thread/INFO]: Saving chunks for level 'New World'/Overworld\n";
        assert_eq!(replay(log), Session::Remote("mc.hypixel.net".into()));
    }

    #[test]
    fn forge_189_connecting_format() {
        let line = "[Client thread/INFO]: Connecting to mc.hypixel.net, 25565";
        assert_eq!(parse_connecting_to(line).as_deref(), Some("mc.hypixel.net"));
    }
}
