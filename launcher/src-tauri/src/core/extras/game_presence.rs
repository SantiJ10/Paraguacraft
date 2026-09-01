//! Detecta mundo / servidor en juego (latest.log + saves/) y actualiza Discord RPC.

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

/// Hilo ligero: lee `latest.log` y refresca el RPC mientras el juego corre.
pub fn watch(ctx: PresenceCtx, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let mut last_mode = String::new();
        while !stop.load(Ordering::Relaxed) {
            let mode = detect_mode(&ctx.game_dir, ctx.launch_server.as_deref());
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
            std::thread::sleep(Duration::from_millis(900));
        }
    });
}

fn detect_mode(game_dir: &Path, launch_server: Option<&str>) -> String {
    if let Some(line) = parse_log_mode(game_dir, launch_server) {
        return line;
    }
    if crate::core::servers::any_playit_running() {
        return "Hosteando Servidor para amigos".into();
    }
    if let Some(addr) = launch_server.filter(|s| !s.trim().is_empty()) {
        let (host, _) = crate::core::favorites::parse_address(addr.trim());
            if !host.is_empty() && !is_local_host(&host) {
            return format!("Conectando a {host}…");
        }
    }
    "En el menú".into()
}

fn parse_log_mode(game_dir: &Path, launch_server: Option<&str>) -> Option<String> {
    let log = game_dir.join("logs").join("latest.log");
    let content = std::fs::read_to_string(&log).ok().unwrap_or_default();
    let tail: String = content
        .lines()
        .rev()
        .take(800)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    classify_session(&tail, game_dir, launch_server)
}

fn classify_session(tail: &str, game_dir: &Path, launch_server: Option<&str>) -> Option<String> {
    let low = tail.to_lowercase();
    let world = world_from_log(tail).or_else(|| active_world_from_saves(game_dir));
    let with_world = |prefix: &str| match world.as_deref() {
        Some(w) => format!("{prefix}: {w}"),
        None => prefix.to_string(),
    };

    if is_friend_host_log(&low) {
        return Some(with_world("Mundo abierto a amigos"));
    }
    if is_lan_host_log(&low) {
        return Some(with_world("Hosteando LAN"));
    }

    if let Some(host) = latest_remote_host(tail) {
        return Some(format!("Jugando en {host}"));
    }
    if let Some(server) = launch_server.filter(|s| !s.trim().is_empty()) {
        if tail.lines().rev().any(|l| {
            l.contains("Joined server") || l.contains("Logged in with entity id")
        }) {
            let (host, _) = crate::core::favorites::parse_address(server.trim());
            if !host.is_empty() && !is_local_host(&host) {
                return Some(format!("Jugando en {host}"));
            }
        }
    }

    if is_integrated_server(&low) || world.is_some() {
        return Some(with_world("Un jugador"));
    }

    None
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
        || (low.contains("singleplayer") && !low.contains("multiplayer"))
        || low.contains("preparing start region")
        || low.contains("saving chunks for level")
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

fn latest_remote_host(tail: &str) -> Option<String> {
    for line in tail.lines().rev() {
        let Some(host) = line
            .split("Connecting to ")
            .nth(1)
            .and_then(|rest| rest.split(',').next())
            .map(|s| s.trim().trim_end_matches('.').to_string())
            .filter(|h| !h.is_empty() && !h.contains(' '))
        else {
            continue;
        };
        if is_local_host(&host) {
            continue;
        }
        return Some(host);
    }
    None
}

fn world_from_log(tail: &str) -> Option<String> {
    for line in tail.lines().rev() {
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
        // 1.8.9: Saving chunks for level 'New World'/Overworld
        if let Some(idx) = line.find("Saving chunks for level '") {
            let rest = &line[idx + 25..];
            if let Some(end) = rest.find('\'') {
                let name = rest[..end].trim();
                if is_plausible_world(name) {
                    return Some(name.to_string());
                }
            }
        }
        if let Some(idx) = line.to_ascii_lowercase().find("loading world ") {
            let rest = line[idx + 14..].trim().trim_matches(['\'', '"', '.']);
            if is_plausible_world(rest) {
                return Some(rest.to_string());
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

/// Mundo actualmente abierto: `saves/<nombre>/session.lock`.
fn active_world_from_saves(game_dir: &Path) -> Option<String> {
    let saves = game_dir.join("saves");
    let rd = std::fs::read_dir(saves).ok()?;
    for e in rd.flatten() {
        let path = e.path();
        if !path.is_dir() {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.join("session.lock").is_file() {
            return Some(name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_hypixel() {
        let log = "[Render thread/INFO]: Connecting to mc.hypixel.net, 25565\n";
        assert_eq!(latest_remote_host(log).as_deref(), Some("mc.hypixel.net"));
    }

    #[test]
    fn ignores_localhost_lan() {
        let log = "[Render thread/INFO]: Connecting to localhost, 25565\n";
        assert_eq!(latest_remote_host(log), None);
    }

    #[test]
    fn lan_open() {
        let log = "[Server thread/INFO]: Started serving on 25565\n";
        let mode = classify_session(log, Path::new("."), None);
        assert_eq!(mode.as_deref(), Some("Hosteando LAN"));
    }

    #[test]
    fn essential_invite() {
        let log = "[Essential] Friends can now join your world\n";
        let mode = classify_session(log, Path::new("."), None);
        assert_eq!(mode.as_deref(), Some("Mundo abierto a amigos"));
    }

    #[test]
    fn e4mc_link() {
        let log = "[e4mc] Your server is now available at abc.e4mc.link\n";
        let mode = classify_session(log, Path::new("."), None);
        assert_eq!(mode.as_deref(), Some("Mundo abierto a amigos"));
    }

    #[test]
    fn singleplayer_world_name() {
        let log = "[Server thread/INFO]: Saving chunks for level 'Mi Mundo'/Overworld\nStarting integrated minecraft server\n";
        let mode = classify_session(log, Path::new("."), None);
        assert_eq!(mode.as_deref(), Some("Un jugador: Mi Mundo"));
    }
}
