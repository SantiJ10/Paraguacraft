//! Renombra la ventana al formato que Discord Game Detection ya conoce.
//!
//! Su base de juegos espera `Minecraft 1.8.9` o `Minecraft* 1.21.1`
//! (el `*` = cliente modificado). `Minecraft - Paraguacraft [...]` no dispara
//! el overlay: hay que agregar el juego a mano. 1.15+ añade
//! " - Multijugador (servidor de terceros)" y Discord deja de reconocerlo.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::core::loaders;

/// Título vanilla que Discord asocia a Minecraft Java.
pub fn title_for_launch(mc_version: &str, loader: &str) -> String {
    let ver = mc_version.trim();
    if loaders::normalize(loader) == "vanilla" {
        format!("Minecraft {ver}")
    } else {
        format!("Minecraft* {ver}")
    }
}

fn title_is_ok(current: &str, expected: &str) -> bool {
    current == expected
}

/// Mantiene el título mientras corre el juego. Si 1.21 pone el sufijo de
/// multiplayer, se restaura. No reescribe si ya coincide.
pub fn watch_window_title(pid: u32, mc_version: &str, loader: &str, stop: Arc<AtomicBool>) {
    let title = title_for_launch(mc_version, loader);
    std::thread::spawn(move || {
        let mut wait_ms = 400u64;
        while !stop.load(Ordering::Relaxed) {
            #[cfg(target_os = "windows")]
            {
                wait_ms = match apply_title(pid, &title) {
                    TitleHit::Ok => 2200,
                    _ => 400,
                };
            }
            std::thread::sleep(Duration::from_millis(wait_ms));
        }
    });
}

#[cfg(target_os = "windows")]
enum TitleHit {
    Missing,
    Ok,
    Rewrote,
}

#[cfg(target_os = "windows")]
fn apply_title(pid: u32, new_title: &str) -> TitleHit {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW, SetWindowTextW};

    let Some(hwnd) = super::game_hwnd::find(pid) else {
        return TitleHit::Missing;
    };
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len > 0 {
        let mut buf = vec![0u16; (len + 1) as usize];
        unsafe {
            GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        }
        let current = String::from_utf16_lossy(&buf[..len as usize]);
        if title_is_ok(&current, new_title) {
            return TitleHit::Ok;
        }
    }
    let wide: Vec<u16> = OsStr::new(new_title).encode_wide().chain(Some(0)).collect();
    unsafe {
        let _ = SetWindowTextW(hwnd, wide.as_ptr());
    }
    TitleHit::Rewrote
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discord_native_format() {
        assert_eq!(title_for_launch("1.8.9", "paraguacraft-pvp"), "Minecraft* 1.8.9");
        assert_eq!(
            title_for_launch("1.21.11", "paraguacraft-pvp-modern"),
            "Minecraft* 1.21.11"
        );
        assert_eq!(
            title_for_launch("1.21.1", "paraguacraft-optimized"),
            "Minecraft* 1.21.1"
        );
        assert_eq!(title_for_launch("1.16.5", "vanilla"), "Minecraft 1.16.5");
        assert_eq!(title_for_launch("1.20.1", "fabric"), "Minecraft* 1.20.1");
        assert_eq!(title_for_launch("1.12.2", "forge"), "Minecraft* 1.12.2");
    }

    #[test]
    fn never_uses_custom_brand_in_title() {
        for loader in [
            "paraguacraft-pvp",
            "paraguacraft-pvp-modern",
            "paraguacraft-optimized",
            "fabric",
            "vanilla",
        ] {
            let t = title_for_launch("1.21.1", loader);
            assert!(t.starts_with("Minecraft"), "{t}");
            assert!(!t.to_ascii_lowercase().contains("paraguacraft"), "{t}");
        }
    }
}
