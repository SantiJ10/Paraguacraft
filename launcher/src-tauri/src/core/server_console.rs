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
    drop(g);
    // Detectar claim / IP de tunnel del plugin playit-gg en la consola de Paper.
    crate::core::servers::on_mc_console_line(id, line);
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

/// Sigue `logs/latest.log` mientras el servidor corre (Paper escribe ahí).
///
/// Importante: tras el primer arranque Paper **deja de flushear stdout** al pipe
/// (no hay TTY) y solo escribe al archivo. Si el log se rota/trunca, hay que
/// resetear el offset; si no, la UI se queda congelada en las últimas líneas.
/// Leemos con buffer de línea incompleta para no perder trozos sin `\n`.
///
/// `start_pos`: offset inicial (normalmente el tamaño del log al iniciar el
/// proceso, para no re-emitir arranques anteriores).
pub fn spawn_log_tail(
    id: String,
    folder: std::path::PathBuf,
    start_pos: u64,
    running: impl Fn() -> bool + Send + 'static,
) {
    std::thread::spawn(move || {
        let log_path = folder.join("logs").join("latest.log");
        let mut last_pos = start_pos;
        let mut last_len = start_pos;
        let mut carry: Vec<u8> = Vec::new();
        while running() {
            std::thread::sleep(Duration::from_millis(400));
            let Ok(meta) = std::fs::metadata(&log_path) else {
                continue;
            };
            let len = meta.len();
            // Rotación / reescritura: el archivo se achicó o se reemplazó.
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
            // Emitir solo líneas completas (terminadas en \n).
            while let Some(nl) = carry.iter().position(|&b| b == b'\n') {
                let mut line_bytes: Vec<u8> = carry.drain(..=nl).collect();
                if line_bytes.last() == Some(&b'\n') {
                    line_bytes.pop();
                }
                if line_bytes.last() == Some(&b'\r') {
                    line_bytes.pop();
                }
                let line = String::from_utf8_lossy(&line_bytes);
                append_dedup(&id, &line);
            }
            // Evitar grow infinito si nunca llega un \n (basura binaria).
            if carry.len() > 64 * 1024 {
                let line = String::from_utf8_lossy(&carry);
                append_dedup(&id, &line);
                carry.clear();
            }
        }
    });
}

/// Igual que `append` pero ignora duplicados exactos recientes (stdout + latest.log).
pub fn append_dedup(id: &str, line: &str) {
    let line = line.trim_end();
    if line.is_empty() {
        return;
    }
    {
        let g = logs();
        if let Some(map) = g.as_ref() {
            if let Some(buf) = map.get(id) {
                for prev in buf.iter().rev().take(40) {
                    if prev == line {
                        return;
                    }
                }
            }
        }
    }
    append(id, line);
}

fn spawn_stream_reader(id: String, stream: impl Read + Send + 'static) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines().map_while(Result::ok) {
            append_dedup(&id, &line);
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
