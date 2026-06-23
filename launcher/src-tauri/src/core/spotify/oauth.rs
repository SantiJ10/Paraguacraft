//! Servidor local minimo para capturar el redirect OAuth de Spotify (puerto 8888).
//! Espeja el flujo del launcher Python: sin PKCE, escucha hasta recibir /callback.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthPoll {
    pub ready: bool,
    pub code: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

static PENDING: Mutex<Option<OAuthPoll>> = Mutex::new(None);
static PKCE_VERIFIER: Mutex<Option<String>> = Mutex::new(None);
static LISTENER_ACTIVE: AtomicBool = AtomicBool::new(false);
static LISTENER_HANDLES: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

pub fn clear_pending() {
    if let Ok(mut guard) = PENDING.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = PKCE_VERIFIER.lock() {
        *guard = None;
    }
}

pub fn begin_pkce() -> String {
    let verifier = generate_verifier();
    if let Ok(mut guard) = PKCE_VERIFIER.lock() {
        *guard = Some(verifier.clone());
    }
    pkce_challenge(&verifier)
}

pub fn take_pkce_verifier() -> Option<String> {
    PKCE_VERIFIER.lock().ok()?.take()
}

fn generate_verifier() -> String {
    use uuid::Uuid;
    let a = Uuid::new_v4().simple().to_string();
    let b = Uuid::new_v4().simple().to_string();
    format!("pc{a}{b}")
}

fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

/// Lee el estado OAuth sin consumirlo (el poll del frontend puede repetirse).
pub fn peek_pending() -> OAuthPoll {
    PENDING
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_default()
}

pub fn take_pending() -> Option<OAuthPoll> {
    PENDING.lock().ok()?.take()
}

pub fn start_listener() -> AppResult<()> {
    if LISTENER_ACTIVE.load(Ordering::SeqCst) {
        let finished = LISTENER_HANDLES
            .lock()
            .map(|g| g.iter().all(|h| h.is_finished()))
            .unwrap_or(true);
        if !finished {
            return Ok(());
        }
        LISTENER_ACTIVE.store(false, Ordering::SeqCst);
    }

    let addrs = ["127.0.0.1:8888", "[::1]:8888"];
    let mut started = false;
    let mut handles = Vec::new();

    for addr in addrs {
        match TcpListener::bind(addr) {
            Ok(listener) => {
                listener
                    .set_nonblocking(true)
                    .map_err(|e| AppError::msg(e.to_string()))?;
                started = true;
                handles.push(spawn_listener_thread(listener));
            }
            Err(_) if addr.starts_with("[::1]") => {
                // IPv6 loopback opcional (algunos Windows no lo tienen).
            }
            Err(e) => {
                return Err(AppError::msg(format!(
                    "No se pudo abrir el puerto 8888 ({e}). Cierra otras apps que lo usen."
                )));
            }
        }
    }

    if !started {
        return Err(AppError::msg("No se pudo iniciar el listener OAuth en el puerto 8888"));
    }

    LISTENER_ACTIVE.store(true, Ordering::SeqCst);
    if let Ok(mut guard) = LISTENER_HANDLES.lock() {
        *guard = handles;
    }
    Ok(())
}

fn spawn_listener_thread(listener: TcpListener) -> JoinHandle<()> {
    thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(180);
        while std::time::Instant::now() < deadline {
            if is_oauth_complete() {
                break;
            }
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = handle_client(stream);
                    if is_oauth_complete() {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(80));
                }
                Err(_) => break,
            }
        }
        // Solo el ultimo hilo activo apaga el flag.
        if let Ok(guard) = LISTENER_HANDLES.lock() {
            if guard.iter().all(|h| h.is_finished()) {
                LISTENER_ACTIVE.store(false, Ordering::SeqCst);
            }
        }
    })
}

