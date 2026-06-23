//! Prioridad del proceso javaw.exe / java.exe.

use std::process::Command;
use std::sync::Mutex;

static LEVEL: Mutex<String> = Mutex::new(String::new());

pub fn current_level() -> String {
    let level = LEVEL.lock().unwrap().clone();
    if level.is_empty() {
        "normal".into()
    } else {
        level
    }
}

#[cfg(windows)]
pub fn set_level(level: &str) -> crate::error::AppResult<u32> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let (ps_prio, label) = match level {
        "realtime" => ("RealTime", "realtime"),
        "high" | "alta" => ("High", "high"),
        "low" | "baja" => ("Idle", "low"),
        _ => ("Normal", "normal"),
    };

    let mut count = 0u32;
    for name in ["javaw", "java"] {
        let script = format!(
            "(Get-Process -Name '{name}' -ErrorAction SilentlyContinue | ForEach-Object {{ $_.PriorityClass = '{ps_prio}' }} | Measure-Object).Count"
        );
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", &script]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        if let Ok(out) = cmd.output() {
            if let Ok(n) = String::from_utf8_lossy(&out.stdout).trim().parse::<u32>() {
                count += n;
            }
        }
    }
    *LEVEL.lock().unwrap() = label.to_string();
    Ok(count)
}

#[cfg(not(windows))]
pub fn set_level(level: &str) -> crate::error::AppResult<u32> {
    *LEVEL.lock().unwrap() = level.to_string();
    Ok(0)
}
