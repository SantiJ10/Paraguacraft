//! Borderless windowed a nivel OS (HWND), agnóstico de LWJGL 2/3.
//!
//! Solo para instancias que **no** son clientes PvP (esos tienen el mod en el
//! menú). No pisa fullscreen exclusivo: si la HWND ya no tiene bordes, no se
//! toca (evita pelear con F11 / Alt-Tab).

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::models::AppSettings;

/// Clientes PvP: el borderless vive en el Mod Menu, no en el launcher.
pub fn pvp_owns_window_mode(loader: &str) -> bool {
    let n = crate::core::loaders::normalize(loader);
    n == "paraguacraft-pvp" || n == "paraguacraft-pvp-modern"
}

/// HWND borderless + cursor lock: otras instancias, si el setting está on y
/// el juego no pidió fullscreen exclusivo. Nunca PvP / Papa / resolución forzada.
pub fn should_apply(loader: &str, game_dir: &Path, settings: &AppSettings) -> bool {
    if settings.papa_mode {
        return false;
    }
    if settings.game_width > 0 && settings.game_height > 0 {
        return false;
    }
    if pvp_owns_window_mode(loader) {
        return false;
    }
    if !settings.os_borderless {
        return false;
    }
    !crate::core::performance::options_want_fullscreen(game_dir)
}

/// Quita `WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | min/max`.
pub const DECORATION_STYLE: u32 = 0x00C0_0000 // WS_CAPTION (BORDER|DLGFRAME)
    | 0x0004_0000 // WS_THICKFRAME
    | 0x0008_0000 // WS_SYSMENU
    | 0x0002_0000 // WS_MINIMIZEBOX
    | 0x0001_0000; // WS_MAXIMIZEBOX

pub const DECORATION_EXSTYLE: u32 = 0x0000_0001 // WS_EX_DLGMODALFRAME
    | 0x0000_0200 // WS_EX_CLIENTEDGE
    | 0x0000_0100 // WS_EX_WINDOWEDGE
    | 0x0002_0000; // WS_EX_STATICEDGE

const WS_POPUP: u32 = 0x8000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;

pub fn is_decorated(style: u32) -> bool {
    (style & 0x00C0_0000) != 0 || (style & 0x0004_0000) != 0
}

pub fn strip_style(style: u32) -> u32 {
    (style & !DECORATION_STYLE) | WS_POPUP | WS_VISIBLE
}

pub fn strip_exstyle(ex: u32) -> u32 {
    ex & !DECORATION_EXSTYLE
}

/// Espera la HWND de `javaw` y la pone borderless a monitor primario.
pub fn watch(pid: u32, stop: Arc<AtomicBool>) {
    #[cfg(target_os = "windows")]
    tauri::async_runtime::spawn(async move {
        watch_loop(pid, stop).await;
    });
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (pid, stop);
    }
}

#[cfg(target_os = "windows")]
async fn watch_loop(pid: u32, stop: Arc<AtomicBool>) {
    let mut interval = tokio::time::interval(Duration::from_millis(350));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let start = Instant::now();
    let mut armed = false;
    while !stop.load(Ordering::Relaxed) {
        interval.tick().await;
        let ok = tokio::task::spawn_blocking(move || apply_for_pid(pid))
            .await
            .ok()
            .unwrap_or(false);
        if ok {
            if !armed {
                armed = true;
                interval = tokio::time::interval(Duration::from_millis(2200));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            }
        } else if !armed && start.elapsed() > Duration::from_secs(75) {
            return;
        }
    }
}

#[cfg(target_os = "windows")]
fn apply_for_pid(pid: u32) -> bool {
    let Some(hwnd) = super::game_hwnd::find(pid) else {
        return false;
    };
    apply_borderless(hwnd)
}

