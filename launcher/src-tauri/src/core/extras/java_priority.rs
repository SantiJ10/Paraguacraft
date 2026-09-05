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

fn normalize_level(level: &str) -> &'static str {
    match level {
        "realtime" => "realtime",
        "high" | "alta" => "high",
        "low" | "baja" => "low",
        _ => "normal",
    }
}

/// Prioridad nativa del PID de `javaw` recién lanzado (`SetPriorityClass`).
/// El fallback PowerShell cubre procesos java hijos que el juego pueda spawnar.
pub fn set_for_pid(pid: u32, level: &str) -> crate::error::AppResult<()> {
    let label = normalize_level(level);
    #[cfg(windows)]
    {
        if set_priority_class_native(pid, label) {
            *LEVEL.lock().unwrap() = label.to_string();
            return Ok(());
        }
    }
    let _ = set_level(label);
    Ok(())
}

#[cfg(windows)]
fn set_priority_class_native(pid: u32, level: &str) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, SetPriorityClass, HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS,
        NORMAL_PRIORITY_CLASS, PROCESS_SET_INFORMATION, REALTIME_PRIORITY_CLASS,
    };

    let class = match level {
        "realtime" => REALTIME_PRIORITY_CLASS,
        "high" => HIGH_PRIORITY_CLASS,
        "low" => IDLE_PRIORITY_CLASS,
        _ => NORMAL_PRIORITY_CLASS,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let ok = SetPriorityClass(handle, class) != 0;
        let _ = CloseHandle(handle);
        ok
    }
}

#[cfg(windows)]
pub fn set_level(level: &str) -> crate::error::AppResult<u32> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let label = normalize_level(level);
    let ps_prio = match label {
        "realtime" => "RealTime",
        "high" => "High",
        "low" => "Idle",
        _ => "Normal",
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
    *LEVEL.lock().unwrap() = normalize_level(level).to_string();
    Ok(0)
}
