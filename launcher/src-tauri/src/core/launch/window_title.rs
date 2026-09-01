//! Renombra la ventana del juego mientras corre.
//!
//! Discord Overlay (`DiscordHook64.dll`) engancha `javaw` si el título
//! **empieza** por "Minecraft". El juego 1.15+ añade " - Multijugador
//! (servidor de terceros)" al entrar a un server y Discord deja de
//! reconocerlo. Forzamos: `Minecraft - Paraguacraft [ver/perfil]`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::core::loaders;

/// Título de ventana: Discord detecta Minecraft; el usuario ve Paraguacraft.
pub fn title_for_launch(mc_version: &str, loader: &str) -> String {
    let ver = mc_version.trim();
    format!("Minecraft - Paraguacraft [{}/{}]", ver, profile_tag(loader))
}

fn profile_tag(loader: &str) -> &'static str {
    match loaders::normalize(loader).as_str() {
        "paraguacraft-pvp" => "PvP",
        "paraguacraft-pvp-modern" => "PvP",
        "paraguacraft-optimized" | "paraguacraft-optimized-neoforge" => "Optimized",
        "fabric-iris" => "Fabric+Iris",
        "fabric" => "Fabric",
        "quilt" => "Quilt",
        "forge" => "Forge",
        "neoforge" => "NeoForge",
        "optifine" => "OptiFine",
        _ => "Vanilla",
    }
}

/// Mantiene el título mientras corre el juego (vanilla, Fabric/Forge/Quilt,
/// OptiFine, PvP, Optimized, modpacks). No reescribe si ya coincide.
pub fn watch_window_title(pid: u32, mc_version: &str, loader: &str, stop: Arc<AtomicBool>) {
    let title = title_for_launch(mc_version, loader);
    std::thread::spawn(move || {
        let mut seen = false;
        while !stop.load(Ordering::Relaxed) {
            #[cfg(target_os = "windows")]
            {
                if apply_title(pid, &title) {
                    seen = true;
                }
            }
            let wait_ms = if seen { 2200 } else { 400 };
            std::thread::sleep(Duration::from_millis(wait_ms));
        }
    });
}

#[cfg(target_os = "windows")]
fn apply_title(pid: u32, new_title: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW, SetWindowTextW};

    let Some(hwnd) = super::game_hwnd::find(pid) else {
        return false;
    };
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len > 0 {
        let mut buf = vec![0u16; (len + 1) as usize];
        unsafe {
            GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        }
        let current = String::from_utf16_lossy(&buf[..len as usize]);
        if current == new_title {
            return true;
        }
    }
    let wide: Vec<u16> = OsStr::new(new_title).encode_wide().chain(Some(0)).collect();
    unsafe {
        let _ = SetWindowTextW(hwnd, wide.as_ptr());
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_starts_with_minecraft() {
        for (ver, loader) in [
            ("1.7.10", "forge"),
            ("1.8.9", "paraguacraft-pvp"),
            ("1.8.9", "optifine"),
            ("1.12.2", "forge"),
            ("1.16.5", "vanilla"),
            ("1.20.1", "fabric"),
            ("1.20.1", "quilt"),
            ("1.21.1", "neoforge"),
            ("1.21.1", "paraguacraft-optimized"),
            ("1.21.1", "fabric-iris"),
            ("1.21.11", "paraguacraft-pvp-modern"),
        ] {
            let t = title_for_launch(ver, loader);
            assert!(t.starts_with("Minecraft - Paraguacraft ["), "{t}");
            assert!(t.contains(ver), "{t}");
        }
    }

    #[test]
    fn pvp_branded_format() {
        assert_eq!(
            title_for_launch("1.8.9", "paraguacraft-pvp"),
            "Minecraft - Paraguacraft [1.8.9/PvP]"
        );
        assert_eq!(
            title_for_launch("1.21.11", "paraguacraft-pvp-modern"),
            "Minecraft - Paraguacraft [1.21.11/PvP]"
        );
        assert_eq!(
            title_for_launch("1.20.1", "forge"),
            "Minecraft - Paraguacraft [1.20.1/Forge]"
        );
        assert_eq!(
            title_for_launch("1.21.1", "paraguacraft-optimized-neoforge"),
            "Minecraft - Paraguacraft [1.21.1/Optimized]"
        );
    }
}
