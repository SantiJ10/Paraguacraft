//! Buffer de consola del cliente (Minecraft) + tail de `logs/latest.log`.
//!
//! No capturamos stdout de `javaw` (Discord overlay / detección de juego);
//! el log real del cliente vive en archivo, igual que en la consola de servers.

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use crate::error::AppResult;

const MAX_LINES: usize = 1000;

static LOGS: Mutex<Option<HashMap<String, VecDeque<String>>>> = Mutex::new(None);

fn logs() -> std::sync::MutexGuard<'static, Option<HashMap<String, VecDeque<String>>>> {
    let mut g = LOGS.lock().unwrap();
    if g.is_none() {
        *g = Some(HashMap::new());
    }
    g
}

pub fn clear(id: &str) {
    if let Some(map) = logs().as_mut() {
        map.remove(id);
    }
}

pub fn append(id: &str, line: &str) {
    let line = line.trim_end();
    if line.is_empty() {
        return;
    }
    let mut g = logs();
    let map = g.as_mut().unwrap();
    let buf = map.entry(id.to_string()).or_default();
    buf.push_back(line.to_string());
    while buf.len() > MAX_LINES {
        buf.pop_front();
    }
}

pub fn get_lines(id: &str, max: usize) -> Vec<String> {
    let g = logs();
    let Some(map) = g.as_ref() else {
        return Vec::new();
    };
    let Some(buf) = map.get(id) else {
        return Vec::new();
    };
    let take = max.min(buf.len());
    buf.iter()
        .skip(buf.len().saturating_sub(take))
        .cloned()
        .collect()
}

pub fn seed_from_file(id: &str, game_dir: &Path, max_lines: usize) {
    let path = game_dir.join("logs").join("latest.log");
    if !path.is_file() {
        append(id, "[launcher] Consola del cliente — esperando logs/latest.log…");
        return;
    }
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    for line in &lines[start..] {
        append(id, line);
    }
}

/// Sigue `logs/latest.log` mientras la partida corre.
///
/// `start_pos`: normalmente el tamaño del archivo al lanzar (solo líneas nuevas).
pub fn spawn_log_tail(
    id: String,
    game_dir: std::path::PathBuf,
    start_pos: u64,
    running: impl Fn() -> bool + Send + 'static,
) {
    std::thread::spawn(move || {
        let log_path = game_dir.join("logs").join("latest.log");
        let mut last_pos = start_pos;
        let mut last_len = start_pos;
        let mut carry: Vec<u8> = Vec::new();
        while running() {
            std::thread::sleep(Duration::from_millis(400));
            let Ok(meta) = std::fs::metadata(&log_path) else {
                continue;
            };
            let len = meta.len();
            if len < last_len || (last_pos > 0 && last_pos > len) {
                last_pos = 0;
                carry.clear();
            }
            last_len = len;

            let Ok(mut f) = std::fs::File::open(&log_path) else {
                continue;
            };
            if f.seek(SeekFrom::Start(last_pos)).is_err() {
                last_pos = 0;
                carry.clear();
                continue;
            }
            let mut bytes = Vec::new();
            if f.read_to_end(&mut bytes).is_err() {
                continue;
            }
            if bytes.is_empty() {
                continue;
            }
            last_pos += bytes.len() as u64;
            carry.extend_from_slice(&bytes);
            while let Some(nl) = carry.iter().position(|&b| b == b'\n') {
                let mut line_bytes: Vec<u8> = carry.drain(..=nl).collect();
                if line_bytes.last() == Some(&b'\n') {
                    line_bytes.pop();
                }
                if line_bytes.last() == Some(&b'\r') {
                    line_bytes.pop();
                }
                let line = String::from_utf8_lossy(&line_bytes);
                append(&id, &line);
            }
            if carry.len() > 64 * 1024 {
                let line = String::from_utf8_lossy(&carry);
                append(&id, &line);
                carry.clear();
            }
        }
    });
}

/// Inicia sesión de consola para un launch (seed + tail).
pub fn begin_session(instance_id: &str, game_dir: &Path) {
    clear(instance_id);
    seed_from_file(instance_id, game_dir, 120);
    let log_start = std::fs::metadata(game_dir.join("logs").join("latest.log"))
        .map(|m| m.len())
        .unwrap_or(0);
    let id = instance_id.to_string();
    let id_check = id.clone();
    spawn_log_tail(id, game_dir.to_path_buf(), log_start, move || {
        crate::core::game_session::is_running()
            && crate::core::game_session::last_launch_instance().as_deref() == Some(id_check.as_str())
    });
}

pub fn export_to_file(id: &str, game_dir: &Path) -> AppResult<String> {
    let lines = {
        let live = get_lines(id, MAX_LINES);
        if !live.is_empty() {
            live
        } else {
            crate::core::instance_repair::read_log_lines(id, MAX_LINES).unwrap_or_default()
        }
    };
    let path = game_dir.join(format!(
        "launcher-client-console-{}.log",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".into())
    ));
    std::fs::write(&path, lines.join("\n"))?;
    Ok(path.to_string_lossy().to_string())
}