#[cfg(target_os = "windows")]
fn primary_monitor_rect() -> (i32, i32, i32, i32) {
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
    };
    use windows_sys::Win32::Foundation::POINT;
    unsafe {
        let pt = POINT { x: 0, y: 0 };
        let mon = MonitorFromPoint(pt, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            rcMonitor: std::mem::zeroed(),
            rcWork: std::mem::zeroed(),
            dwFlags: 0,
        };
        if !mon.is_null() && GetMonitorInfoW(mon, &mut info) != 0 {
            let r = info.rcMonitor;
            return (r.left, r.top, r.right - r.left, r.bottom - r.top);
        }
    }
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe {
        (
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
        )
    }
}

#[cfg(target_os = "windows")]
fn apply_borderless(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE, HWND_TOP,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    };

    let (x, y, w, h) = primary_monitor_rect();
    if w <= 0 || h <= 0 {
        return false;
    }

    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        // Sin caption/frame: ya es borderless nuestro o fullscreen exclusivo.
        // No hacer SetWindowPos: pelea con F11 y rompe Alt-Tab.
        if !is_decorated(style) {
            return true;
        }
        let new_style = strip_style(style);
        let new_ex = strip_exstyle(ex);
        let already = style == new_style && ex == new_ex && window_covers(hwnd, x, y, w, h);
        if already {
            return true;
        }

        let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, new_style as isize);
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex as isize);
        let flags = SWP_FRAMECHANGED | SWP_SHOWWINDOW | SWP_NOACTIVATE;
        SetWindowPos(hwnd, HWND_TOP, x, y, w, h, flags) != 0
    }
}

#[cfg(target_os = "windows")]
fn window_covers(
    hwnd: windows_sys::Win32::Foundation::HWND,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> bool {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect;
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        if GetWindowRect(hwnd, &mut rc) == 0 {
            return false;
        }
    }
    (rc.left - x).abs() <= 2
        && (rc.top - y).abs() <= 2
        && (rc.right - rc.left - w).abs() <= 4
        && (rc.bottom - rc.top - h).abs() <= 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_caption_and_frame() {
        let decorated = 0x00CF_0000; // typical overlapped window bits
        let clean = strip_style(decorated);
        assert_eq!(clean & DECORATION_STYLE, 0);
        assert_ne!(clean & WS_POPUP, 0);
        assert_eq!(strip_style(clean), clean);
    }

    #[test]
    fn strips_ex_edge() {
        let ex = DECORATION_EXSTYLE | 0x0004_0000; // keep WS_EX_APPWINDOW
        let clean = strip_exstyle(ex);
        assert_eq!(clean & DECORATION_EXSTYLE, 0);
        assert_ne!(clean & 0x0004_0000, 0);
    }

    #[test]
    fn decorated_overlapped_vs_popup() {
        assert!(is_decorated(0x00CF_0000));
        assert!(!is_decorated(strip_style(0x00CF_0000)));
    }

    #[test]
    fn pvp_loaders_own_window_mode() {
        assert!(pvp_owns_window_mode("paraguacraft-pvp"));
        assert!(pvp_owns_window_mode("paraguacraft-pvp-modern"));
        assert!(!pvp_owns_window_mode("fabric"));
        assert!(!pvp_owns_window_mode("forge"));
        assert!(!pvp_owns_window_mode("vanilla"));
        assert!(!pvp_owns_window_mode("paraguacraft-optimized"));
    }

    #[test]
    fn should_apply_skips_pvp_exclusive_and_setting_off() {
        let mut settings = AppSettings::default();
        let dir = std::env::temp_dir().join(format!(
            "pc_bl_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("options.txt"), "fullscreen:false\n").unwrap();
        assert!(!should_apply("paraguacraft-pvp", &dir, &settings));
        assert!(!should_apply("paraguacraft-pvp-modern", &dir, &settings));
        assert!(should_apply("fabric", &dir, &settings));
        std::fs::write(dir.join("options.txt"), "fullscreen:true\n").unwrap();
        assert!(!should_apply("vanilla", &dir, &settings));
        std::fs::write(dir.join("options.txt"), "fullscreen:false\n").unwrap();
        settings.os_borderless = false;
        assert!(!should_apply("forge", &dir, &settings));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
