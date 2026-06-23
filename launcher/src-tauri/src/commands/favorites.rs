//! Servidores multijugador favoritos.

use crate::core::favorites::{self, FavoriteServer};
use crate::error::AppResult;

#[tauri::command]
pub fn list_favorite_servers() -> Vec<FavoriteServer> {
    favorites::list()
}

#[tauri::command]
pub fn add_favorite_server(
    name: String,
    address: String,
    notes: Option<String>,
) -> AppResult<FavoriteServer> {
    favorites::add(&name, &address, notes)
}

#[tauri::command]
pub fn remove_favorite_server(id: String) -> AppResult<()> {
    favorites::remove(&id)
}
