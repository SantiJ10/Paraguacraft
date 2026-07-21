//! IPC mmap launcher → juego (CPU/RAM/música para HUD in-game).

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use sysinfo::System;

use crate::core::hardware;
use crate::core::music_art_cache;
use crate::core::paths;

const MAGIC: &[u8; 4] = b"PGIP";
const SIZE: usize = 512;

static MUSIC: Mutex<(bool, String, String, String)> = Mutex::new((false, String::new(), String::new(), String::new()));
static ART_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn art_client() -> reqwest::Client {
    ART_CLIENT.get_or_init(reqwest::Client::new).clone()
}

/// Si `image_url` es http(s), intenta resolverlo a un archivo local ya
/// cacheado por `music_art_cache` (más rápido y sin depender de que el juego
/// mismo pueda descargarlo). Si aún no está cacheada, dispara la descarga en
/// background y devuelve la URL original para que el cliente la use mientras.
fn path_to_file_url(path: &std::path::Path) -> String {
    let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut s = p.to_string_lossy().replace('\\', "/");
    if s.starts_with("//") {
        s = s.trim_start_matches('/').to_string();
    }
    if s.len() >= 2 && s.as_bytes()[1] == b':' {
        format!("file:///{}", s)
    } else {
        format!("file://{}", s)
    }
}

fn resolve_image_field(image_url: &str) -> String {
    let trimmed = image_url.trim();
    if trimmed.is_empty() || !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return trimmed.to_string();
    }
    match music_art_cache::ensure_cached(art_client(), trimmed) {
        Some(path) => path_to_file_url(&path),
        None => trimmed.to_string(),
    }
}

pub fn ipc_path() -> PathBuf {
    paths::data_dir().join("game-overlay.dat")
}

pub fn set_music(playing: bool, title: &str, artist: &str, image_url: &str) {
    if let Ok(mut g) = MUSIC.lock() {
        *g = (
            playing,
            title.to_string(),
            artist.to_string(),
            image_url.to_string(),
        );
    }
    flush_music_fields();
}

fn flush_music_fields() {
    let path = ipc_path();
    let mut buf = match std::fs::read(&path) {
        Ok(data) if data.len() >= SIZE => data,
        Ok(mut data) => {
            data.resize(SIZE, 0);
            data
        }
        _ => vec![0u8; SIZE],
    };
    let (playing, title, artist, image) = MUSIC
        .lock()
        .map(|g| (g.0, g.1.clone(), g.2.clone(), g.3.clone()))
        .unwrap_or((false, String::new(), String::new(), String::new()));
    let image_field = resolve_image_field(&image);
    buf[0..4].copy_from_slice(MAGIC);
    if buf.len() >= 8 {
        buf[4..8].copy_from_slice(&1i32.to_le_bytes());
    }
    buf[24] = if playing { 1 } else { 0 };
    write_fixed_str(&mut buf, 25, 128, &title);
    write_fixed_str(&mut buf, 153, 64, &artist);
    write_fixed_str(&mut buf, 217, 256, &image_field);
    let _ = std::fs::write(&path, &buf[..SIZE]);
}

fn write_fixed_str(buf: &mut [u8], off: usize, max: usize, text: &str) {
    let end = off.saturating_add(max).min(buf.len());
    if off >= end {
        return;
    }
    buf[off..end].fill(0);
    let bytes = text.as_bytes();
    let len = bytes.len().min(max.saturating_sub(1));
    buf[off..off + len].copy_from_slice(&bytes[..len]);
}

pub fn write_snapshot(sys: &mut System) {
    // sysinfo calcula el % de CPU por DELTA entre dos refrescos: por eso el
    // System se reutiliza entre llamadas (antes se creaba nuevo cada vez y daba
    // siempre 0/100 o valores basura, "demasiada carga").
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu = sys.global_cpu_usage();
    let ram_pct = if sys.total_memory() > 0 {
        (sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0) as f32
    } else {
        -1.0
    };

    let gpu = hardware::read_gpu_snapshot();

    let (music_playing, music_title, music_artist, music_image) = MUSIC
        .lock()
        .map(|g| (g.0, g.1.clone(), g.2.clone(), g.3.clone()))
        .unwrap_or((false, String::new(), String::new(), String::new()));

    let mut buf = [0u8; SIZE];
    buf[0..4].copy_from_slice(MAGIC);
    buf[4..8].copy_from_slice(&1i32.to_le_bytes());
    buf[8..12].copy_from_slice(&cpu.to_le_bytes());
    buf[12..16].copy_from_slice(&ram_pct.to_le_bytes());
    buf[16..20].copy_from_slice(&gpu.usage_pct.to_le_bytes());
    buf[20..24].copy_from_slice(&gpu.temp_c.to_le_bytes());
    buf[24] = if music_playing { 1 } else { 0 };
    write_fixed_str(&mut buf, 25, 128, &music_title);
    write_fixed_str(&mut buf, 153, 64, &music_artist);
    write_fixed_str(&mut buf, 217, 256, &resolve_image_field(&music_image));

    let path = ipc_path();
    if let Ok(mut f) = std::fs::File::create(&path) {
        let _ = f.write_all(&buf);
    }
}

pub fn watch(stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        // System persistente: necesario para que el % de CPU se calcule por delta.
        let mut sys = System::new();
        // Primer refresco "de cebado": la primera medición de CPU siempre es 0.
        sys.refresh_cpu_all();
        std::thread::sleep(Duration::from_millis(250));
        while !stop.load(Ordering::Relaxed) {
            write_snapshot(&mut sys);
            std::thread::sleep(Duration::from_millis(500));
        }
        let _ = std::fs::remove_file(ipc_path());
    });
}
