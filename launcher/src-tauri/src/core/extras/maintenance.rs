//! Limpieza de logs/crash-reports y info de espacio.

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::core::instances;
use crate::core::paths;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupInfo {
    pub logs_mb: f64,
    pub crash_mb: f64,
    pub mc_ram_mb: u32,
}

fn dir_size_mb(path: &Path) -> f64 {
    let mut total = 0u64;
    if path.is_dir() {
        if let Ok(rd) = std::fs::read_dir(path) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_file() {
                    total += p.metadata().map(|m| m.len()).unwrap_or(0);
                } else if p.is_dir() {
                    total += (dir_size_mb(&p) * 1024.0 * 1024.0) as u64;
                }
            }
        }
    }
    (total as f64) / (1024.0 * 1024.0)
}

fn mc_java_ram_mb() -> u32 {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut mb = 0u64;
    for (_pid, proc_) in sys.processes() {
        let name = proc_.name().to_string_lossy().to_lowercase();
        if name == "javaw.exe" || name == "java.exe" {
            mb += proc_.memory() / 1024;
        }
    }
    mb as u32
}

pub fn info() -> CleanupInfo {
    static CACHE: Mutex<Option<(Instant, CleanupInfo)>> = Mutex::new(None);
    let mut guard = CACHE.lock().unwrap();
    if let Some((t, data)) = guard.as_ref() {
        if t.elapsed() < Duration::from_secs(120) {
            return data.clone();
        }
    }
    let data = info_uncached();
    *guard = Some((Instant::now(), data.clone()));
    data
}

fn info_uncached() -> CleanupInfo {
    let mc = paths::default_minecraft_dir();
    let mut logs = dir_size_mb(&mc.join("logs"));
    let mut crash = dir_size_mb(&mc.join("crash-reports"));
    for inst in instances::list_local() {
        let dir = instances::instance_dir(&inst.id);
        logs += dir_size_mb(&dir.join("logs"));
        crash += dir_size_mb(&dir.join("crash-reports"));
    }
    CleanupInfo {
        logs_mb: (logs * 10.0).round() / 10.0,
        crash_mb: (crash * 10.0).round() / 10.0,
        mc_ram_mb: mc_java_ram_mb(),
    }
}

pub fn run(kind: &str) -> crate::error::AppResult<u32> {
    let mut deleted = 0u32;
    let mc = paths::default_minecraft_dir();
    let mut log_dirs = vec![mc.join("logs")];
    let mut crash_dirs = vec![mc.join("crash-reports")];
    for inst in instances::list_local() {
        let dir = instances::instance_dir(&inst.id);
        log_dirs.push(dir.join("logs"));
        crash_dirs.push(dir.join("crash-reports"));
    }

    if kind == "logs" || kind == "both" {
        for dir in log_dirs {
            deleted += clean_logs_dir(&dir);
        }
    }
    if kind == "crash" || kind == "both" {
        for dir in crash_dirs {
            deleted += clean_all_files(&dir);
        }
    }
    Ok(deleted)
}

fn clean_logs_dir(dir: &Path) -> u32 {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut n = 0;
    for e in rd.flatten() {
        let p = e.path();
        if !p.is_file() {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if name == "latest.log" {
            continue;
        }
        if std::fs::remove_file(&p).is_ok() {
            n += 1;
        }
    }
    n
}

fn clean_all_files(dir: &Path) -> u32 {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut n = 0;
    for e in rd.flatten() {
        if e.path().is_file() && std::fs::remove_file(e.path()).is_ok() {
            n += 1;
        }
    }
    n
}

/// Backup rápido de mundos de una instancia (zip en backups/).
pub fn auto_backup_worlds(instance_id: &str) -> crate::error::AppResult<()> {
    let _ = instances::backups::create(instance_id)?;
    Ok(())
}
