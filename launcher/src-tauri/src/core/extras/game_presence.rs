//! Detecta mundo / servidor en juego (latest.log incremental) y actualiza Discord RPC.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::core::extras::discord_rpc;
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
}

/// Hilo ligero: lee `latest.log` por incrementos y refresca el RPC mientras el juego corre.
pub fn watch(ctx: PresenceCtx, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let mut last_mode = String::new();
        let mut session = Session::Menu;
        let mut pos = 0u64;
        let mut primed = false;
        while !stop.load(Ordering::Relaxed) {
            let log = ctx.game_dir.join("logs").join("latest.log");
            if !primed {
                pos = initial_read_pos(&log);
                primed = true;
            }
            if let Some(chunk) = read_new_bytes(&log, &mut pos) {
                for line in chunk.lines() {
                    apply_line(&mut session, line);
                }
            }
            let mode = session_line(&session, &ctx);
            if mode != last_mode {
                last_mode = mode.clone();
                if ctx.settings.discord_rpc {
                    discord_rpc::set_playing_session(
                        &ctx.username,
                        &ctx.mc_version,
                        &ctx.loader,
                        &ctx.profile,
                        Some(&mode),
                        ctx.settings.discord_rpc_version,
                        ctx.settings.discord_rpc_time,
                    );
                }
            }
            std::thread::sleep(Duration::from_millis(800));
        }
    });
}

fn session_line(session: &Session, ctx: &PresenceCtx) -> String {
    match session {
        Session::Remote(host) => format!("Jugando en {}", pretty_host(host)),
        Session::Friends(world) => with_world("Mundo abierto a amigos", world.as_deref()),
        Session::Lan(world) => with_world("Hosteando LAN", world.as_deref()),
        Session::Singleplayer(world) => with_world("Un jugador", world.as_deref()),
        Session::Menu => {
            if crate::core::servers::any_playit_running() {
                return "Hosteando Servidor para amigos".into();
            }
            if let Some(addr) = ctx.launch_server.as_deref().map(str::trim).filter(|s| !s.is_empty())
            {
                let (host, _) = crate::core::favorites::parse_address(addr);
                if !host.is_empty() && !is_local_host(&host) {
                    return format!("Conectando a {}…", pretty_host(&host));
                }
            }
            "En el menú".into()
        }
    }
}

fn with_world(prefix: &str, world: Option<&str>) -> String {
    match world {
        Some(w) => format!("{prefix}: {w}"),
        None => prefix.to_string(),
    }
}

fn apply_line(session: &mut Session, line: &str) {
    let low_line = line.to_lowercase();
    if low_line.contains("[chat]") {
        return;
    }
    if let Some(host) = parse_connecting_to(line) {
        if is_local_host(&host) {
            if !matches!(session, Session::Remote(_)) {
                let world = match session {
                    Session::Lan(w) | Session::Friends(w) | Session::Singleplayer(w) => w.clone(),
                    _ => None,
                };
                *session = Session::Lan(world);
            }
        } else {
            *session = Session::Remote(host);
        }
        return;
    }

    let low = line.to_lowercase();
    if is_disconnect(&low) {
        if matches!(
            session,
            Session::Remote(_) | Session::Lan(_) | Session::Friends(_)
        ) {
            *session = Session::Menu;
        }
        return;
    }

    if let Some(world) = world_name_in_line(line) {
        match session {
            Session::Remote(_) => {}
            Session::Friends(w) => *w = Some(world),
            Session::Lan(w) => *w = Some(world),
            Session::Singleplayer(w) => *w = Some(world),
            Session::Menu => *session = Session::Singleplayer(Some(world)),
        }
    }

    if matches!(session, Session::Remote(_)) {
        return;
    }

    if is_friend_host_log(&low) {
        let world = session_world(session);
        *session = Session::Friends(world);
        return;
    }
    if is_lan_host_log(&low) {
        let world = session_world(session);
        *session = Session::Lan(world);
        return;
    }
    if is_integrated_server(&low) {
        let world = session_world(session);
        *session = Session::Singleplayer(world);
    }
}

