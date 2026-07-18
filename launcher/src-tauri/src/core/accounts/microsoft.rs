//! Autenticacion Microsoft (Premium).
//!
//! Implementa los dos flujos del Python:
//!   - device-code / QR  (MS_DEVICE_CLIENT_ID + microsoft.com/link)
//!   - browser auth-code (CLIENT_ID + login.live.com)
//! y la cadena Xbox Live -> XSTS -> Minecraft -> profile (`_ms_login_desde_tokens`).
//!
//! Eficiencia: el refresh es ON-DEMAND (no hay hilo timer como en el Python).
//! El poll del device-code lo dispara la UI; aqui no hay loops de fondo.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::error::{AppError, AppResult};
use crate::models::DeviceCodeStart;

// Constantes reales del launcher Python (paragua.py).
pub const CLIENT_ID: &str = "72fb7c48-c2f5-4d13-b0e7-9835b3b906c0";
pub const REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
pub const DEVICE_CLIENT_ID: &str = "d772766b-19b4-4f69-b353-989f890c5d3b";
pub const SCOPE: &str = "XboxLive.signin offline_access";
pub const DEVICE_VERIFY_URI: &str = "https://www.microsoft.com/link";

const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const DEVICECODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const XBL_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

/// Estado transitorio del device-code flow (guardado en `AppState`).
#[derive(Debug, Clone)]
pub struct DeviceAuth {
    pub device_code: String,
    pub expires_at: u64,
    pub interval: u64,
    pub client_id: String,
}

/// Sesion Minecraft resuelta tras la cadena de auth.
#[derive(Debug, Clone)]
pub struct MsLogin {
    pub id: String,
    pub name: String,
    pub mc_access_token: String,
    pub ms_refresh_token: String,
    pub ms_client_id: String,
}

/// Resultado de un poll del device-code.
pub enum PollOutcome {
    Pending,
    Done(MsLogin),
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// URL de login por navegador (auth-code). Fuerza selector de cuenta.
pub fn build_login_url() -> String {
    let scope = urlencoding(SCOPE);
    let redirect = urlencoding(REDIRECT_URI);
    format!(
        "https://login.live.com/oauth20_authorize.srf?client_id={CLIENT_ID}\
         &response_type=code&redirect_uri={redirect}&scope={scope}&prompt=select_account"
    )
}

/// Inicia el device-code flow. Devuelve datos para la UI + estado a guardar.
pub async fn device_start(http: &reqwest::Client) -> AppResult<(DeviceCodeStart, DeviceAuth)> {
    let resp = http
        .post(DEVICECODE_URL)
        .form(&[("client_id", DEVICE_CLIENT_ID), ("scope", SCOPE)])
        .send()
        .await?
        .error_for_status()?;
    let data: Value = resp.json().await?;

    let device_code = data["device_code"].as_str().unwrap_or_default().to_string();
    let user_code = data["user_code"].as_str().unwrap_or_default().to_string();
    let expires_in = data["expires_in"].as_u64().unwrap_or(900);
    let interval = data["interval"].as_u64().unwrap_or(5).max(3);

    let auth = DeviceAuth {
        device_code,
        expires_at: now() + expires_in,
        interval,
        client_id: DEVICE_CLIENT_ID.to_string(),
    };
    let start = DeviceCodeStart {
        user_code,
        verification_uri: DEVICE_VERIFY_URI.to_string(),
        expires_in,
        interval,
    };
    Ok((start, auth))
}

/// Un intento de poll del device-code.
pub async fn device_poll(http: &reqwest::Client, auth: &DeviceAuth) -> AppResult<PollOutcome> {
    if now() > auth.expires_at {
        return Err(AppError::msg("El codigo expiro. Genera uno nuevo."));
    }
    let resp = http
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", auth.client_id.as_str()),
            ("device_code", auth.device_code.as_str()),
        ])
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::BAD_REQUEST {
        let err: Value = resp.json().await.unwrap_or(Value::Null);
        let code = err["error"].as_str().unwrap_or("");
        return match code {
            "authorization_pending" | "slow_down" => Ok(PollOutcome::Pending),
            "expired_token" => Err(AppError::msg("Codigo expirado. Genera uno nuevo.")),
            other => Err(AppError::msg(
                err["error_description"].as_str().unwrap_or(other).to_string(),
            )),
        };
    }
    let tok: Value = resp.error_for_status()?.json().await?;
    let access = tok["access_token"].as_str().unwrap_or_default().to_string();
    let refresh = tok["refresh_token"].as_str().unwrap_or_default().to_string();
    let login = login_from_tokens(http, &access, &refresh, &auth.client_id).await?;
    Ok(PollOutcome::Done(login))
}

