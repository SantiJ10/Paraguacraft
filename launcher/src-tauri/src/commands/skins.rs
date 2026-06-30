//! Comandos de skins (Premium + No-Premium + catálogo estilo NameMC).

use std::path::PathBuf;

use tauri::AppHandle;
use tauri::State;

use crate::core::accounts;
use crate::core::skins::{self, catalog, history, mojang, offline, pending_premium, SkinProfile};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub async fn get_active_skin(state: State<'_, AppState>) -> AppResult<SkinProfile> {
    let http = state.client();
    let profile = skins::active_or_steve();
    let enriched = skins::enrich_profile(&http, profile).await;
    state.shutdown_network();
    Ok(enriched)
}

#[tauri::command]
pub async fn get_skin_for_account(state: State<'_, AppState>, id: String) -> AppResult<SkinProfile> {
    let http = state.client();
    let acc = accounts::list()
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| AppError::msg("Cuenta no encontrada"))?;
    let profile = skins::profile_for(&acc.username, &acc.uuid, acc.premium);
    let enriched = skins::enrich_profile(&http, profile).await;
    state.shutdown_network();
    Ok(enriched)
}

#[tauri::command]
pub fn get_offline_skin(username: String) -> SkinProfile {
    skins::offline_profile(&username)
}

#[tauri::command]
pub fn has_offline_skin_file() -> bool {
    offline::has_global_skin()
}

#[tauri::command]
pub fn get_offline_skin_path() -> Option<String> {
    if offline::has_global_skin() {
        Some(offline::global_skin_path().to_string_lossy().to_string())
    } else {
        None
    }
}

#[tauri::command]
pub async fn pick_and_apply_offline_skin(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<offline::ApplySkinResult> {
    use tauri_plugin_dialog::DialogExt;

    let file = app
        .dialog()
        .file()
        .add_filter("Skin PNG", &["png"])
        .blocking_pick_file();

    let Some(file) = file else {
        return Err(AppError::msg("No se seleccionó ningún archivo"));
    };

    let path = file
        .into_path()
        .map_err(|e| AppError::msg(format!("Ruta inválida: {e}")))?;

    apply_skin_file_path(state, path.to_string_lossy().to_string(), "classic".into()).await
}

#[tauri::command]
pub async fn apply_offline_skin_path(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<offline::ApplySkinResult> {
    apply_skin_file_path(state, path, "classic".into()).await
}

async fn apply_skin_file_path(
    state: State<'_, AppState>,
    path: String,
    variant: String,
) -> AppResult<offline::ApplySkinResult> {
    let path = PathBuf::from(path.trim());
    if !path.is_file() {
        return Err(AppError::msg("Archivo de skin no encontrado"));
    }
    let http = state.client();
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("skin")
        .to_string();
    let result = skins::apply_skin_file(&http, &path, &variant, &name, "").await?;
    state.shutdown_network();
    Ok(result)
}

#[tauri::command]
pub async fn lookup_skin_player(
    state: State<'_, AppState>,
    username: String,
) -> AppResult<mojang::SkinLookup> {
    let http = state.client();
    let result = mojang::lookup_player(&http, Some(&username), None).await;
    state.shutdown_network();
    result
}

#[tauri::command]
pub async fn skin_catalog_search(
    state: State<'_, AppState>,
    query: String,
    page: u32,
    random: bool,
) -> AppResult<catalog::SkinCatalogPage> {
    let http = state.client();
    let result = catalog::search(&http, &query, page, random).await;
    state.shutdown_network();
    result
}

#[tauri::command]
pub async fn skin_preview_image(
    state: State<'_, AppState>,
    id: String,
    kind: String,
    size: u32,
) -> AppResult<Option<String>> {
    let http = state.client();
    let url = if kind == "helm" {
        catalog::helm_preview_url(&id, size.max(16).min(512))
    } else {
        catalog::body_preview_url(&id, size.max(32).min(512))
    };
    let data = mojang::fetch_image_data_url(&http, &url).await;
    state.shutdown_network();
    Ok(data)
}

#[tauri::command]
pub async fn apply_skin_from_username(
    state: State<'_, AppState>,
    username: String,
    variant: String,
) -> AppResult<offline::ApplySkinResult> {
    let http = state.client();
    let result = skins::apply_from_username(&http, &username, &variant).await?;
    state.shutdown_network();
    Ok(result)
}

#[tauri::command]
pub async fn apply_skin_from_url(
    state: State<'_, AppState>,
    url: String,
    variant: String,
    name: String,
) -> AppResult<offline::ApplySkinResult> {
    let http = state.client();
    let result = skins::apply_from_url(&http, &url, &variant, &name).await?;
    state.shutdown_network();
    Ok(result)
}

#[tauri::command]
pub async fn download_skin_file(
    state: State<'_, AppState>,
    url: String,
    name: String,
) -> AppResult<String> {
    let http = state.client();
    let path = mojang::save_skin_to_library(&http, &url, &name).await?;
    history::push(&name, &url, "classic");
    state.shutdown_network();
    Ok(path)
}

#[tauri::command]
pub fn get_skin_history() -> Vec<history::SkinHistoryEntry> {
    history::list()
}

#[tauri::command]
pub fn clear_skin_history() -> AppResult<()> {
    history::clear()
}

#[tauri::command]
pub fn can_upload_premium_skin() -> bool {
    mojang::active_can_upload_premium()
}

#[tauri::command]
pub fn has_pending_premium_skin() -> bool {
    pending_premium::has_pending()
}

#[tauri::command]
pub async fn sync_pending_premium_skin(state: State<'_, AppState>) -> AppResult<Option<String>> {
    let http = state.client();
    let result = pending_premium::flush_pending(&http).await;
    state.shutdown_network();
    result
}

#[tauri::command]
pub async fn pick_skin_file_for_preview(app: AppHandle) -> AppResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;

    let file = app
        .dialog()
        .file()
        .add_filter("Skin PNG", &["png"])
        .blocking_pick_file();

    let Some(file) = file else {
        return Ok(None);
    };

    let path = file
        .into_path()
        .map_err(|e| AppError::msg(format!("Ruta inválida: {e}")))?;

    Ok(Some(path.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn apply_skin_file_with_variant(
    state: State<'_, AppState>,
    path: String,
    variant: String,
) -> AppResult<offline::ApplySkinResult> {
    apply_skin_file_path(state, path, variant).await
}
