//! Lanzamiento Bedrock en Windows (UWP / Xbox App).

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager};

use crate::config;
use crate::core::extras::discord_rpc;
use crate::core::game_session;
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;

const BEDROCK_TITLE: &str = "Paraguacraft Bedrock";
const PROC_NAMES: [&str; 3] = ["minecraft.windows.exe", "minecraftuwp.exe", "minecraftpe.exe"];

fn is_bedrock_process_name(name: &str) -> bool {
    let n = name.to_lowercase();
    PROC_NAMES.contains(&n.as_str())
        || n.contains("minecraftwindowsbeta")
        || (n.starts_with("minecraft") && n.contains("windows") && n.ends_with(".exe"))
}

fn is_bedrock_process(p: &sysinfo::Process) -> bool {
    if is_bedrock_process_name(&p.name().to_string_lossy()) {
        return true;
    }
    let Some(path) = p.exe() else {
        return false;
    };
    let s = path.to_string_lossy().to_lowercase().replace('/', "\\");
    s.ends_with("\\minecraft.windows.exe")
        || s.contains("\\microsoft.minecraftuwp")
        || s.contains("\\microsoft.minecraftwindowsbeta")
}

fn collect_bedrock_pids(sys: &System) -> Vec<u32> {
    sys.processes()
        .iter()
        .filter_map(|(pid, p)| is_bedrock_process(p).then_some(pid.as_u32()))
        .collect()
}

fn bedrock_game_pid() -> Option<u32> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut fallback = None;
    for (pid, p) in sys.processes() {
        if !is_bedrock_process(p) {
            continue;
        }
        let name = p.name().to_string_lossy().to_lowercase();
        if name == "minecraft.windows.exe" {
            return Some(pid.as_u32());
        }
        fallback = Some(pid.as_u32());
    }
    fallback
}

fn is_bedrock_menu_title(title: &str) -> bool {
    let l = title.trim().to_lowercase();
    l == "minecraft"
        || l == "minecraft: bedrock edition"
        || l == "minecraft: windows 10 edition"
        || l == "minecraft windows 10 edition"
        || l.contains("minecraft for windows")
}
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn mojang_dir() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    let packages = PathBuf::from(local).join("Packages");
    if !packages.is_dir() {
        return None;
    }
    let Ok(entries) = std::fs::read_dir(packages) else {
        return None;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("Microsoft.MinecraftUWP_")
            || name.starts_with("Microsoft.MinecraftWindowsBeta_")
        {
            let mojang = entry.path().join("LocalState/games/com.mojang");
            if mojang.is_dir() {
                return Some(mojang);
            }
        }
    }
    None
}

pub fn has_bedrock_data() -> bool {
    mojang_dir().is_some()
}

/// Cierra el proceso UWP para poder (des)registrar el paquete.
pub fn kill_bedrock() {
    use std::os::windows::process::CommandExt;
    for name in ["Minecraft.Windows.exe", "minecraftuwp.exe", "minecraftpe.exe"] {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", name, "/T"])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

pub fn find_exe_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from(r"C:\XboxGames\Minecraft for Windows\Content\Minecraft.Windows.exe"),
        PathBuf::from(r"C:\XboxGames\Minecraft for Windows\Content\gamelaunchhelper.exe"),
    ];
    if let Ok(prog) = std::env::var("ProgramFiles") {
        let base = PathBuf::from(prog).join("WindowsApps");
        if base.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&base) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains("Minecraft") && name.contains("Windows") {
                        let candidate = entry.path().join("Minecraft.Windows.exe");
                        if candidate.is_file() {
                            paths.insert(0, candidate);
                        }
                    }
                }
            }
        }
    }
    paths
}

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn try_shell_open(aumid: &str) -> bool {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let file = wide(aumid);
    let op = wide("open");
    let ret = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL as i32,
        )
    };
    ret as isize > 32
}

const KNOWN_AUMIDS: [&str; 2] = [
    r"shell:AppsFolder\Microsoft.MinecraftUWP_8wekyb3d8bbwe!App",
    r"shell:AppsFolder\Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe!App",
];

