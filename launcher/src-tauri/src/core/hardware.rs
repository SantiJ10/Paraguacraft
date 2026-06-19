//! Deteccion de hardware y autoconfig de JVM/GC/RAM.
//!
//! Lee el sistema con `sysinfo` y deriva un perfil (baja/media/alta) y una
//! recomendacion de RAM + Garbage Collector. La GPU se detecta de forma mas
//! completa en fases posteriores; aqui dejamos un valor base.

use crate::models::HardwareInfo;
use sysinfo::System;

pub fn detect() -> HardwareInfo {
    // `new_all` garantiza que las listas de CPU y memoria esten pobladas
    // (con `new()` + `refresh_cpu_all` la lista de CPUs puede quedar vacia).
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    let ram_gb = (sys.total_memory() as f64) / (1024.0 * 1024.0 * 1024.0);
    let ram_gb = (ram_gb * 10.0).round() / 10.0;

    let cpu_threads = sys.cpus().len() as u32;
    let cpu_cores = System::physical_core_count().unwrap_or(cpu_threads as usize) as u32;
    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "CPU desconocida".to_string());

    let perfil = if ram_gb <= 8.0 || cpu_cores <= 4 {
        "baja"
    } else if ram_gb <= 16.0 || cpu_cores <= 8 {
        "media"
    } else {
        "alta"
    };

    let (recommended_ram_mb, recommended_gc) = recommend(ram_gb);

    HardwareInfo {
        ram_gb,
        cpu_cores,
        cpu_threads,
        cpu_name,
        gpu_name: "GPU del sistema".to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        perfil_sugerido: perfil.to_string(),
        recommended_ram_mb,
        recommended_gc: recommended_gc.to_string(),
    }
}

/// Devuelve (RAM recomendada en MB, GC recomendado) segun la RAM total.
/// Conservador en gama baja: nunca mas del ~45% para dejar respirar al SO.
fn recommend(ram_gb: f64) -> (u32, &'static str) {
    let ram_mb = if ram_gb <= 8.0 {
        ((ram_gb * 1024.0 * 0.45) as u32).min(4096)
    } else if ram_gb <= 16.0 {
        6144
    } else {
        8192
    };
    // Redondeo a multiplos de 512 MB, minimo 2 GB.
    let ram_mb = ((ram_mb + 256) / 512 * 512).max(2048);
    let gc = if ram_gb >= 8.0 { "ZGC" } else { "G1GC" };
    (ram_mb, gc)
}
