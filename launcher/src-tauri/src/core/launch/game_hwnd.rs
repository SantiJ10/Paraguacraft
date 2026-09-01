//! HWND del `javaw` del juego, agnóstico de versión/loader/modpack.
//!
//! Discord y el borderless no pueden filtrar solo por "Minecraft" en el título:
//! 1.21+ pone "Multijugador (servidor de terceros)", los modpacks usan su marca,
//! y LWJGL 2/3 arrancan con títulos vacíos o "LWJGL".

use std::sync::atomic::Ordering;

pub fn is_excluded_title(title: &str) -> bool {
    let t = title.to_ascii_lowercase();
    t.contains("discord")
        || t.contains("ime")
        || t.contains("msctf")
        || t.contains("nvidia")
        || t.contains("geforce overlay")
}

/// Ventana visible del PID, ya inicializada (cliente ≥ 640×360).
#[cfg(target_os = "windows")]
pub fn find(pid: u32) -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::Foundation::{HWND, LPARAM, TRUE};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    };

    struct Ctx {
        pid: u32,
        hwnd: std::sync::atomic::AtomicIsize,
    }

    unsafe extern "system" fn callback(
        hwnd: HWND,
        lparam: LPARAM,
    ) -> windows_sys::Win32::Foundation::BOOL {
        let ctx = &*(lparam as *const Ctx);
        if IsWindowVisible(hwnd) == 0 {
            return TRUE;
        }
        let mut wpid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut wpid);
        if wpid != ctx.pid {
            return TRUE;
        }
        if !client_is_ready(hwnd) {
            return TRUE;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            if is_excluded_title(&title) {
                return TRUE;
            }
        }
        ctx.hwnd.store(hwnd as isize, Ordering::Relaxed);
        0
    }

    let ctx = Ctx {
        pid,
        hwnd: std::sync::atomic::AtomicIsize::new(0),
    };
    let lparam = &ctx as *const Ctx as isize;
    unsafe {
        let _ = EnumWindows(Some(callback), lparam);
    }
    let hwnd = ctx.hwnd.load(Ordering::Relaxed);
    if hwnd == 0 {
        None
    } else {
        Some(hwnd as HWND)
    }
}

#[cfg(target_os = "windows")]
pub fn client_is_ready(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect;
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        if GetClientRect(hwnd, &mut rc) == 0 {
            return false;
        }
    }
    let w = rc.right - rc.left;
    let h = rc.bottom - rc.top;
    w >= 640 && h >= 360
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_minecraft_and_modpack_titles() {
        assert!(!is_excluded_title("Minecraft* 1.21.11 - Multijugador (servidor de terceros)"));
        assert!(!is_excluded_title("FTB StoneBlock"));
        assert!(!is_excluded_title("All the Mods 9"));
        assert!(!is_excluded_title("LWJGL"));
        assert!(is_excluded_title("Discord Overlay"));
    }
}
