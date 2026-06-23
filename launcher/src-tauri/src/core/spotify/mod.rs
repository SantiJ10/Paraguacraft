//! Spotify Web API (OAuth + now playing). Espeja el flujo del launcher Python.

mod oauth;

pub use oauth::OAuthPoll;

use std::sync::Mutex;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::keys;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

const SCOPES: &str =
    "user-read-playback-state user-modify-playback-state user-read-currently-playing";

static ACCESS_TOKEN: Mutex<Option<String>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifyStatus {
    pub connected: bool,
    pub client_id: Option<String>,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifySetupInfo {
    pub redirect_uri: String,
    pub dashboard_url: String,
    pub auth_url_preview: Option<String>,
    pub client_id_ok: bool,
    pub client_id_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifyNowPlaying {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub playing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub progress_ms: u64,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub shuffle: bool,
    #[serde(default)]
    pub repeat_state: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifySimpleResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn simple_ok() -> SpotifySimpleResult {
    SpotifySimpleResult {
        ok: true,
        error: None,
    }
}

fn simple_err(msg: impl Into<String>) -> SpotifySimpleResult {
    SpotifySimpleResult {
        ok: false,
        error: Some(msg.into()),
    }
}

fn token() -> Option<String> {
    ACCESS_TOKEN.lock().ok()?.clone()
}

fn set_token(value: Option<String>) {
    if let Ok(mut guard) = ACCESS_TOKEN.lock() {
        *guard = value;
    }
}

fn basic_auth(client_id: &str, client_secret: &str) -> String {
    let raw = format!("{client_id}:{client_secret}");
    base64::engine::general_purpose::STANDARD.encode(raw.as_bytes())
}

fn redirect_uri() -> String {
    keys::read_spotify_redirect_uri()
}

fn auth_url(client_id: &str) -> String {
    let redirect = redirect_uri();
    let challenge = oauth::begin_pkce();
    let scope = SCOPES.replace(' ', "%20");
    format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&show_dialog=true&code_challenge_method=S256&code_challenge={}",
        urlencoding::encode(client_id),
        urlencoding::encode(&redirect),
        scope,
        urlencoding::encode(&challenge),
    )
}

pub fn build_auth_url() -> AppResult<String> {
    let client_id = keys::read_spotify_client_id()
        .ok_or_else(|| AppError::msg("Falta Client ID de Spotify"))?;
    validate_client_id(&client_id)?;
    Ok(auth_url(&client_id))
}

fn validate_client_id(client_id: &str) -> AppResult<()> {
    let id = client_id.trim();
    if id.len() != 32 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::msg(format!(
            "Client ID invalido ({} caracteres). Copialo completo del Dashboard: deben ser 32 caracteres hex.",
            id.len()
        )));
    }
    Ok(())
}

pub fn dashboard_url(client_id: &str) -> String {
    let id = client_id.trim();
    if id.is_empty() {
        return "https://developer.spotify.com/dashboard".into();
    }
    format!("https://developer.spotify.com/dashboard/applications/{id}")
}

pub fn setup_info() -> SpotifySetupInfo {
    let client_id = keys::read_spotify_client_id().unwrap_or_default();
    let id_len = client_id.trim().len();
    let client_id_ok = validate_client_id(&client_id).is_ok();
    SpotifySetupInfo {
        redirect_uri: redirect_uri(),
        dashboard_url: dashboard_url(&client_id),
        auth_url_preview: if client_id_ok {
            build_auth_url().ok()
        } else {
            None
        },
        client_id_ok,
        client_id_length: id_len,
    }
}

pub fn status() -> SpotifyStatus {
    SpotifyStatus {
        connected: token().is_some() || keys::read_spotify_refresh_token().is_some(),
        client_id: keys::read_spotify_client_id(),
        redirect_uri: redirect_uri(),
    }
}