fn session_world(session: &Session) -> Option<String> {
    match session {
        Session::Singleplayer(w) | Session::Lan(w) | Session::Friends(w) => w.clone(),
        _ => None,
    }
}

fn parse_connecting_to(line: &str) -> Option<String> {
    let idx = line.find("Connecting to ")?;
    let rest = line[idx + 14..].trim();
    let token = rest
        .split([',', ' ', '\t'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_end_matches('.')
        .trim_end_matches(':');
    let host = token.split(':').next().unwrap_or(token).trim();
    if host.is_empty() || host.contains('/') || host.len() > 128 {
        return None;
    }
    Some(host.to_string())
}

fn is_disconnect(low: &str) -> bool {
    const MARKERS: &[&str] = &[
        "disconnected from",
        "lost connection: ",
        "connection closed",
        "reached end of stream",
        "stopping!",
        "connecting aborted",
        "el servidor cerró la conexión",
        "conexion perdida",
        "conexión perdida",
    ];
    MARKERS.iter().any(|m| low.contains(m))
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

fn is_local_host(host: &str) -> bool {
    let h = host.trim().trim_end_matches('.').to_ascii_lowercase();
    h == "localhost"
        || h == "127.0.0.1"
        || h == "0.0.0.0"
        || h == "::1"
        || h.starts_with("192.168.")
        || h.starts_with("10.")
        || h.starts_with("172.16.")
}

fn pretty_host(host: &str) -> String {
    let h = host.trim().trim_end_matches('.').to_ascii_lowercase();
    const MAP: &[(&str, &str)] = &[
        ("hypixel.net", "Hypixel"),
        ("minemen.club", "Minemen Club"),
        ("cubecraft.net", "CubeCraft"),
        ("universocraft.net", "UniversoCraft"),
        ("mushmc.com.br", "Mush"),
        ("mush.com.br", "Mush"),
        ("regorland.net", "Regorland"),
        ("hylex.net", "Hylex"),
        ("bedwarspractice.club", "Bedwars Practice"),
    ];
    for (suffix, name) in MAP {
        if h == *suffix || h.ends_with(&format!(".{suffix}")) {
            return (*name).to_string();
        }
    }
    host.trim().trim_end_matches('.').to_string()
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

fn read_new_bytes(path: &Path, pos: &mut u64) -> Option<String> {
    let mut f = File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    if len < *pos {
        *pos = initial_read_pos(path);
    }
    if len == *pos {
        return None;
    }
    f.seek(SeekFrom::Start(*pos)).ok()?;
    let mut buf = String::new();
    f.read_to_string(&mut buf).ok()?;
    *pos = len;
    Some(buf)
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
        assert_eq!(
            replay(&log),
            Session::Remote("mc.hypixel.net".into())
        );
    }

    #[test]
    fn pretty_hypixel_name() {
        assert_eq!(pretty_host("mc.hypixel.net"), "Hypixel");
        assert_eq!(pretty_host("na.minemen.club"), "Minemen Club");
    }

    #[test]
    fn ignores_localhost_lan() {
        assert_eq!(parse_connecting_to("[INFO]: Connecting to localhost, 25565"), Some("localhost".into()));
        let s = replay("[Client thread/INFO]: Connecting to localhost, 25565\n");
        assert_eq!(s, Session::Lan(None));
    }

    #[test]
    fn disconnect_returns_to_menu() {
        let log = "[Client thread/INFO]: Connecting to mc.hypixel.net, 25565\n\
                   [Client thread/INFO]: [CHAT] hi\n\
                   [Client thread/INFO]: Reached end of stream.\n";
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
    fn multiplayer_not_overwritten_by_world_save() {
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
