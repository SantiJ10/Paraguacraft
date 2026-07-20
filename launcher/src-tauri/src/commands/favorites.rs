//! Servidores multijugador favoritos.

use tauri::{AppHandle, State};

use crate::config;
use crate::core::accounts;
use crate::core::bedrock;
use crate::core::favorites::{self, FavoriteServer};
use crate::core::game_profiles;
use crate::core::paths;
use crate::core::servers;
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use crate::state::AppState;

#[tauri::command]
pub fn list_favorite_servers() -> Vec<FavoriteServer> {
    favorites::list()
}

#[tauri::command]
pub fn add_favorite_server(
    name: String,
    address: String,
    notes: Option<String>,
    loader_hint: Option<String>,
    from_playit: Option<bool>,
    bedrock_port: Option<u16>,
) -> AppResult<FavoriteServer> {
    favorites::add(
        &name,
        &address,
        notes,
        loader_hint,
        from_playit.unwrap_or(false),
        bedrock_port,
    )
}

#[tauri::command]
pub fn remove_favorite_server(id: String) -> AppResult<()> {
    favorites::remove(&id)
}

/// Perfil sugerido para un favorito (no lanza nada): útil para mostrar en la UI
/// antes de apretar "Unirse".
#[tauri::command]
pub fn suggest_favorite_profile(id: String) -> AppResult<String> {
    let fav = favorites::get(&id).ok_or_else(|| AppError::msg("Servidor favorito no encontrado"))?;
    Ok(game_profiles::infer_profile_id_for_favorite(&fav))
}

/// Join inteligente: infiere el perfil correcto (1.8.9 vs 1.21.11) y lanza
/// directo, sin depender de qué instancia haya quedada seleccionada en la
/// biblioteca.
#[tauri::command]
pub async fn join_favorite_server(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<u32> {
    let fav = favorites::get(&id).ok_or_else(|| AppError::msg("Servidor favorito no encontrado"))?;
    let profile_id = game_profiles::infer_profile_id_for_favorite(&fav);
    crate::commands::playbook::launch_game_profile(
        app,
        state,
        profile_id,
        Some("favorite".into()),
        Some(id),
    )
    .await
}

/// Guarda un servidor local (con túnel Playit activo o dirección manual) como
/// favorito, infiriendo `loaderHint` desde la versión de MC del servidor.
#[tauri::command]
pub fn add_favorite_from_server(server_id: String, bedrock_port: Option<u16>) -> AppResult<FavoriteServer> {
    let prof = servers::profile_by_id(&server_id)?;
    let status = servers::status(&server_id)?;
    let address = status
        .playit_address
        .or(prof.playit_address.clone())
        .ok_or_else(|| AppError::msg("Este servidor todavía no tiene una dirección Playit detectada."))?;
    let hint = if prof.mc_version.starts_with("1.8") {
        Some("189".to_string())
    } else if prof.mc_version.starts_with("1.21") {
        Some("modern".to_string())
    } else {
        None
    };
    favorites::add(
        &prof.name,
        &address,
        Some(format!("Servidor local · {}", prof.mc_version)),
        hint,
        true,
        bedrock_port,
    )
}

/// Direccion Bedrock (host:puerto) de un favorito, si tiene `bedrockPort`.
#[tauri::command]
pub fn favorite_bedrock_address(id: String) -> AppResult<String> {
    let fav = favorites::get(&id).ok_or_else(|| AppError::msg("Servidor favorito no encontrado"))?;
    let port = fav
        .bedrock_port
        .ok_or_else(|| AppError::msg("Este favorito no tiene puerto Bedrock (Geyser) configurado."))?;
    let (host, _) = favorites::parse_address(&fav.address);
    Ok(format!("{host}:{port}"))
}

/// Abre Minecraft: Bedrock Edition para unirse a un favorito con Geyser.
/// Por limitaciones de la app UWP no se puede unir directo por IP: se abre
/// la app y el front muestra/copia `host:puerto` para agregarlo a mano una vez.
#[tauri::command]
pub fn join_favorite_bedrock(app: AppHandle, id: String) -> AppResult<String> {
    let address = favorite_bedrock_address(id)?;

    let account = accounts::active_account()
        .ok_or_else(|| AppError::msg("No hay cuenta activa. Agrega una en Ajustes."))?;
    if !account.premium {
        return Err(AppError::msg(
            "Se necesita cuenta Premium (Microsoft) para jugar Minecraft: Bedrock Edition",
        ));
    }

    let settings = config::read_json::<AppSettings>(&paths::config_file()).unwrap_or_default();
    let username = account.username.clone();

    bedrock::launch(&username)?;
    bedrock::watch_session(app, username, settings.close_on_launch);

    Ok(address)
}
