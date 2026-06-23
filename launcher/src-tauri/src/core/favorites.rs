//! Servidores multijugador favoritos (acceso rápido desde el inicio).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config;
use crate::core::paths;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteServer {
    pub id: String,
    pub name: String,
    /// Host o `host:puerto` (puerto 25565 por defecto).
    pub address: String,
    #[serde(default)]
    pub notes: Option<String>,
    pub created_at: u64,
}

#[derive(Default, Serialize, Deserialize)]
struct FavoritesFile {
    #[serde(default)]
    servers: Vec<FavoriteServer>,
}

fn favorites_path() -> PathBuf {
    paths::data_dir().join("favorite_servers.json")
}

fn load() -> Vec<FavoriteServer> {
    config::read_json::<FavoritesFile>(&favorites_path())
        .unwrap_or_default()
        .servers
}

fn save(list: &[FavoriteServer]) -> AppResult<()> {
    config::write_json_atomic(&favorites_path(), &FavoritesFile {
        servers: list.to_vec(),
    })
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn list() -> Vec<FavoriteServer> {
    load()
}

pub fn add(name: &str, address: &str, notes: Option<String>) -> AppResult<FavoriteServer> {
    let name = name.trim();
    let address = address.trim();
    if name.is_empty() || address.is_empty() {
        return Err(AppError::msg("Nombre y dirección son obligatorios"));
    }
    let mut list = load();
    let fav = FavoriteServer {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        address: address.to_string(),
        notes: notes.filter(|n| !n.trim().is_empty()),
        created_at: now(),
    };
    list.push(fav.clone());
    save(&list)?;
    Ok(fav)
}

pub fn remove(id: &str) -> AppResult<()> {
    let mut list = load();
    let before = list.len();
    list.retain(|s| s.id != id);
    if list.len() == before {
        return Err(AppError::msg("Servidor favorito no encontrado"));
    }
    save(&list)
}

/// Separa `host:port` en (host, port opcional).
pub fn parse_address(address: &str) -> (String, Option<u16>) {
    let s = address.trim();
    if s.starts_with('[') {
        if let Some(end) = s.find(']') {
            let host = s[1..end].to_string();
            let rest = s[end + 1..].trim_start_matches(':');
            let port = rest.parse().ok();
            return (host, port);
        }
    }
    if let Some((host, port)) = s.rsplit_once(':') {
        if let Ok(p) = port.parse::<u16>() {
            return (host.to_string(), Some(p));
        }
    }
    (s.to_string(), None)
}
