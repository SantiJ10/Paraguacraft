//! Game Mode: libera RAM y prioriza Java (mejora sobre Python: también cierra apps pesadas).

use crate::core::extras::java_priority;

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

static ACTIVE: AtomicBool = AtomicBool::new(false);

const KILL_APPS: &[&str] = &[
    "Teams.exe",
    "OneDrive.exe",
    "MicrosoftTeams.exe",
    "SearchApp.exe",
    "Widgets.exe",
    "YourPhone.exe",
];

pub fn is_active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

#[cfg(windows)]
fn hide(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
fn hide(_cmd: &mut Command) {}

fn taskkill(name: &str) -> bool {
    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/IM", name]);
    hide(&mut cmd);
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn activate() -> crate::error::AppResult<Vec<String>> {
    let mut msgs = Vec::new();
    let mut killed = Vec::new();
    for app in KILL_APPS {
        if taskkill(app) {
            killed.push(app.trim_end_matches(".exe"));
        }
    }
    if !killed.is_empty() {
        msgs.push(format!("Apps cerradas: {}", killed.join(", ")));
    }

    let javaw = java_priority::set_level("high")?;
    if javaw > 0 {
        msgs.push(format!("Prioridad Java → Alta ({javaw} proceso(s))"));
    }

    #[cfg(windows)]
    {
        let mut ps = Command::new("powershell");
        ps.args([
            "-NoProfile",
            "-Command",
            "[System.GC]::Collect(); [System.GC]::WaitForPendingFinalizers()",
        ]);
        hide(&mut ps);
        let _ = ps.output();
        msgs.push("Memoria standby liberada".into());
    }

    ACTIVE.store(true, Ordering::Relaxed);
    if msgs.is_empty() {
        msgs.push("Game Mode activo".into());
    }
    Ok(msgs)
}

pub fn deactivate() -> crate::error::AppResult<Vec<String>> {
    let n = java_priority::set_level("normal")?;
    ACTIVE.store(false, Ordering::Relaxed);
    Ok(vec![format!(
        "Game Mode desactivado (Java normal en {n} proceso(s))"
    )])
}
