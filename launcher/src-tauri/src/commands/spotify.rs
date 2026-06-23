//! Comandos IPC de Spotify.

use tauri::{AppHandle, State};
use tauri_plugin_shell::ShellExt;

use crate::core::spotify;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub fn spotify_status() -> spotify::SpotifyStatus {
    spotify::status()
}

#[tauri::command]
pub fn spotify_save_credentials(
    client_id: String,
    client_secret: String,
    redirect_uri: Option<String>,
) -> AppResult<()> {
    spotify::save_credentials(&client_id, &client_secret, redirect_uri.as_deref())
}

#[tauri::command]
pub async fn spotify_validate_app(state: State<'_, AppState>) -> AppResult<spotify::SpotifySimpleResult> {
    Ok(spotify::validate_app(&state).await)
}

#[tauri::command]
pub async fn spotify_auth_start(app: AppHandle) -> AppResult<String> {
    let url = spotify::auth_start()?;
    tokio::time::sleep(std::time::Duration::from_millis(350)).await;
    app.shell()
        .open(&url, None)
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    Ok(url)
}

#[tauri::command]
pub fn spotify_poll_auth() -> spotify::OAuthPoll {
    spotify::poll_auth()
}

#[tauri::command]
pub fn spotify_oauth_hint(error: String) -> String {
    spotify::oauth_error_hint(&error).to_string()
}

#[tauri::command]
pub fn spotify_setup_info() -> spotify::SpotifySetupInfo {
    spotify::setup_info()
}

#[tauri::command]
pub async fn spotify_open_dashboard(app: AppHandle, client_id: String) -> AppResult<()> {
    let id = client_id.trim();
    let url = spotify::dashboard_url(id);
    app.shell()
        .open(&url, None)
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn spotify_connect(state: State<'_, AppState>, code: String) -> AppResult<spotify::SpotifySimpleResult> {
    Ok(spotify::connect(&state, &code).await)
}

#[tauri::command]
pub async fn spotify_try_autoconnect(state: State<'_, AppState>) -> AppResult<spotify::SpotifySimpleResult> {
    Ok(spotify::try_autoconnect(&state).await)
}

#[tauri::command]
pub fn spotify_disconnect() -> AppResult<()> {
    spotify::disconnect()
}

#[tauri::command]
pub async fn spotify_now_playing(state: State<'_, AppState>) -> AppResult<spotify::SpotifyNowPlaying> {
    Ok(spotify::now_playing(&state).await)
}

#[tauri::command]
pub async fn spotify_control(state: State<'_, AppState>, action: String) -> AppResult<spotify::SpotifySimpleResult> {
    Ok(spotify::control(&state, &action).await)
}

#[tauri::command]
pub async fn spotify_shuffle(state: State<'_, AppState>, enabled: bool) -> AppResult<spotify::SpotifySimpleResult> {
    Ok(spotify::set_shuffle(&state, enabled).await)
}

#[tauri::command]
pub async fn spotify_repeat(state: State<'_, AppState>, mode: String) -> AppResult<spotify::SpotifySimpleResult> {
    Ok(spotify::set_repeat(&state, &mode).await)
}
