//! Renombra la ventana del juego a «Paraguacraft X.X.X» (espejo de `_hilo_ninja_renombrar`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Inicia un hilo que renombra la ventana del proceso Java mientras el juego corre.
pub fn watch_window_title(pid: u32, mc_version: &str, stop: Arc<AtomicBool>) {
    let title = format!("Paraguacraft {mc_version}");
    std::thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            #[cfg(target_os = "windows")]
            rename_windows_for_pid(pid, &title);
            std::thread::sleep(Duration::from_millis(300));
        }
    });
}

#[cfg(target_os = "windows")]
fn rename_windows_for_pid(pid: u32, new_title: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    struct Ctx {
        pid: u32,
        title: String,
    }

    unsafe extern "system" fn callback(hwnd: windows_sys::Win32::Foundation::HWND, lparam: windows_sys::Win32::Foundation::LPARAM) -> windows_sys::Win32::Foundation::BOOL {
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
        if !current.contains("Minecraft") || current.contains("Paraguacraft") {
            return TRUE;
        }
        let wide: Vec<u16> = OsStr::new(&ctx.title).encode_wide().chain(Some(0)).collect();
        SetWindowTextW(hwnd, wide.as_ptr());
        TRUE
    }

    let ctx = Ctx {
        pid,
        title: new_title.to_string(),
    };
    let lparam = &ctx as *const Ctx as isize;
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;
        EnumWindows(Some(callback), lparam);
    }
}

#[cfg(not(target_os = "windows"))]
fn rename_windows_for_pid(_pid: u32, _new_title: &str) {}
