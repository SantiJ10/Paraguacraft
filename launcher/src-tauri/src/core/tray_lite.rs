//! Bandeja del sistema ultra-lite: restaurar launcher mientras el juego corre.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

const TRAY_ID: &str = "paraguacraft-tray";

fn tray_err(e: tauri::Error) -> AppError {
    AppError::msg(e.to_string())
}

pub fn install(app: &AppHandle) -> AppResult<()> {
    let settings: crate::models::AppSettings =
        crate::config::read_json(&crate::core::paths::config_file()).unwrap_or_default();
    if !settings.tray_lite {
        return Ok(());
    }

    let show = MenuItem::with_id(app, "tray_show", "Mostrar launcher", true, None::<&str>)
        .map_err(tray_err)?;
    let quit = MenuItem::with_id(app, "tray_quit", "Salir", true, None::<&str>).map_err(tray_err)?;
    let menu = Menu::with_items(app, &[&show, &quit]).map_err(tray_err)?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| crate::error::AppError::msg("Sin icono de aplicación"))?;

    let app_show = app.clone();
    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip("Paraguacraft")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "tray_show" => show_main(app),
            "tray_quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(&app_show);
            }
        })
        .build(app)
        .map_err(tray_err)?;

    Ok(())
}

pub fn set_playing(app: &AppHandle, playing: bool) {
    let settings: crate::models::AppSettings =
        crate::config::read_json(&crate::core::paths::config_file()).unwrap_or_default();
    if !settings.tray_lite {
        return;
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tip = if playing {
            "Jugando — clic para abrir Paraguacraft"
        } else {
            "Paraguacraft Launcher"
        };
        let _ = tray.set_tooltip(Some(tip));
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

pub fn hide_to_tray(app: &AppHandle) {
    let settings: crate::models::AppSettings =
        crate::config::read_json(&crate::core::paths::config_file()).unwrap_or_default();
    if !settings.tray_lite {
        return;
    }
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}
