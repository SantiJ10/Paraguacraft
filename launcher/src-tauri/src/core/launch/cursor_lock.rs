//! Encierra el cursor en la ventana de Minecraft (ClipCursor) mientras el juego
//! tiene el foco y el puntero está oculto. Evita flicks al segundo monitor en
//! borderless. Libera el mouse con ESC, menú (cursor visible) o Alt-Tab.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const POLL_MS: u64 = 40;
const VK_ESCAPE: i32 = 0x1B;
const CURSOR_SHOWING: u32 = 0x0000_0001;

/// Loop de ClipCursor mientras la sesión de juego está viva (`stop` = false).
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
    while !stop.load(Ordering::Relaxed) {
        tick(pid);
        tokio::time::sleep(Duration::from_millis(POLL_MS)).await;
    }
    release();
}

#[cfg(target_os = "windows")]
fn tick(pid: u32) {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetCursorInfo, GetForegroundWindow, GetWindowRect, CURSORINFO,
    };

    let Some(hwnd) = super::game_hwnd::find(pid) else {
        release();
        return;
    };

    unsafe {
        if GetForegroundWindow() != hwnd {
            release();
            return;
        }
        if key_down(VK_ESCAPE) {
            release();
            return;
        }

        let mut info = CURSORINFO {
            cbSize: std::mem::size_of::<CURSORINFO>() as u32,
            flags: 0,
            hCursor: 0 as _,
            ptScreenPos: windows_sys::Win32::Foundation::POINT { x: 0, y: 0 },
        };
        if GetCursorInfo(&mut info) != 0 && (info.flags & CURSOR_SHOWING) != 0 {
            // Inventario / menú de pausa: el cursor se ve → no clip.
            release();
            return;
        }

        let mut rc = windows_sys::Win32::Foundation::RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(hwnd as HWND, &mut rc) == 0 {
            release();
            return;
        }
        // Un píxel de margen interno evita que el clip se escape por el borde DPI.
        if rc.right - rc.left > 4 {
            rc.left += 1;
            rc.right -= 1;
        }
        if rc.bottom - rc.top > 4 {
            rc.top += 1;
            rc.bottom -= 1;
        }
        let _ = windows_sys::Win32::UI::WindowsAndMessaging::ClipCursor(&rc);
    }
}

#[cfg(target_os = "windows")]
fn key_down(vk: i32) -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

#[cfg(target_os = "windows")]
fn release() {
    unsafe {
        let _ = windows_sys::Win32::UI::WindowsAndMessaging::ClipCursor(std::ptr::null());
    }
}