/// Completa el flujo browser auth-code: intercambia `code` por tokens.
pub async fn complete_auth_code(http: &reqwest::Client, code: &str) -> AppResult<MsLogin> {
    let tok: Value = http
        .post(TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
            ("scope", SCOPE),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let access = tok["access_token"].as_str().unwrap_or_default().to_string();
    let refresh = tok["refresh_token"].as_str().unwrap_or_default().to_string();
    login_from_tokens(http, &access, &refresh, CLIENT_ID).await
}

/// Refresca la sesion completa a partir del refresh_token (on-demand).
pub async fn refresh(
    http: &reqwest::Client,
    client_id: &str,
    refresh_token: &str,
) -> AppResult<MsLogin> {
    let tok: Value = http
        .post(TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", SCOPE),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let access = tok["access_token"].as_str().unwrap_or_default().to_string();
    // El refresh_token rota; si no viene, se reutiliza el anterior.
    let new_refresh = tok["refresh_token"].as_str().unwrap_or(refresh_token).to_string();
    login_from_tokens(http, &access, &new_refresh, client_id).await
}

/// Valida el access_token contra Mojang. Ok(Some((name,uuid))) si es valido,
/// Ok(None) si expiro (401/403), Err en fallos de red.
pub async fn validate_profile(
    http: &reqwest::Client,
    mc_access: &str,
) -> AppResult<Option<(String, String)>> {
    let resp = http
        .get(PROFILE_URL)
        .bearer_auth(mc_access)
        .send()
        .await?;
    match resp.status().as_u16() {
        200 => {
            let p: Value = resp.json().await?;
            Ok(Some((
                p["name"].as_str().unwrap_or_default().to_string(),
                p["id"].as_str().unwrap_or_default().to_string(),
            )))
        }
        401 | 403 => Ok(None),
        s if matches!(s, 429 | 502 | 503 | 504) => Err(AppError::transient_http(s)),
        s => Err(AppError::msg(format!("Mojang respondio HTTP {s}"))),
    }
}

/// Cadena Xbox Live -> XSTS -> Minecraft -> profile (`_ms_login_desde_tokens`).
pub async fn login_from_tokens(
    http: &reqwest::Client,
    ms_access: &str,
    ms_refresh: &str,
    client_id: &str,
) -> AppResult<MsLogin> {
    // 1) Xbox Live.
    let xbl: Value = http
        .post(XBL_URL)
        .json(&json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={ms_access}"),
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let uhs = xbl["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or_else(|| AppError::msg("Respuesta Xbox sin uhs"))?
        .to_string();
    let xbl_token = xbl["Token"].as_str().unwrap_or_default().to_string();

    // 2) XSTS.
    let xsts: Value = http
        .post(XSTS_URL)
        .json(&json!({
            "Properties": { "SandboxId": "RETAIL", "UserTokens": [xbl_token] },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let xsts_token = xsts["Token"].as_str().unwrap_or_default().to_string();

    // 3) Minecraft login.
    let mc: Value = http
        .post(MC_LOGIN_URL)
        .json(&json!({ "identityToken": format!("XBL3.0 x={uhs};{xsts_token}") }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let mc_access = mc["access_token"].as_str().unwrap_or_default().to_string();

    // 4) Profile.
    let resp = http.get(PROFILE_URL).bearer_auth(&mc_access).send().await?;
    if resp.status().as_u16() == 404 {
        return Err(AppError::msg(
            "Esta cuenta Microsoft no tiene Minecraft Java comprado.",
        ));
    }
    let profile: Value = resp.error_for_status()?.json().await?;

    Ok(MsLogin {
        id: profile["id"].as_str().unwrap_or_default().to_string(),
        name: profile["name"].as_str().unwrap_or_default().to_string(),
        mc_access_token: mc_access,
        ms_refresh_token: ms_refresh.to_string(),
        ms_client_id: client_id.to_string(),
    })
}

/// Codificacion percent minima para querystrings.
fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
