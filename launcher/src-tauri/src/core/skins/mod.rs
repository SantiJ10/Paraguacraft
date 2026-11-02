//! Skins: avatares Premium/No-Premium, catálogo, historial e inyección offline.

pub mod catalog;
pub mod csl;
pub mod history;
pub mod mojang;
pub mod offline;
pub mod pending_premium;

use std::path::{Path, PathBuf};

use crate::core::accounts;
use crate::error::{AppError, AppResult};

pub use offline::ApplySkinResult;

/// UUID de Steve — fallback cuando no hay cuenta activa.
pub const STEVE_UUID: &str = "8667ba71-b85a-4004-af54-4576b382cd64";

pub fn avatar_url(uuid: &str) -> String {
    let clean = uuid.replace('-', "");
    format!("https://minotar.net/helm/{clean}/128.png")
}

pub fn body_url(uuid: &str) -> String {
    let clean = uuid.replace('-', "");
    format!("https://minotar.net/armor/body/{clean}/160.png")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinProfile {
    pub username: String,
    pub uuid: String,
    pub premium: bool,
    pub avatar_url: String,
    pub body_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_avatar_path: Option<String>,
    /// Imagen embebida (proxy Rust) para WebView cuando URLs externas fallan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_data_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

pub fn steve_profile() -> SkinProfile {
    SkinProfile {
        username: "Steve".into(),
        uuid: STEVE_UUID.into(),
        premium: false,
        avatar_url: avatar_url(STEVE_UUID),
        body_url: body_url(STEVE_UUID),
        local_avatar_path: None,
        avatar_data_url: None,
        skin_url: None,
        model: Some("classic".into()),
    }
}

pub fn profile_for(username: &str, uuid: &str, premium: bool) -> SkinProfile {
    let has_local = !premium && offline::has_skin_for_user(username);
    let local = if has_local {
        let face = offline::global_face_path();
        if !face.is_file() {
            offline::refresh_face_cache();
        }
        if face.is_file() {
            Some(face.to_string_lossy().to_string())
        } else {
            None
        }
    } else {
        None
    };
    let model = if has_local {
        offline::variant_for_user(username)
    } else {
        "classic".into()
    };
    SkinProfile {
        username: username.to_string(),
        uuid: uuid.to_string(),
        premium,
        avatar_url: avatar_url(uuid),
        body_url: body_url(uuid),
        local_avatar_path: local,
        avatar_data_url: None,
        skin_url: None,
        model: Some(model),
    }
}

pub fn active_or_steve() -> SkinProfile {
    accounts::active_account()
        .map(|acc| profile_for(&acc.username, &acc.uuid, acc.premium))
        .unwrap_or_else(steve_profile)
}

pub fn offline_profile(username: &str) -> SkinProfile {
    let uuid = crate::core::accounts::offline::offline_uuid(username);
    profile_for(username, &uuid, false)
}

pub async fn enrich_profile(http: &reqwest::Client, mut profile: SkinProfile) -> SkinProfile {
    if !profile.premium {
        if let Some(path) = offline::effective_skin_path_for(&profile.username) {
            offline::refresh_face_cache();
            let face = offline::global_face_path();
            if face.is_file() {
                profile.local_avatar_path = Some(face.to_string_lossy().to_string());
            } else {
                profile.local_avatar_path = None;
            }
            profile.model = Some(offline::variant_for_user(&profile.username));
            if let Ok(bytes) = std::fs::read(&path) {
                let ts = path
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                profile.avatar_url = format!("{}?v={ts}", avatar_url(&profile.uuid));
                profile.avatar_data_url = mojang::helm_data_url_from_skin_png(&bytes);
                use base64::Engine;
                profile.skin_url = Some(format!(
                    "data:image/png;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(&bytes)
                ));
            }
            // No sobrescribir la skin personalizada del usuario offline con una cuenta ajena de Mojang.
            return profile;
        }
    }

    let cache_bust = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if let Ok(info) = mojang::lookup_player(http, Some(&profile.username), Some(&profile.uuid)).await {
        if info.ok {
            if !info.username.is_empty() {
                if profile.username != info.username {
                    if let Some(acc) = accounts::active_account() {
                        if acc.premium && acc.uuid == profile.uuid {
                            let _ = accounts::sync_premium_username(&acc.id, &info.username);
                        }
                    }
                }
                if profile.premium {
                    profile.username = info.username;
                }
            }
            profile.skin_url = info.skin_url.clone();
            profile.model = Some(info.model);

            if let Some(skin_url) = info.skin_url {
                let bust = texture_cache_key(&skin_url);
                profile.avatar_url = format!("{}?t={bust}", avatar_url(&profile.uuid));
                profile.body_url = format!("{}?t={bust}", body_url(&profile.uuid));
                if let Some(ref mut su) = profile.skin_url {
                    if !su.starts_with("data:") && !su.starts_with("file:") {
                        *su = format!("{su}{}t={bust}", if su.contains('?') { "&" } else { "?" });
                    }
                }
                if let Ok(bytes) = mojang::download_skin_png(http, &skin_url).await {
                    profile.avatar_data_url = mojang::helm_data_url_from_skin_png(&bytes);
                }
            }
        }
    }

    if profile.avatar_data_url.is_none() {
        profile.avatar_url = format!("{}?t={cache_bust}", avatar_url(&profile.uuid));
        profile.body_url = format!("{}?t={cache_bust}", body_url(&profile.uuid));
        profile.avatar_data_url =
            mojang::fetch_avatar_data_url(http, &profile.uuid, &profile.username, cache_bust).await;
    }

    profile
}

fn texture_cache_key(skin_url: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    skin_url.hash(&mut h);
    h.finish()
}

fn write_temp_png(bytes: &[u8]) -> AppResult<PathBuf> {
    let tmp = std::env::temp_dir().join(format!("pc_skin_{}.png", uuid::Uuid::new_v4()));
    std::fs::write(&tmp, bytes)?;
    Ok(tmp)
}

/// Descarga y aplica skin (Premium → Mojang, Offline → LocalSkin + pack Vanilla + servidores).
pub async fn apply_skin_bytes(
    http: &reqwest::Client,
    bytes: &[u8],
    variant: &str,
    history_name: &str,
    history_url: &str,
) -> AppResult<ApplySkinResult> {
    let tmp = write_temp_png(bytes)?;
    let result = apply_skin_file(http, &tmp, variant, history_name, history_url).await;
    let _ = std::fs::remove_file(&tmp);
    result
}

pub async fn apply_skin_file(
    http: &reqwest::Client,
    path: &Path,
    variant: &str,
    history_name: &str,
    history_url: &str,
) -> AppResult<ApplySkinResult> {
    let variant = if variant == "slim" { "slim" } else { "classic" };

    if let Some(acc) = accounts::active_account() {
        if acc.premium {
            let local = offline::apply_offline_skin_with_variant(path, &acc.username, variant)?;
            pending_premium::save_pending(&acc.id, path, variant)?;

            let upload_result = async {
                let _ = accounts::ensure_valid_token(http, &acc.id).await?;
                mojang::upload_premium_skin(http, &acc.id, path, variant).await
            }
            .await;

            match upload_result {
                Ok(()) => {
                    pending_premium::clear_pending();
                    if !history_url.is_empty() {
                        history::push(history_name, history_url, variant);
                    }
                    return Ok(ApplySkinResult {
                        ok: true,
                        message: "Skin aplicada en el juego y subida a Mojang.".into(),
                        instances: local.instances,
                        server_sync: local.server_sync,
                        premium: true,
                    });
                }
                Err(e) if e.is_connectivity() => {
                    if !history_url.is_empty() {
                        history::push(history_name, history_url, variant);
                    }
                    return Ok(ApplySkinResult {
                        ok: true,
                        message: "Skin aplicada en el juego. Se subirá a Mojang cuando haya conexión."
                            .into(),
                        instances: local.instances,
                        server_sync: local.server_sync,
                        premium: true,
                    });
                }
                Err(e) => return Err(e),
            }
        }
    }

    let username = accounts::active_account()
        .map(|a| a.username)
        .unwrap_or_else(|| "Steve".into());
    let mut result = offline::apply_offline_skin_with_variant(path, &username, variant)?;
    result.premium = false;
    if !history_url.is_empty() {
        history::push(history_name, history_url, variant);
    }
    Ok(result)
}

pub async fn apply_from_url(
    http: &reqwest::Client,
    url: &str,
    variant: &str,
    history_name: &str,
) -> AppResult<ApplySkinResult> {
    if let Some(b64) = url.strip_prefix("data:image/png;base64,") {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .map_err(|e| AppError::msg(format!("PNG base64 inválido: {e}")))?;
        return apply_skin_bytes(http, &bytes, variant, history_name, "").await;
    }
    let bytes = mojang::download_skin_png(http, url).await?;
    apply_skin_bytes(http, &bytes, variant, history_name, url).await
}

pub async fn apply_from_username(
    http: &reqwest::Client,
    username: &str,
    variant: &str,
) -> AppResult<ApplySkinResult> {
    let info = mojang::lookup_player(http, Some(username), None).await?;
    if !info.ok {
        return Err(AppError::msg(
            info.error.unwrap_or_else(|| "Jugador no encontrado".into()),
        ));
    }
    let skin_url = info
        .skin_url
        .ok_or_else(|| AppError::msg("El jugador no tiene skin"))?;
    let model = if variant.is_empty() {
        info.model.as_str()
    } else {
        variant
    };
    apply_from_url(http, &skin_url, model, &info.username).await
}
