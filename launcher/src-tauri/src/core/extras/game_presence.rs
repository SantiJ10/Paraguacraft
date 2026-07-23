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
            std::thread::sleep(Duration::from_millis(900));
        }
    });
}

fn detect_mode(game_dir: &Path, launch_server: Option<&str>) -> String {
    if let Some(line) = parse_log_mode(game_dir, launch_server) {
        return line;
    }
    if let Some(addr) = launch_server.filter(|s| !s.trim().is_empty()) {
        let (host, _) = crate::core::favorites::parse_address(addr.trim());
        if !host.is_empty() {
            return format!("Conectando a {host}…");
        }
    }
    "En el menú".into()
}

fn parse_log_mode(game_dir: &Path, launch_server: Option<&str>) -> Option<String> {
    let log = game_dir.join("logs").join("latest.log");
    let content = std::fs::read_to_string(&log).ok()?;
    let tail: String = content
        .lines()
        .rev()
        .take(500)
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
                return Some(host.to_string());
            }
        }
        if let Some(host) = line.split("Connecting to ").nth(1) {
            let host = host.trim().trim_end_matches('.');
            if !host.is_empty() && !host.contains(' ') {
                return Some(host.to_string());
            }
        }
        if line.contains("Joined server") || line.contains("Logged in with entity id") {
            if let Some(server) = launch_server.filter(|s| !s.trim().is_empty()) {
                let (host, _) = crate::core::favorites::parse_address(server.trim());
                if !host.is_empty() {
                    return Some(host.to_string());
                }
            }
        }
    }

    if low.contains("starting integrated minecraft server")
        || low.contains("singleplayer")
        || low.contains("local game hosted on")
    {
        if let Some(world) = world_from_log(&tail) {
            return Some(format!("Un jugador: {world}"));
        }
        return Some("Un jugador".into());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiplayer_connect() {
        let log = "[Render thread/INFO]: Connecting to mc.hypixel.net, 25565\n";
        let mode = parse_log_mode_from_tail(log);
        assert_eq!(mode.as_deref(), Some("mc.hypixel.net"));
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
                    return Some(host.to_string());
                }
            }
        }
        None
    }
}
