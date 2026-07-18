//! Deteccion de hardware y autoconfig de JVM/GC/RAM.

use crate::models::HardwareInfo;
use sysinfo::System;

pub fn detect() -> HardwareInfo {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    let ram_gb = (sys.total_memory() as f64) / (1024.0 * 1024.0 * 1024.0);
    let ram_gb = (ram_gb * 10.0).round() / 10.0;

    let cpu_threads = sys.cpus().len() as u32;
    let cpu_cores = sys.physical_core_count().unwrap_or(cpu_threads as usize) as u32;
    let cpu_name = detect_cpu_name().unwrap_or_else(|| {
        sys.cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "CPU desconocida".to_string())
    });

    let gpu_name = detect_gpu_name().unwrap_or_else(|| "GPU del sistema".to_string());

    let perfil = if ram_gb <= 8.0 || cpu_cores <= 4 {
        "baja"
    } else if ram_gb >= 24.0 || (ram_gb >= 16.0 && cpu_cores >= 6) {
        "alta"
    } else if ram_gb <= 12.0 && cpu_cores <= 6 {
        "baja"
    } else {
        "media"
    };

    let (recommended_ram_mb, recommended_gc) = recommend(ram_gb);

    HardwareInfo {
        ram_gb,
        cpu_cores,
        cpu_threads,
        cpu_name,
        gpu_name,
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        perfil_sugerido: perfil.to_string(),
        recommended_ram_mb,
        recommended_gc: recommended_gc.to_string(),
    }
}

#[cfg(windows)]
fn run_hidden_ps(script: &str) -> Option<String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(not(windows))]
fn run_hidden_ps(_script: &str) -> Option<String> {
    None
}