fn powershell_aumids() -> Vec<String> {
    use std::os::windows::process::CommandExt;

    let mut aumids = Vec::new();
    if let Ok(out) = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-AppxPackage *Minecraft* | Select-Object -ExpandProperty PackageFamilyName",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        if out.status.success() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let pfn = line.trim();
                if !pfn.is_empty() {
                    aumids.push(format!(r"shell:AppsFolder\{pfn}!App"));
                }
            }
        }
    }
    aumids
}

pub fn open_bedrock_app(_username: &str) -> AppResult<()> {
    for aumid in KNOWN_AUMIDS {
        if try_shell_open(aumid) {
            return Ok(());
        }
    }

    for aumid in powershell_aumids() {
        if KNOWN_AUMIDS.contains(&aumid.as_str()) {
            continue;
        }
        if try_shell_open(&aumid) {
            return Ok(());
        }
    }

    for path in find_exe_paths() {
        if path.is_file() {
            use std::os::windows::process::CommandExt;
            std::process::Command::new(&path)
                .current_dir(path.parent().unwrap_or(Path::new(".")))
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(AppError::Io)?;
            return Ok(());
        }
    }

    Err(AppError::msg(
        "No se encontró Minecraft: Bedrock Edition. Instalalo desde Xbox / Microsoft Store.",
    ))
}

/// Vigila la sesión Bedrock: estado, RPC (cualquier versión Store/extraída), minimizar launcher.
pub fn watch_session(
    app: AppHandle,
    username: String,
    close_on_launch: bool,
    version: Option<String>,
) {
    std::thread::spawn(move || {
        let settings = config::read_json::<AppSettings>(&crate::core::paths::config_file())
            .unwrap_or_default();
        let display_user = username.replace(" [PREMIUM]", "");
        let ver = version.as_deref().map(str::trim).filter(|s| !s.is_empty());

        if settings.discord_rpc {
            discord_rpc::set_bedrock_loading(
                &display_user,
                ver,
                settings.discord_rpc_version,
                settings.discord_rpc_time,
            );
        }

        emit_status(&app, "Detectando Bedrock…");

        if !wait_for_bedrock_process(Duration::from_secs(180)) {
            if settings.discord_rpc {
                discord_rpc::set_launcher_idle(&display_user);
            }
            emit_status(&app, "");
            return;
        }

        if let Some(pid) = bedrock_game_pid() {
            discord_rpc::bind_game_pid(pid);
        }

        emit_status(&app, "Bedrock activo");
        let _ = app.emit("bedrock://started", serde_json::json!({}));
        game_session::set_running(true);

        if settings.discord_rpc {
            discord_rpc::set_bedrock_session(
                &display_user,
                ver,
                Some("En el menú"),
                settings.discord_rpc_version,
                settings.discord_rpc_time,
            );
        }

        if let Some(win) = app.get_webview_window("main") {
            if close_on_launch {
                let _ = win.hide();
            } else {
                let _ = win.minimize();
            }
        }

        let mut last_rename = std::time::Instant::now();
        let mut last_rpc_state = String::from("En el menú");
        let mut last_rpc_sent = String::new();
        loop {
            if !bedrock_running() {
                break;
            }
            if settings.discord_rpc {
                let title = read_bedrock_window_title();
                let state = bedrock_rpc_state(title.as_deref(), &mut last_rpc_state);
                if state != last_rpc_sent {
                    last_rpc_sent = state.clone();
                    discord_rpc::set_bedrock_session(
                        &display_user,
                        ver,
                        Some(&state),
                        settings.discord_rpc_version,
                        settings.discord_rpc_time,
                    );
                }
            }
            // Renombrar poco: spamear SetWindowText congela Discord al compartir.
            if last_rename.elapsed() >= Duration::from_secs(5) {
                rename_bedrock_windows(BEDROCK_TITLE);
                last_rename = std::time::Instant::now();
            }
            std::thread::sleep(Duration::from_millis(500));
        }

        game_session::set_running(false);
        discord_rpc::clear_game_pid();

        if let Some(win) = app.get_webview_window("main") {
            if close_on_launch {
                let _ = win.show();
            } else {
                let _ = win.unminimize();
            }
        }

        if settings.discord_rpc {
            discord_rpc::set_launcher_idle(&display_user);
        }

        emit_status(&app, "Bedrock cerrado");
        let _ = app.emit("bedrock://exited", serde_json::json!({}));
    });
}