fn is_oauth_complete() -> bool {
    PENDING
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|p| p.ready))
        .unwrap_or(false)
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .unwrap_or("");

    let path_only = path.split('?').next().unwrap_or("");
    if path_only != "/callback" {
        let body = "Not Found";
        let response = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
        return Ok(());
    }

    let query = path.split('?').nth(1).unwrap_or("");
    let code = parse_query_param(query, "code");
    let error = parse_query_param(query, "error");
    let error_description = parse_query_param(query, "error_description");

    let poll = if let Some(c) = code.filter(|s| !s.is_empty()) {
        OAuthPoll {
            ready: true,
            code: Some(c),
            error: None,
            error_description: None,
        }
    } else {
        let (error, error_description) = if error.is_some() || error_description.is_some() {
            (error, error_description)
        } else {
            (
                Some("redirect_mismatch".into()),
                Some(
                    "Spotify no envio codigo. Si viste 'redirect_uri: Not matching configuration', \
                     agrega http://127.0.0.1:8888/callback en Dashboard > Settings > Redirect URIs, \
                     pulsa Add y luego SAVE al final de la pagina (obligatorio)."
                        .into(),
                ),
            )
        };
        OAuthPoll {
            ready: true,
            code: None,
            error,
            error_description,
        }
    };

    let html = if poll.code.is_some() {
        r#"<!DOCTYPE html><html><body style="background:#121212;color:#1DB954;font-family:sans-serif;text-align:center;padding:60px"><h2>Spotify autorizado</h2><p>Ya podes cerrar esta ventana y volver al launcher.</p></body></html>"#.to_string()
    } else {
        let err = poll.error.as_deref().unwrap_or("desconocido");
        let hint = spotify_error_hint(err);
        let msg = poll
            .error_description
            .as_deref()
            .unwrap_or(err);
        format!(
            r#"<!DOCTYPE html><html><body style="background:#121212;color:#e74c3c;font-family:sans-serif;text-align:center;padding:32px"><h2>Error de Spotify ({err})</h2><p style="max-width:480px;margin:16px auto;line-height:1.5;color:#fff">{msg}</p><p style="max-width:480px;margin:16px auto;line-height:1.5;color:#aaa;font-size:14px">{hint}</p></body></html>"#
        )
    };

    if let Ok(mut guard) = PENDING.lock() {
        *guard = Some(poll);
    }

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn parse_query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next()?;
        if k == key {
            return parts.next().map(|v| {
                urlencoding::decode(v)
                    .map(|s| s.into_owned())
                    .unwrap_or_else(|_| v.to_string())
            });
        }
    }
    None
}

pub fn spotify_error_hint(code: &str) -> &'static str {
    match code {
        "server_error" => {
            "Spotify rechazo la app. En apps nuevas (2025+) NO uses localhost: usa SOLO \
             http://127.0.0.1:8888/callback en Dashboard y en el launcher. En User Management \
             agrega el email EXACTO de tu cuenta (open.spotify.com > Settings > Profile). \
             Cierra sesion en Spotify, guarda cambios, espera 1 min e intenta de nuevo."
        }
        "access_denied" => "Cancelaste la autorizacion en Spotify.",
        "invalid_client" => "Client ID o Client Secret incorrectos en el launcher.",
        "invalid_grant" => "Codigo expirado o redirect URI distinto al autorizar. Usa el mismo URI en launcher y Dashboard.",
        "redirect_mismatch" | "no_code" => {
            "El Redirect URI no coincide con Spotify Dashboard. Abre Settings de TU app, agrega \
             http://127.0.0.1:8888/callback, pulsa Add y baja hasta SAVE (sin Save no se guarda). \
             Espera 30 segundos e intenta de nuevo."
        }
        _ => "Verifica Redirect URIs (copialos del launcher), User Management y credenciales.",
    }
}

pub fn normalize_redirect_uri(raw: &str) -> AppResult<String> {
    let mut t = raw.trim().to_string();
    if t.is_empty() {
        return Err(AppError::msg("Redirect URI vacio"));
    }
    if t.contains("localhost") {
        t = t.replace("localhost", "127.0.0.1");
    }
    while t.ends_with('/') && t.len() > 1 {
        t.pop();
    }
    if !t.starts_with("http://127.0.0.1") && !t.starts_with("http://[::1]") {
        return Err(AppError::msg(
            "Spotify ya no acepta localhost. Usa http://127.0.0.1:8888/callback",
        ));
    }
    if !t.ends_with("/callback") {
        return Err(AppError::msg("El redirect debe terminar en /callback"));
    }
    Ok(t)
}
