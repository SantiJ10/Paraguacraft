//! IPC mmap launcher → juego (CPU/RAM/música para HUD in-game).

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sysinfo::System;

use crate::core::hardware;
use crate::core::paths;

const MAGIC: &[u8; 4] = b"PGIP";
const SIZE: usize = 512;

static MUSIC: Mutex<(bool, String, String, String)> = Mutex::new((false, String::new(), String::new(), String::new()));

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
}

fn write_fixed_str(buf: &mut [u8], off: usize, max: usize, text: &str) {
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
    write_fixed_str(&mut buf, 217, 256, &music_image);

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
