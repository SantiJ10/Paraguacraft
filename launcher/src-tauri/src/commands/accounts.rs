//! Comandos de cuentas: Microsoft (device-code + browser) y offline.

use tauri::State;

use crate::core::accounts::{self, microsoft};
use crate::error::AppResult;
use crate::models::{Account, DeviceCodeStart};
use crate::state::AppState;

#[tauri::command]
pub fn get_accounts() -> Vec<Account> {
    accounts::list()
}

#[tauri::command]
pub fn set_active_account(id: String) -> AppResult<Vec<Account>> {
    let list = accounts::set_active(&id)?;
    let settings: crate::models::AppSettings =
        crate::config::read_json(&crate::core::paths::config_file()).unwrap_or_default();
    if settings.discord_rpc {
        if let Some(acc) = list.iter().find(|a| a.active) {
            crate::core::extras::discord_rpc::set_launcher_idle(&acc.username);
        }
    }
    Ok(list)
}

#[tauri::command]
pub fn add_offline_account(username: String) -> AppResult<Vec<Account>> {
    accounts::add_offline(&username)
}

#[tauri::command]
pub fn remove_account(id: String) -> AppResult<Vec<Account>> {
    accounts::remove(&id)
}

/// URL de login Microsoft por navegador (auth-code). La UI la abre con el shell.
#[tauri::command]
pub fn ms_login_url() -> String {
    microsoft::build_login_url()
}

/// Completa el flujo browser auth-code con el `code` de la URL de redireccion.
#[tauri::command]
pub async fn ms_login_complete_code(
    state: State<'_, AppState>,
    code: String,
) -> AppResult<Vec<Account>> {
    let http = state.client();
    let login = microsoft::complete_auth_code(&http, &code).await?;
    let accounts = accounts::upsert_microsoft(&login)?;
    state.shutdown_network();
    Ok(accounts)
}

/// Inicia el device-code flow (QR / microsoft.com/link).
#[tauri::command]
pub async fn ms_login_start(state: State<'_, AppState>) -> AppResult<DeviceCodeStart> {
    let http = state.client();
    let (start, auth) = microsoft::device_start(&http).await?;
    *state.ms_device.lock().unwrap() = Some(auth);
    Ok(start)
}

/// Poll del device-code. Devuelve `None` si aun esta pendiente, o la lista de
/// cuentas (con la nueva activa) cuando el usuario autoriza.
#[tauri::command]
pub async fn ms_login_poll(state: State<'_, AppState>) -> AppResult<Option<Vec<Account>>> {
    let http = state.client();
    let auth = state.ms_device.lock().unwrap().clone();
    let Some(auth) = auth else {
        return Ok(None);
    };
    match microsoft::device_poll(&http, &auth).await {
        Ok(microsoft::PollOutcome::Pending) => Ok(None),
        Ok(microsoft::PollOutcome::Done(login)) => {
            let accounts = accounts::upsert_microsoft(&login)?;
            *state.ms_device.lock().unwrap() = None;
            state.shutdown_network();
            Ok(Some(accounts))
        }
        Err(e) => {
            *state.ms_device.lock().unwrap() = None;
            state.shutdown_network();
            Err(e)
        }
    }
}

/// Refresca el token de la cuenta activa bajo demanda (p. ej. antes de jugar).
#[tauri::command]
pub async fn ensure_account_token(state: State<'_, AppState>, id: String) -> AppResult<bool> {
    let http = state.client();
    let res = accounts::ensure_valid_token(&http, &id).await.map(|_| true);
    state.shutdown_network();
    res
}