pub fn save_credentials(
    client_id: &str,
    client_secret: &str,
    redirect_uri: Option<&str>,
) -> AppResult<()> {
    validate_client_id(client_id)?;
    let secret = client_secret.trim();
    if secret.len() < 16 {
        return Err(AppError::msg("Client Secret invalido o incompleto"));
    }
    let normalized = redirect_uri
        .map(oauth::normalize_redirect_uri)
        .transpose()?;
    keys::save_spotify_credentials(
        client_id,
        client_secret,
        normalized.as_deref(),
    )
}

pub fn auth_start() -> AppResult<String> {
    oauth::clear_pending();
    oauth::start_listener()?;
    build_auth_url()
}

pub fn poll_auth() -> OAuthPoll {
    oauth::peek_pending()
}

pub fn oauth_error_hint(code: &str) -> &'static str {
    oauth::spotify_error_hint(code)
}

pub async fn connect(state: &AppState, code: &str) -> SpotifySimpleResult {
    let code = code.trim();
    if code.is_empty() {
        return simple_err("Codigo vacio");
    }
    let (Some(client_id), Some(client_secret)) = keys::spotify_credentials() else {
        return simple_err("Faltan credenciales de Spotify Developer App");
    };
    match exchange_code(state, &client_id, &client_secret, code).await {
        Ok(()) => {
            oauth::clear_pending();
            simple_ok()
        }
        Err(e) => simple_err(e.to_string()),
    }
}

/// Comprueba Client ID / Secret contra el endpoint de token de Spotify.
pub async fn validate_app(state: &AppState) -> SpotifySimpleResult {
    let (Some(client_id), Some(client_secret)) = keys::spotify_credentials() else {
        return simple_err("Guarda Client ID y Client Secret primero");
    };
    let redirect = redirect_uri();
    let (http, _guard) = state.net_scope();
    let body = format!(
        "grant_type=authorization_code&code=paraguacraft_validate&redirect_uri={}",
        urlencoding::encode(&redirect)
    );
    let resp = match http
        .post("https://accounts.spotify.com/api/token")
        .header(
            "Authorization",
            format!("Basic {}", basic_auth(&client_id, &client_secret)),
        )
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return simple_err(format!("Sin conexion a Spotify: {e}")),
    };
    let status = resp.status();
    let data: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return simple_err(format!("Respuesta invalida de Spotify: {e}")),
    };
    let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("");
    match err {
        "invalid_grant" => simple_ok(),
        "invalid_client" => simple_err("Client ID o Client Secret incorrectos"),
        "" if status.is_success() => simple_ok(),
        _ => {
            let msg = data
                .get("error_description")
                .or_else(|| data.get("error"))
                .and_then(|v| v.as_str())
                .unwrap_or("Credenciales rechazadas");
            simple_err(msg)
        }
    }
}

async fn exchange_code(
    state: &AppState,
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> AppResult<()> {
    let (http, _guard) = state.net_scope();
    let redirect = redirect_uri();
    let mut body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}",
        urlencoding::encode(code),
        urlencoding::encode(&redirect),
        urlencoding::encode(client_id),
    );
    if let Some(verifier) = oauth::take_pkce_verifier() {
        body.push_str("&code_verifier=");
        body.push_str(&urlencoding::encode(&verifier));
    }
    let resp = http
        .post("https://accounts.spotify.com/api/token")
        .header(
            "Authorization",
            format!("Basic {}", basic_auth(client_id, client_secret)),
        )
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Spotify token: {e}")))?;
    let status = resp.status();
    let data: Value = resp
        .json()
        .await
        .map_err(|e| AppError::msg(format!("Spotify token JSON: {e}")))?;
    if !status.is_success() {
        let msg = data
            .get("error_description")
            .or_else(|| data.get("error"))
            .and_then(|v| v.as_str())
            .unwrap_or("Error al intercambiar el codigo");
        return Err(AppError::msg(format!(
            "{} (HTTP {}). Verifica que el Redirect URI del launcher coincida con Spotify Dashboard.",
            msg,
            status
        )));
    }
    apply_token_response(&data)
}

