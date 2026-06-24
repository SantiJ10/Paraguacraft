//! Detecta mundo / servidor en juego (latest.log) y actualiza Discord RPC.

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
                        Some(&mode),
                        ctx.settings.discord_rpc_version,
                        ctx.settings.discord_rpc_time,
                    );
                }
            }
            std::thread::sleep(Duration::from_secs(3));
        }
    });
}

fn detect_mode(game_dir: &Path, launch_server: Option<&str>) -> String {
    if let Some(line) = parse_log_mode(game_dir) {
        return line;
    }
    if let Some(addr) = launch_server.filter(|s| !s.trim().is_empty()) {
        let (host, _) = crate::core::favorites::parse_address(addr.trim());
        return format!("Multijugador: {host}");
    }
    if let Some(world) = newest_save_name(game_dir) {
        return format!("Un Jugador: {world}");
    }
    "En el menú".into()
}

fn parse_log_mode(game_dir: &Path) -> Option<String> {
    let log = game_dir.join("logs").join("latest.log");
    let content = std::fs::read_to_string(&log).ok()?;
    let tail: String = content
        .lines()
        .rev()
        .take(400)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    let low = tail.to_lowercase();

    for line in tail.lines().rev() {
        if let Some(host) = line
            .split("Connecting to ")
            .nth(1)
            .and_then(|rest| rest.split(',').next())
        {
            let host = host.trim();
            if !host.is_empty() {
                return Some(format!("Multijugador: {host}"));
            }
        }
    }

    if low.contains("starting integrated minecraft server")
        || low.contains("singleplayer")
        || low.contains("local game hosted on")
    {
        if let Some(world) = world_from_log(&tail).or_else(|| newest_save_name(game_dir)) {
            return Some(format!("Un Jugador: {world}"));
        }
        return Some("Un Jugador".into());
    }

    None
}

fn world_from_log(tail: &str) -> Option<String> {
    for line in tail.lines().rev() {
        if let Some(idx) = line.find("ServerLevel[") {
            let rest = &line[idx + 12..];
            if let Some(end) = rest.find(']') {
                let name = rest[..end].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        if let Some(idx) = line.find("Joined world '") {
            let rest = &line[idx + 14..];
            if let Some(end) = rest.find('\'') {
                let name = rest[..end].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

fn newest_save_name(game_dir: &Path) -> Option<String> {
    let saves = game_dir.join("saves");
    let mut best: Option<(std::time::SystemTime, String)> = None;
    for entry in std::fs::read_dir(&saves).ok()?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.eq_ignore_ascii_case("readme.txt") {
            continue;
        }
        if !path.join("level.dat").is_file() && !path.join("level.dat_old").is_file() {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        if best.as_ref().map(|(t, _)| modified > *t).unwrap_or(true) {
            best = Some((modified, name));
        }
    }
    best.map(|(_, n)| n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiplayer_connect() {
        let log = "[Render thread/INFO]: Connecting to tax-estimated.gl.joinmc.link, 25565\n";
        assert_eq!(
            world_from_log(log),
            None
        );
        let mode = parse_log_mode_from_tail(log);
        assert_eq!(mode.as_deref(), Some("Multijugador: tax-estimated.gl.joinmc.link"));
    }

    fn parse_log_mode_from_tail(tail: &str) -> Option<String> {
        for line in tail.lines().rev() {
            if let Some(host) = line
                .split("Connecting to ")
                .nth(1)
                .and_then(|rest| rest.split(',').next())
            {
                let host = host.trim();
                if !host.is_empty() {
                    return Some(format!("Multijugador: {host}"));
                }
            }
        }
        None
    }
}
