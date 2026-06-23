//! Buffer de consola del servidor + lectura de stdout y `logs/latest.log`.

use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::process::{ChildStdout, Stdio};
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

pub fn spawn_stdout_reader(id: String, stdout: ChildStdout) {
    spawn_stream_reader(id, stdout);
}

pub fn spawn_stderr_reader(id: String, stderr: impl Read + Send + 'static) {
    spawn_stream_reader(id, stderr);
}

fn spawn_stream_reader(id: String, stream: impl Read + Send + 'static) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines().map_while(Result::ok) {
            append(&id, &line);
        }
    });
}

/// Sigue `logs/latest.log` mientras el servidor corre (Paper escribe ahí).
pub fn spawn_log_tail(id: String, folder: std::path::PathBuf, running: impl Fn() -> bool + Send + 'static) {
    std::thread::spawn(move || {
        let log_path = folder.join("logs").join("latest.log");
        let mut last_pos = 0u64;
        while running() {
            std::thread::sleep(Duration::from_millis(800));
            let Ok(mut f) = std::fs::File::open(&log_path) else {
                continue;
            };
            if f.seek(SeekFrom::Start(last_pos)).is_err() {
                continue;
            }
            let mut buf = String::new();
            if f.read_to_string(&mut buf).is_err() {
                continue;
            }
            last_pos += buf.len() as u64;
            for line in buf.lines() {
                append(&id, line);
            }
        }
    });
}

pub fn pipe_stdio() -> (Stdio, Stdio, Stdio) {
    (Stdio::piped(), Stdio::piped(), Stdio::piped())
}

pub fn export_to_file(id: &str, folder: &std::path::Path) -> AppResult<String> {
    let lines = get_lines(id, MAX_LINES);
    let path = folder.join(format!(
        "launcher-console-{}.log",
        chrono_lite_timestamp()
    ));
    std::fs::write(&path, lines.join("\n"))?;
    Ok(path.to_string_lossy().to_string())
}

fn chrono_lite_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}