fn detect_cpu_name() -> Option<String> {
    #[cfg(windows)]
    {
        if let Some(json) = run_hidden_ps(
            "(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name)",
        ) {
            let name = json.lines().next()?.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        if let Ok(out) = Command::new("wmic")
            .args(["cpu", "get", "name"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines().skip(1) {
                let name = line.trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let out = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()?;
        let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("model name") {
                    if let Some(name) = rest.strip_prefix(':').map(str::trim).filter(|s| !s.is_empty()) {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }
    None
}

struct GpuEntry {
    name: String,
    vram_bytes: u64,
}

fn is_virtual_gpu(name: &str) -> bool {
    let l = name.to_lowercase();
    [
        "virtual",
        "parsec",
        "microsoft basic",
        "remote desktop",
        "spacedesk",
        "meta virtual",
        "vmware",
        "virtualbox",
        "indirect display",
        "mirror",
    ]
    .iter()
    .any(|k| l.contains(k))
}

fn gpu_score(name: &str, vram_bytes: u64) -> i32 {
    if is_virtual_gpu(name) {
        return -10_000;
    }
    let mut score = 0i32;
    let vram_gb = (vram_bytes / (1024 * 1024 * 1024)) as i32;
    score += vram_gb * 8;
    if vram_bytes >= 2 * 1024 * 1024 * 1024 {
        score += 20;
    }

    let l = name.to_lowercase();
    if l.contains("rx ")
        || l.contains("rtx ")
        || l.contains("gtx ")
        || l.contains("arc a")
        || l.contains("radeon pro")
    {
        score += 60;
    }
    if l.contains("radeon") && l.contains("graphics") && !l.contains("rx") {
        score -= 40;
    }
    if l.contains("intel") && (l.contains("uhd") || l.contains("iris")) {
        score -= 30;
    }
    score
}

fn pick_best_gpu(entries: &[GpuEntry]) -> Option<String> {
    entries
        .iter()
        .max_by_key(|g| gpu_score(&g.name, g.vram_bytes))
        .filter(|g| gpu_score(&g.name, g.vram_bytes) > 0)
        .map(|g| g.name.clone())
}

#[cfg(windows)]
fn list_gpus_windows() -> Vec<GpuEntry> {
    let Some(json) = run_hidden_ps(
        "Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM | ConvertTo-Json -Compress",
    ) else {
        return Vec::new();
    };

    let v: serde_json::Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let items: Vec<serde_json::Value> = if v.is_array() {
        v.as_array().cloned().unwrap_or_default()
    } else {
        vec![v]
    };

    items
        .into_iter()
        .filter_map(|item| {
            let name = item["Name"].as_str()?.trim().to_string();
            if name.is_empty() {
                return None;
            }
            let vram = item["AdapterRAM"]
                .as_u64()
                .or_else(|| item["AdapterRAM"].as_i64().map(|n| n.max(0) as u64))
                .unwrap_or(0);
            Some(GpuEntry {
                name,
                vram_bytes: vram,
            })
        })
        .collect()
}

fn detect_gpu_name() -> Option<String> {
    #[cfg(windows)]
    {
        let entries = list_gpus_windows();
        if let Some(name) = pick_best_gpu(&entries) {
            return Some(name);
        }
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        if let Ok(out) = Command::new("wmic")
            .args(["path", "win32_VideoController", "get", "name,AdapterRAM"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut entries = Vec::new();
            let mut pending_name: Option<String> = None;
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.eq_ignore_ascii_case("name") {
                    continue;
                }
                if pending_name.is_none() && !line.chars().all(|c| c.is_ascii_digit()) {
                    pending_name = Some(line.to_string());
                } else if let Some(name) = pending_name.take() {
                    let vram = line.parse::<u64>().unwrap_or(0);
                    entries.push(GpuEntry {
                        name,
                        vram_bytes: vram,
                    });
                }
            }
            if let Some(name) = pick_best_gpu(&entries) {
                return Some(name);
            }
        }
    }
    None
}

/// Lectura de uso/temperatura de GPU para el HUD in-game (Windows).
pub struct GpuSnapshot {
    pub usage_pct: f32,
    pub temp_c: f32,
}

pub fn read_gpu_snapshot() -> GpuSnapshot {
  GpuSnapshot {
    usage_pct: read_gpu_usage_pct().unwrap_or(-1.0),
    temp_c: read_gpu_temp_c().unwrap_or(-1.0),
  }
}

fn read_gpu_usage_pct() -> Option<f32> {
    #[cfg(windows)]
    {
        if let Some(text) = run_hidden_ps(
            "(Get-Counter '\\GPU Engine(*engtype_3D*)\\Utilization Percentage' -ErrorAction SilentlyContinue).CounterSamples | Measure-Object -Property CookedValue -Maximum | Select-Object -ExpandProperty Maximum",
        ) {
            let normalized = text.trim().replace(',', ".");
            if let Ok(v) = normalized.parse::<f32>() {
                if v >= 0.0 && v <= 100.0 {
                    return Some(v);
                }
            }
        }
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        if let Ok(out) = Command::new("nvidia-smi")
            .args([
                "--query-gpu=utilization.gpu",
                "--format=csv,noheader,nounits",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Ok(v) = text.replace('%', "").trim().parse::<f32>() {
                if v >= 0.0 && v <= 100.0 {
                    return Some(v);
                }
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = ();
    }
    None
}

fn read_gpu_temp_c() -> Option<f32> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        if let Ok(out) = Command::new("nvidia-smi")
            .args([
                "--query-gpu=temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Ok(v) = text.parse::<f32>() {
                if v > 0.0 && v < 130.0 {
                    return Some(v);
                }
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = ();
    }
    None
}

/// Devuelve (RAM recomendada en MB, GC recomendado) segun la RAM total.
pub fn recommend(ram_gb: f64) -> (u32, &'static str) {
    let ram_mb = if ram_gb <= 8.0 {
        ((ram_gb * 1024.0 * 0.45) as u32).min(4096)
    } else if ram_gb <= 16.0 {
        6144
    } else if ram_gb <= 32.0 {
        12288
    } else {
        16384
    };
    let ram_mb = ((ram_mb + 256) / 512 * 512).max(2048);
    let gc = if ram_gb >= 16.0 { "ZGC" } else { "G1GC" };
    (ram_mb, gc)
}

/// RAM sugerida al crear instancia 1.8.9 PvP (tope 4 GB).
pub fn recommend_pvp_189_ram_mb(ram_gb: f64) -> u32 {
    crate::core::launch::pvp_jvm::resolve_ram_mb(ram_gb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_discrete_over_virtual() {
        let entries = vec![
            GpuEntry {
                name: "Parsec Virtual Display Adapter".into(),
                vram_bytes: 0,
            },
            GpuEntry {
                name: "AMD Radeon RX 6650 XT".into(),
                vram_bytes: 8 * 1024 * 1024 * 1024,
            },
        ];
        assert_eq!(
            pick_best_gpu(&entries).as_deref(),
            Some("AMD Radeon RX 6650 XT")
        );
    }
}