fn emit_status(app: &AppHandle, message: &str) {
    let _ = app.emit(
        "bedrock://status",
        serde_json::json!({ "message": message }),
    );
}

fn wait_for_bedrock_process(timeout: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if bedrock_running() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    false
}

fn bedrock_running() -> bool {
    bedrock_game_pid().is_some()
}

fn bedrock_rpc_state(title: Option<&str>, last: &mut String) -> String {
    match title {
        None => last.clone(),
        Some(t) if t.contains("Paraguacraft") => last.clone(),
        Some(t) if is_bedrock_menu_title(t) => {
            let s = "En el menú".to_string();
            *last = s.clone();
            s
        }
        Some(t) => {
            let s = t.trim().to_string();
            if !s.is_empty() {
                *last = s.clone();
            }
            last.clone()
        }
    }
}

fn read_bedrock_window_title() -> Option<String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    struct Ctx {
        pids: Vec<u32>,
        title: Option<String>,
    }

    let bedrock_pids: Vec<u32> = collect_bedrock_pids(&sys);

    let mut ctx = Ctx {
        pids: bedrock_pids,
        title: None,
    };

    unsafe extern "system" fn read_callback(
        hwnd: windows_sys::Win32::Foundation::HWND,
        lparam: windows_sys::Win32::Foundation::LPARAM,
    ) -> windows_sys::Win32::Foundation::BOOL {
        use windows_sys::Win32::Foundation::TRUE;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        };

        let ctx = &mut *(lparam as *mut Ctx);
        if ctx.title.is_some() {
            return TRUE;
        }
        if IsWindowVisible(hwnd) == 0 {
            return TRUE;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if !ctx.pids.contains(&pid) {
            return TRUE;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return TRUE;
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        let current = String::from_utf16_lossy(&buf[..len as usize]);
        if current.is_empty() {
            return TRUE;
        }
        ctx.title = Some(current);
        TRUE
    }

    let lparam = &mut ctx as *mut Ctx as isize;
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;
        EnumWindows(Some(read_callback), lparam);
    }
    ctx.title
}

fn rename_bedrock_windows(new_title: &str) {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    struct Ctx {
        title: String,
        pids: Vec<u32>,
    }

    let bedrock_pids: Vec<u32> = collect_bedrock_pids(&sys);

    let ctx = Ctx {
        title: new_title.to_string(),
        pids: bedrock_pids,
    };

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
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if !ctx.pids.contains(&pid) {
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

    let lparam = &ctx as *const Ctx as isize;
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;
        EnumWindows(Some(callback), lparam);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_current_and_legacy_process_names() {
        assert!(is_bedrock_process_name("Minecraft.Windows.exe"));
        assert!(is_bedrock_process_name("minecraftuwp.exe"));
        assert!(is_bedrock_process_name("MinecraftPE.exe"));
        assert!(is_bedrock_process_name("MinecraftWindowsBeta.exe"));
        assert!(!is_bedrock_process_name("MinecraftLauncher.exe"));
        assert!(!is_bedrock_process_name("javaw.exe"));
    }

    #[test]
    fn old_win10_titles_count_as_menu() {
        assert!(is_bedrock_menu_title("Minecraft"));
        assert!(is_bedrock_menu_title("Minecraft: Bedrock Edition"));
        assert!(is_bedrock_menu_title("Minecraft: Windows 10 Edition"));
        assert!(is_bedrock_menu_title("Minecraft for Windows"));
        assert!(!is_bedrock_menu_title("Mi mundo"));
    }

    #[test]
    fn rpc_state_keeps_world_name_on_old_titles() {
        let mut last = String::from("En el menú");
        assert_eq!(
            bedrock_rpc_state(Some("Minecraft: Windows 10 Edition"), &mut last),
            "En el menú"
        );
        assert_eq!(bedrock_rpc_state(Some("Survival"), &mut last), "Survival");
    }
}

