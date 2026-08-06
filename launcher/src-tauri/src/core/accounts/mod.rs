//! Gestor de cuentas: Microsoft Premium (tokens persistentes) + offline.

pub mod microsoft;
pub mod offline;
pub mod store;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AppError, AppResult};
use crate::models::{Account, TokenRecord};

use microsoft::MsLogin;

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn avatar_url(uuid: &str) -> String {
    let clean = uuid.replace('-', "");
    format!("https://crafatar.com/avatars/{clean}?overlay")
}

pub fn list() -> Vec<Account> {
    store::load_accounts()
}

pub fn active_account() -> Option<Account> {
    store::load_accounts().into_iter().find(|a| a.active)
}

pub fn set_active(id: &str) -> AppResult<Vec<Account>> {
    let mut accounts = store::load_accounts();
    let mut found = false;
    for a in accounts.iter_mut() {
        a.active = a.id == id;
        found |= a.active;
    }
    if !found {
        return Err(AppError::msg("Cuenta no encontrada"));
    }
    store::save_accounts(&accounts)?;
    Ok(accounts)
}

pub fn add_offline(username: &str) -> AppResult<Vec<Account>> {
    let username = validate_offline_username(username)?;
    let uuid = offline::offline_uuid(&username);
    let mut accounts = store::load_accounts();
    // Evitar duplicados por uuid offline.
    if let Some(existing) = accounts.iter().find(|a| a.uuid == uuid).cloned() {
        return set_active(&existing.id);
    }
    for a in accounts.iter_mut() {
        a.active = false;
    }
    let acc = Account {
        id: format!("offline-{uuid}"),
        kind: "offline".into(),
        username: username.clone(),
        uuid: uuid.clone(),
        avatar_url: avatar_url(&uuid),
        active: true,
        premium: false,
    };
    accounts.push(acc);
    store::save_accounts(&accounts)?;
    Ok(accounts)
}

/// Cambia el nick de una cuenta offline (útil para alinear con Ely.by).
/// Recalcula el UUID vanilla (`OfflinePlayer:name`) y el id `offline-{uuid}`.
pub fn rename_offline(id: &str, new_username: &str) -> AppResult<Vec<Account>> {
    let new_username = validate_offline_username(new_username)?;
    let mut accounts = store::load_accounts();
    let idx = accounts
        .iter()
        .position(|a| a.id == id)
        .ok_or_else(|| AppError::msg("Cuenta no encontrada"))?;

    if accounts[idx].kind != "offline" || accounts[idx].premium {
        return Err(AppError::msg("Solo se puede renombrar cuentas offline / no-premium"));
    }

    let old_username = accounts[idx].username.clone();
    if old_username == new_username {
        return Ok(accounts);
    }

    let new_uuid = offline::offline_uuid(&new_username);
    if accounts
        .iter()
        .any(|a| a.id != id && (a.uuid == new_uuid || a.username.eq_ignore_ascii_case(&new_username)))
    {
        return Err(AppError::msg("Ya existe una cuenta con ese nombre"));
    }

    let was_active = accounts[idx].active;
    accounts[idx].id = format!("offline-{new_uuid}");
    accounts[idx].username = new_username.clone();
    accounts[idx].uuid = new_uuid.clone();
    accounts[idx].avatar_url = avatar_url(&new_uuid);
    accounts[idx].active = was_active;
    accounts[idx].kind = "offline".into();
    accounts[idx].premium = false;

    store::save_accounts(&accounts)?;

    // Best-effort: renombrar cache SkinsRestorer y re-aplicar skin global al nick nuevo.
    let _ = crate::core::skins::offline::on_offline_username_changed(&old_username, &new_username);

    Ok(accounts)
}

fn validate_offline_username(username: &str) -> AppResult<String> {
    let username = username.trim();
    if username.len() < 3 {
        return Err(AppError::msg("El nombre debe tener al menos 3 caracteres"));
    }
    if username.len() > 16 {
        return Err(AppError::msg("El nombre no puede superar 16 caracteres"));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::msg(
            "Solo letras, números y guion bajo (como en Minecraft)",
        ));
    }
    Ok(username.to_string())
}

pub fn remove(id: &str) -> AppResult<Vec<Account>> {
    let mut accounts = store::load_accounts();
    let was_active = accounts.iter().find(|a| a.id == id).map(|a| a.active).unwrap_or(false);
    accounts.retain(|a| a.id != id);
    let _ = store::remove_token(id);
    // Si borramos la activa, activamos la primera que quede.
    if was_active {
        if let Some(first) = accounts.first_mut() {
            first.active = true;
        }
    }
    store::save_accounts(&accounts)?;
    Ok(accounts)
}

/// Crea o actualiza una cuenta Microsoft a partir de una sesion resuelta y la
/// marca como activa, persistiendo el token de forma segura.
pub fn upsert_microsoft(login: &MsLogin) -> AppResult<Vec<Account>> {
    let id = format!("ms-{}", login.id);
    let mut accounts = store::load_accounts();
    for a in accounts.iter_mut() {
        a.active = false;
    }
    let account = Account {
        id: id.clone(),
        kind: "microsoft".into(),
        username: login.name.clone(),
        uuid: login.id.clone(),
        avatar_url: avatar_url(&login.id),
        active: true,
        premium: true,
    };
    if let Some(slot) = accounts.iter_mut().find(|a| a.id == id) {
        *slot = account;
    } else {
        accounts.push(account);
    }
    store::set_token(
        &id,
        TokenRecord {
            ms_refresh_token: login.ms_refresh_token.clone(),
            mc_access_token: login.mc_access_token.clone(),
            ms_client_id: login.ms_client_id.clone(),
            last_refresh: now(),
        },
    )?;
    store::save_accounts(&accounts)?;
    Ok(accounts)
}

/// Garantiza un access_token valido para la cuenta `id` (refresh perezoso).
/// Solo refresca si el token fue rechazado o si pasaron >55 min. Se usa antes
/// de lanzar el juego (Fase 3). Reemplaza el hilo de auto-refresh del Python.
pub async fn ensure_valid_token(
    http: &reqwest::Client,
    id: &str,
) -> AppResult<TokenRecord> {
    let rec = store::get_token(id).ok_or_else(|| AppError::msg("Sin token para esta cuenta"))?;

    // Si el token es reciente, validamos contra Mojang; si responde 200, listo.
    let fresh = now().saturating_sub(rec.last_refresh) < 55 * 60;
    if fresh {
        match microsoft::validate_profile(http, &rec.mc_access_token).await {
            Ok(Some(_)) => return Ok(rec),
            Ok(None) => {}
            Err(e) if e.is_transient_server() => return Ok(rec),
            Err(_) => {}
        }
    }

    match microsoft::refresh(http, &rec.ms_client_id, &rec.ms_refresh_token).await {
        Ok(login) => {
            upsert_microsoft(&login)?;
            store::get_token(id).ok_or_else(|| AppError::msg("No se pudo guardar el token refrescado"))
        }
        Err(e) if e.is_transient_server() && !rec.mc_access_token.is_empty() => Ok(rec),
        Err(e) => Err(e),
    }
}