fn apply_token_response(data: &Value) -> AppResult<()> {
    let access = data
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AppError::msg(
                data.get("error_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Error de autenticacion Spotify"),
            )
        })?;
    set_token(Some(access.to_string()));
    if let Some(refresh) = data.get("refresh_token").and_then(|v| v.as_str()) {
        keys::save_spotify_refresh_token(Some(refresh))?;
    }
    Ok(())
}

async fn refresh_token(state: &AppState) -> bool {
    let refresh = match keys::read_spotify_refresh_token() {
        Some(v) => v,
        None => return false,
    };
    let (Some(client_id), Some(client_secret)) = keys::spotify_credentials() else {
        return false;
    };
    let (http, _guard) = state.net_scope();
    let body = format!(
        "grant_type=refresh_token&refresh_token={}",
        urlencoding::encode(&refresh)
    );
    let resp = match http
        .post("https://accounts.spotify.com/api/token")
        .header(
            "Authorization",
            format!("Basic {}", basic_auth(&client_id, &client_secret)),
        )
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return false,
    };
    let data: Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return false,
    };
    if apply_token_response(&data).is_err() {
        let _ = keys::save_spotify_refresh_token(None);
        set_token(None);
        return false;
    }
    true
}

pub async fn try_autoconnect(state: &AppState) -> SpotifySimpleResult {
    if token().is_some() {
        return simple_ok();
    }
    if keys::read_spotify_refresh_token().is_none() {
        return simple_err("Sin sesion guardada");
    }
    if refresh_token(state).await {
        simple_ok()
    } else {
        simple_err("Sesion expirada, autoriza nuevamente")
    }
}

pub fn disconnect() -> AppResult<()> {
    set_token(None);
    keys::save_spotify_refresh_token(None)
}

async fn authed_get(state: &AppState, url: &str) -> Result<reqwest::Response, SpotifyNowPlaying> {
    let access = token().ok_or_else(|| SpotifyNowPlaying {
        ok: false,
        error: Some("No conectado".into()),
        playing: false,
        title: None,
        artist: None,
        album: None,
        image_url: None,
        progress_ms: 0,
        duration_ms: 0,
        shuffle: false,
        repeat_state: "off".into(),
    })?;
    let (http, _guard) = state.net_scope();
    let resp = http
        .get(url)
        .header("Authorization", format!("Bearer {access}"))
        .send()
        .await
        .map_err(|e| SpotifyNowPlaying {
            ok: false,
            error: Some(e.to_string()),
            playing: false,
            title: None,
            artist: None,
            album: None,
            image_url: None,
            progress_ms: 0,
            duration_ms: 0,
            shuffle: false,
            repeat_state: "off".into(),
        })?;
    Ok(resp)
}

