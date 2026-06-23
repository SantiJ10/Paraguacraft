//! Modo Turbo: DNS gaming + cierre de apps que consumen ancho de banda.

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

static ACTIVE: AtomicBool = AtomicBool::new(false);

const KILL_APPS: &[&str] = &["Teams.exe", "OneDrive.exe", "Spotify.exe", "Discord.exe"];

#[cfg(windows)]
fn hide(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
fn hide(_cmd: &mut Command) {}

pub fn is_active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

pub fn activate() -> crate::error::AppResult<Vec<String>> {
    let mut msgs = Vec::new();

    #[cfg(windows)]
    {
        for iface in ["Wi-Fi", "Ethernet"] {
            let mut cmd = Command::new("netsh");
            cmd.args([
                "interface",
                "ip",
                "set",
                "dns",
                &format!("name={iface}"),
                "static",
                "1.1.1.1",
            ]);
            hide(&mut cmd);
            if cmd.output().map(|o| o.status.success()).unwrap_or(false) {
                msgs.push(format!("DNS {iface} → Cloudflare 1.1.1.1"));
                break;
            }
        }
        if msgs.is_empty() {
            msgs.push("DNS: requiere ejecutar el launcher como administrador".into());
        }
    }

    let mut killed = Vec::new();
    for app in KILL_APPS {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/F", "/IM", app]);
        hide(&mut cmd);
        if cmd.output().map(|o| o.status.success()).unwrap_or(false) {
            killed.push(app.trim_end_matches(".exe"));
        }
    }
    if !killed.is_empty() {
        msgs.push(format!("Cerrados: {}", killed.join(", ")));
    }

    ACTIVE.store(true, Ordering::Relaxed);
    msgs.push("Modo Turbo activo".into());
    Ok(msgs)
}

pub fn deactivate() -> crate::error::AppResult<Vec<String>> {
    let mut msgs = vec!["Modo Turbo desactivado".into()];

    #[cfg(windows)]
    {
        for iface in ["Wi-Fi", "Ethernet"] {
            let mut cmd = Command::new("netsh");
            cmd.args(["interface", "ip", "set", "dns", &format!("name={iface}"), "dhcp"]);
            hide(&mut cmd);
            if cmd.output().map(|o| o.status.success()).unwrap_or(false) {
                msgs.push(format!("DNS {iface} → automático (DHCP)"));
                break;
            }
        }
    }

    ACTIVE.store(false, Ordering::Relaxed);
    Ok(msgs)
}
