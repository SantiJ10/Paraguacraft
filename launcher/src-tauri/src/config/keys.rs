//! Resolucion segura de claves API (nunca hardcodeadas en el repo).
//!
//! Cadena de prioridad para CurseForge:
//!   1. Variable de entorno `CURSEFORGE_API_KEY` (cargada desde `.env` al inicio)
//!   2. `app_secrets.json` en el directorio de datos del launcher
//!   3. Campo manual en `launcher_config.json` (Ajustes)

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config;
use crate::core::paths;
use crate::models::AppSettings;

const ENV_CF_KEY: &str = "CURSEFORGE_API_KEY";
const ENV_OPENAI_KEY: &str = "OPENAI_API_KEY";
const ENV_GROQ_KEY: &str = "GROQ_API_KEY";

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSecretsFile {
    #[serde(default)]
    curseforge_api_key: Option<String>,
    #[serde(default)]
    spotify_client_id: Option<String>,
    #[serde(default)]
    spotify_client_secret: Option<String>,
    #[serde(default)]
    spotify_refresh_token: Option<String>,
    #[serde(default)]
    spotify_redirect_uri: Option<String>,
    #[serde(default)]
    openai_api_key: Option<String>,
    #[serde(default)]
    groq_api_key: Option<String>,
}

fn app_secrets_file() -> PathBuf {
    paths::data_dir().join("app_secrets.json")
}