pub async fn now_playing(state: &AppState) -> SpotifyNowPlaying {
    let mut resp = match authed_get(state, "https://api.spotify.com/v1/me/player/currently-playing").await
    {
        Ok(r) => r,
        Err(e) => return e,
    };
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        if refresh_token(state).await {
            resp = match authed_get(
                state,
                "https://api.spotify.com/v1/me/player/currently-playing",
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return e,
            };
        } else {
            set_token(None);
            return SpotifyNowPlaying {
                ok: false,
                error: Some("Sesion expirada, reconecta".into()),
                playing: false,
                title: None,
                artist: None,
                album: None,
                image_url: None,
                progress_ms: 0,
                duration_ms: 0,
                shuffle: false,
                repeat_state: "off".into(),
            };
        }
    }
    if resp.status() == reqwest::StatusCode::NO_CONTENT {
        return SpotifyNowPlaying {
            ok: true,
            error: None,
            playing: false,
            title: None,
            artist: None,
            album: None,
            image_url: None,
            progress_ms: 0,
            duration_ms: 0,
            shuffle: false,
            repeat_state: "off".into(),
        };
    }
    if !resp.status().is_success() {
        return SpotifyNowPlaying {
            ok: false,
            error: Some(format!("Spotify HTTP {}", resp.status())),
            playing: false,
            title: None,
            artist: None,
            album: None,
            image_url: None,
            progress_ms: 0,
            duration_ms: 0,
            shuffle: false,
            repeat_state: "off".into(),
        };
    }
    let data: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            return SpotifyNowPlaying {
                ok: false,
                error: Some(e.to_string()),
                playing: false,
                title: None,
                artist: None,
                album: None,
                image_url: None,
                progress_ms: 0,
                duration_ms: 0,
                shuffle: false,
                repeat_state: "off".into(),
            }
        }
    };
    let item = data.get("item").cloned().unwrap_or(Value::Null);
    let artists = item
        .get("artists")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    SpotifyNowPlaying {
        ok: true,
        error: None,
        playing: data.get("is_playing").and_then(|v| v.as_bool()).unwrap_or(false),
        title: item.get("name").and_then(|v| v.as_str()).map(str::to_string),
        artist: if artists.is_empty() {
            None
        } else {
            Some(artists)
        },
        album: item
            .get("album")
            .and_then(|a| a.get("name"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        image_url: item
            .get("album")
            .and_then(|a| a.get("images"))
            .and_then(|imgs| imgs.as_array())
            .and_then(|arr| arr.first())
            .and_then(|img| img.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        progress_ms: data
            .get("progress_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        duration_ms: item.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(1),
        shuffle: data
            .get("shuffle_state")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        repeat_state: data
            .get("repeat_state")
            .and_then(|v| v.as_str())
            .unwrap_or("off")
            .to_string(),
    }
}

pub async fn control(state: &AppState, action: &str) -> SpotifySimpleResult {
    if token().is_none() {
        return simple_err("No conectado");
    }
    let (method, url) = match action {
        "play" => ("PUT", "https://api.spotify.com/v1/me/player/play"),
        "pause" => ("PUT", "https://api.spotify.com/v1/me/player/pause"),
        "next" => ("POST", "https://api.spotify.com/v1/me/player/next"),
        "prev" | "previous" => ("POST", "https://api.spotify.com/v1/me/player/previous"),
        _ => return simple_err("Accion desconocida"),
    };
    if !player_request(state, method, url).await && refresh_token(state).await {
        if !player_request(state, method, url).await {
            return simple_err("No se pudo controlar la reproduccion");
        }
    }
    simple_ok()
}

async fn player_request(state: &AppState, method: &str, url: &str) -> bool {
    let access = match token() {
        Some(v) => v,
        None => return false,
    };
    let (http, _guard) = state.net_scope();
    let req = http
        .request(
            reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::PUT),
            url,
        )
        .header("Authorization", format!("Bearer {access}"));
    match req.send().await {
        Ok(r) => r.status().is_success() || r.status() == reqwest::StatusCode::NO_CONTENT,
        Err(_) => false,
    }
}

pub async fn set_shuffle(state: &AppState, enabled: bool) -> SpotifySimpleResult {
    if token().is_none() {
        return simple_err("No conectado");
    }
    let url = format!(
        "https://api.spotify.com/v1/me/player/shuffle?state={}",
        if enabled { "true" } else { "false" }
    );
    if !player_request(state, "PUT", &url).await && refresh_token(state).await {
        let _ = player_request(state, "PUT", &url).await;
    }
    simple_ok()
}

pub async fn set_repeat(state: &AppState, mode: &str) -> SpotifySimpleResult {
    if token().is_none() {
        return simple_err("No conectado");
    }
    let state_val = match mode {
        "track" | "context" | "off" => mode,
        _ => "off",
    };
    let url = format!("https://api.spotify.com/v1/me/player/repeat?state={state_val}");
    if !player_request(state, "PUT", &url).await && refresh_token(state).await {
        let _ = player_request(state, "PUT", &url).await;
    }
    simple_ok()
}
