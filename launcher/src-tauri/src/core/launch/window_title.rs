//! Renombra la ventana del juego mientras corre.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::core::loaders;

/// Título de ventana según versión y loader.
pub fn title_for_launch(mc_version: &str, loader: &str) -> String {
    let loader = loaders::normalize(loader);
    if loader == "paraguacraft-pvp" && mc_version == "1.8.9" {
        "Paraguacraft PvP".into()
    } else if loader == "paraguacraft-pvp-modern" && mc_version == "1.21.11" {
        "Paraguacraft PvP 1.21.11".into()
    } else {
        format!("Paraguacraft {mc_version}")
    }
}

/// Renombra la ventana del proceso Java **una vez** y sale.
/// Spamear `SetWindowTextW` rompe Discord Overlay y congela la captura al compartir pantalla.
pub fn watch_window_title(pid: u32, mc_version: &str, loader: &str, stop: Arc<AtomicBool>) {
    let title = title_for_launch(mc_version, loader);
    std::thread::spawn(move || {
        // Esperar a que aparezca la ventana de Minecraft (máx ~20s).
        for _ in 0..25 {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            #[cfg(target_os = "windows")]
            {
                if rename_windows_for_pid(pid, &title) {
                    // Segunda pasada breve por si GLFW recrea el título al entrar al menú.
                    std::thread::sleep(Duration::from_millis(1200));
                    if !stop.load(Ordering::Relaxed) {
                        let _ = rename_windows_for_pid(pid, &title);
                    }
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(800));
        }
    });
}

#[cfg(target_os = "windows")]
fn rename_windows_for_pid(pid: u32, new_title: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

    struct Ctx {
        pid: u32,
        title: String,
        changed: AtomicBool,
    }

    unsafe extern "system" fn callback(
        hwnd: windows_sys::Win32::Foundation::HWND,
        lparam: windows_sys::Win32::Foundation::LPARAM,
    ) -> windows_sys::Win32::Foundation::BOOL {
        use windows_sys::Win32::Foundation::TRUE;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
            SetWindowTextW,
        };

        let ctx = &*(lparam as *const Ctx);
        if IsWindowVisible(hwnd) == 0 {
            return TRUE;
        }
        let mut wpid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut wpid);
        if wpid != ctx.pid {
            return TRUE;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return TRUE;
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        let current = String::from_utf16_lossy(&buf[..len as usize]);
        if current == ctx.title {
            return TRUE;
        }
        // No tocar ventanas auxiliares (Discord overlay inject, IME, etc.).
        if !current.contains("Minecraft") && !current.contains("Paraguacraft") {
            return TRUE;
        }
        let wide: Vec<u16> = OsStr::new(&ctx.title).encode_wide().chain(Some(0)).collect();
        SetWindowTextW(hwnd, wide.as_ptr());
        ctx.changed.store(true, AtomicOrdering::Relaxed);
        TRUE
    }

    let ctx = Ctx {
        pid,
        title: new_title.to_string(),
        changed: AtomicBool::new(false),
    };
    let lparam = &ctx as *const Ctx as isize;
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;
        EnumWindows(Some(callback), lparam);
    }
    ctx.changed.load(AtomicOrdering::Relaxed)
}

#[cfg(not(target_os = "windows"))]
fn rename_windows_for_pid(_pid: u32, _new_title: &str) -> bool {
    false
}