/// Carga `.env` desde rutas candidatas (dev + produccion). Idempotente.
pub fn load_env_files() {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Dev Tauri: launcher/.env junto al crate (src-tauri/../.env)
    candidates.push(PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env")));

    if let Ok(mut dir) = std::env::current_dir() {
        for _ in 0..8 {
            candidates.push(dir.join(".env"));
            candidates.push(dir.join("launcher").join(".env"));
            if dir.file_name().and_then(|n| n.to_str()) == Some("src-tauri") {
                if let Some(launcher) = dir.parent() {
                    candidates.push(launcher.join(".env"));
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }

    candidates.push(paths::data_dir().join(".env"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(".env"));
        }
    }

    let mut seen = std::collections::HashSet::new();
    for p in candidates {
        if seen.insert(p.clone()) && p.is_file() {
            let _ = dotenvy::from_path(&p);
        }
    }
}

/// Indica si Paraguabot tiene proveedor LLM configurado.
pub fn llm_is_configured() -> bool {
    resolve_llm_config().is_some()
}

fn read_app_secrets() -> AppSecretsFile {
    config::read_json(&app_secrets_file()).unwrap_or_default()
}

/// Key embebida en compile-time (release CI con `CURSEFORGE_API_KEY`); fallback para usuarios finales.
fn baked_curseforge_key() -> Option<&'static str> {
    option_env!("CURSEFORGE_API_KEY").filter(|s| !s.is_empty())
}

/// Key embebida en compile-time (release CI con `GROQ_API_KEY`).
fn baked_groq_key() -> Option<&'static str> {
    option_env!("GROQ_API_KEY").filter(|s| !s.is_empty())
}

/// True si las keys vienen del launcher (env / compile-time) y no hace falta pedirlas al usuario.
pub fn api_keys_managed_by_launcher() -> bool {
    curseforge_api_key_from_env_or_baked().is_some() && groq_api_key_from_env_or_baked().is_some()
}

fn curseforge_api_key_from_env_or_baked() -> Option<String> {
    if let Ok(v) = std::env::var(ENV_CF_KEY) {
        let t = v.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    baked_curseforge_key().map(|s| s.to_string())
}

fn groq_api_key_from_env_or_baked() -> Option<String> {
    if let Ok(v) = std::env::var(ENV_GROQ_KEY) {
        let t = v.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    baked_groq_key().map(|s| s.to_string())
}

/// Resuelve la API key de CurseForge sin exponerla en el codigo fuente.
pub fn curseforge_api_key() -> String {
    if let Some(k) = curseforge_api_key_from_env_or_baked() {
        return k;
    }
    if let Some(k) = read_app_secrets().curseforge_api_key {
        let t = k.trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    let from_settings = config::read_json::<AppSettings>(&paths::config_file())
        .unwrap_or_default()
        .curseforge_api_key
        .unwrap_or_default();
    let t = from_settings.trim();
    if !t.is_empty() {
        return t.to_string();
    }
    String::new()
}

/// Persiste la key en `app_secrets.json` (permisos restringidos en Unix).
pub fn save_curseforge_api_key(key: &str) -> crate::error::AppResult<()> {
    let t = key.trim();
    let mut file = read_app_secrets();
    file.curseforge_api_key = if t.is_empty() { None } else { Some(t.to_string()) };
    config::write_secret_json(&app_secrets_file(), &file)
}

pub fn read_spotify_client_id() -> Option<String> {
    read_app_secrets()
        .spotify_client_id
        .filter(|s| !s.trim().is_empty())
}

pub fn default_spotify_redirect_uri() -> &'static str {
    "http://127.0.0.1:8888/callback"
}

pub fn read_spotify_redirect_uri() -> String {
    read_app_secrets()
        .spotify_redirect_uri
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.replace("localhost", "127.0.0.1"))
        .unwrap_or_else(|| default_spotify_redirect_uri().to_string())
}

pub fn save_spotify_credentials(
    client_id: &str,
    client_secret: &str,
    redirect_uri: Option<&str>,
) -> crate::error::AppResult<()> {
    let mut file = read_app_secrets();
    let cid = client_id.trim();
    let sec = client_secret.trim();
    let prev_id = file.spotify_client_id.clone();
    file.spotify_client_id = if cid.is_empty() { None } else { Some(cid.to_string()) };
    file.spotify_client_secret = if sec.is_empty() { None } else { Some(sec.to_string()) };
    if prev_id.as_deref() != Some(cid) && !cid.is_empty() {
        file.spotify_refresh_token = None;
    }
    if let Some(uri) = redirect_uri.map(str::trim).filter(|s| !s.is_empty()) {
        let normalized = uri.replace("localhost", "127.0.0.1");
        file.spotify_redirect_uri = Some(normalized);
    } else if file
        .spotify_redirect_uri
        .as_ref()
        .is_some_and(|u| u.contains("localhost"))
    {
        file.spotify_redirect_uri = Some(default_spotify_redirect_uri().to_string());
    }
    config::write_secret_json(&app_secrets_file(), &file)
}

fn read_spotify_client_secret() -> Option<String> {
    read_app_secrets()
        .spotify_client_secret
        .filter(|s| !s.trim().is_empty())
}

pub fn read_spotify_refresh_token() -> Option<String> {
    read_app_secrets()
        .spotify_refresh_token
        .filter(|s| !s.trim().is_empty())
}

pub fn save_spotify_refresh_token(token: Option<&str>) -> crate::error::AppResult<()> {
    let mut file = read_app_secrets();
    file.spotify_refresh_token = token
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    config::write_secret_json(&app_secrets_file(), &file)
}

pub fn spotify_credentials() -> (Option<String>, Option<String>) {
    (read_spotify_client_id(), read_spotify_client_secret())
}

/// Configuración del proveedor LLM (Groq primero, OpenAI como fallback).
pub struct LlmConfig {
    pub provider: &'static str,
    pub api_key: String,
    pub chat_url: &'static str,
    pub model: &'static str,
}

pub fn groq_api_key() -> Option<String> {
    if let Some(k) = groq_api_key_from_env_or_baked() {
        return Some(k);
    }
    read_app_secrets()
        .groq_api_key
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

pub fn save_groq_api_key(key: &str) -> crate::error::AppResult<()> {
    let t = key.trim();
    let mut file = read_app_secrets();
    file.groq_api_key = if t.is_empty() { None } else { Some(t.to_string()) };
    config::write_secret_json(&app_secrets_file(), &file)
}

/// Groq tiene prioridad sobre OpenAI para Paraguabot.
pub fn resolve_llm_config() -> Option<LlmConfig> {
    if let Some(key) = groq_api_key() {
        return Some(LlmConfig {
            provider: "groq",
            api_key: key,
            chat_url: "https://api.groq.com/openai/v1/chat/completions",
            model: "openai/gpt-oss-120b",
        });
    }
    openai_api_key().map(|key| LlmConfig {
        provider: "openai",
        api_key: key,
        chat_url: "https://api.openai.com/v1/chat/completions",
        model: "gpt-4o-mini",
    })
}

/// API key OpenAI (env → app_secrets.json).
pub fn openai_api_key() -> Option<String> {
    if let Ok(v) = std::env::var(ENV_OPENAI_KEY) {
        let t = v.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    read_app_secrets()
        .openai_api_key
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

pub fn save_openai_api_key(key: &str) -> crate::error::AppResult<()> {
    let t = key.trim();
    let mut file = read_app_secrets();
    file.openai_api_key = if t.is_empty() { None } else { Some(t.to_string()) };
    config::write_secret_json(&app_secrets_file(), &file)
}
