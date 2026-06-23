//! APIs Mojang / Minecraft Services para consultar y subir skins Premium.

use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::multipart;
use serde::Deserialize;

use crate::core::accounts::{self, store};
use crate::error::{AppError, AppResult};

const PROFILE_URL: &str = "https://api.mojang.com/users/profiles/minecraft";
const SESSION_URL: &str = "https://sessionserver.mojang.com/session/minecraft/profile";
const UPLOAD_URL: &str = "https://api.minecraftservices.com/minecraft/profile/skins";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinLookup {
    pub ok: bool,
    pub username: String,
    pub uuid: String,
    pub skin_url: Option<String>,
    pub cape_url: Option<String>,
    /// `classic` (Steve) o `slim` (Alex).
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NameLookup {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SessionProfile {
    name: Option<String>,
    properties: Option<Vec<SessionProperty>>,
}

#[derive(Debug, Deserialize)]
struct SessionProperty {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct TexturesPayload {
    textures: Option<TexturesMap>,
}

#[derive(Debug, Deserialize)]
struct TexturesMap {
    #[serde(rename = "SKIN")]
    skin: Option<TextureEntry>,
    #[serde(rename = "CAPE")]
    cape: Option<TextureEntry>,
}

#[derive(Debug, Deserialize)]
struct TextureEntry {
    url: Option<String>,
    metadata: Option<TextureMeta>,
}

#[derive(Debug, Deserialize)]
struct TextureMeta {
    model: Option<String>,
}

pub fn normalize_uuid(uuid: &str) -> String {
    uuid.replace('-', "")
}

pub fn format_uuid(raw: &str) -> String {
    let clean = normalize_uuid(raw);
    if clean.len() != 32 {
        return raw.to_string();
    }
    format!(
        "{}-{}-{}-{}-{}",
        &clean[0..8],
        &clean[8..12],
        &clean[12..16],
        &clean[16..20],
        &clean[20..32]
    )
}

pub async fn resolve_username(http: &reqwest::Client, username: &str) -> AppResult<(String, String)> {
    let name = username.trim();
    if name.is_empty() {
        return Err(AppError::msg("Nombre vacío"));
    }
    let resp = http
        .get(format!("{PROFILE_URL}/{name}"))
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Mojang: {e}")))?;
    if resp.status() == reqwest::StatusCode::NO_CONTENT || resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::msg(format!("Jugador '{name}' no encontrado")));
    }
    if !resp.status().is_success() {
        return Err(AppError::msg(format!("Mojang HTTP {}", resp.status())));
    }
    let data: NameLookup = resp
        .json()
        .await
        .map_err(|e| AppError::msg(format!("Respuesta Mojang inválida: {e}")))?;
    Ok((data.name, format_uuid(&data.id)))
}

pub async fn lookup_player(
    http: &reqwest::Client,
    username: Option<&str>,
    uuid: Option<&str>,
) -> AppResult<SkinLookup> {
    let (name, id) = if let Some(u) = uuid.filter(|s| !s.is_empty()) {
        let formatted = format_uuid(u);
        let uname = if let Some(n) = username.filter(|s| !s.is_empty()) {
            n.to_string()
        } else {
            formatted.clone()
        };
        (uname, formatted)
    } else if let Some(n) = username.filter(|s| !s.is_empty()) {
        resolve_username(http, n).await?
    } else if let Some(acc) = accounts::active_account() {
        (acc.username.clone(), acc.uuid.clone())
    } else {
        return Ok(SkinLookup {
            ok: false,
            username: String::new(),
            uuid: String::new(),
            skin_url: None,
            cape_url: None,
            model: "classic".into(),
            error: Some("Sin cuenta ni jugador especificado".into()),
        });
    };

    let resp = http
        .get(format!("{SESSION_URL}/{}", normalize_uuid(&id)))
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Session server: {e}")))?;
    if !resp.status().is_success() {
        return Ok(SkinLookup {
            ok: false,
            username: name,
            uuid: id,
            skin_url: None,
            cape_url: None,
            model: "classic".into(),
            error: Some(format!("Session HTTP {}", resp.status())),
        });
    }

    let profile: SessionProfile = resp
        .json()
        .await
        .map_err(|e| AppError::msg(format!("Perfil inválido: {e}")))?;
    let display_name = profile.name.unwrap_or(name);

    let tex_prop = profile
        .properties
        .unwrap_or_default()
        .into_iter()
        .find(|p| p.name == "textures");

    let Some(tex_prop) = tex_prop else {
        return Ok(SkinLookup {
            ok: true,
            username: display_name,
            uuid: id,
            skin_url: None,
            cape_url: None,
            model: "classic".into(),
            error: None,
        });
    };

    let decoded = STANDARD
        .decode(tex_prop.value.as_bytes())
        .map_err(|e| AppError::msg(format!("Base64 texturas: {e}")))?;
    let payload: TexturesPayload = serde_json::from_slice(&decoded)
        .map_err(|e| AppError::msg(format!("JSON texturas: {e}")))?;

    let textures = payload.textures.unwrap_or(TexturesMap {
        skin: None,
        cape: None,
    });
    let skin_entry = textures.skin;
    let skin_url = skin_entry.as_ref().and_then(|s| s.url.clone());
    let cape_url = textures.cape.and_then(|c| c.url);
    let model = skin_entry
        .and_then(|s| s.metadata)
        .and_then(|m| m.model)
        .filter(|m| m == "slim")
        .map(|_| "slim".to_string())
        .unwrap_or_else(|| "classic".into());

    Ok(SkinLookup {
        ok: true,
        username: display_name,
        uuid: id,
        skin_url,
        cape_url,
        model,
        error: None,
    })
}

pub async fn download_skin_png(http: &reqwest::Client, url: &str) -> AppResult<Vec<u8>> {
    let resp = http
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Descarga skin: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::msg(format!("Descarga skin HTTP {}", resp.status())));
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| AppError::msg(format!("Bytes skin: {e}")))
}

pub async fn fetch_image_data_url(http: &reqwest::Client, url: &str) -> Option<String> {
    let bytes = download_skin_png(http, url).await.ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

/// Cabeza 2D recortada del PNG de skin (64×64) — siempre al día con Mojang.
pub fn helm_data_url_from_skin_png(bytes: &[u8]) -> Option<String> {
    use image::imageops::{overlay, FilterType};
    use image::GenericImageView;
    use std::io::Cursor;

    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    if w < 64 || h < 64 {
        return None;
    }
    const OUT: u32 = 64;
    let mut face = image::imageops::resize(&img.view(8, 8, 8, 8).to_image(), OUT, OUT, FilterType::Nearest);
    if w >= 64 && h >= 64 {
        let hat = image::imageops::resize(&img.view(40, 8, 8, 8).to_image(), OUT, OUT, FilterType::Nearest);
        overlay(&mut face, &hat, 0, 0);
    }
    let mut buf = Vec::new();
    face.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png).ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(buf)))
}

/// Cabeza 2D vía Minotar (fallback si falla la textura Mojang).
pub async fn fetch_avatar_data_url(
    http: &reqwest::Client,
    uuid: &str,
    username: &str,
    cache_bust: u64,
) -> Option<String> {
    let clean = normalize_uuid(uuid);
    for id in [clean.as_str(), username.trim()] {
        if id.is_empty() {
            continue;
        }
        let url = format!("https://minotar.net/helm/{id}/64.png?v={cache_bust}");
        if let Some(data) = fetch_image_data_url(http, &url).await {
            return Some(data);
        }
    }
    None
}

pub async fn save_skin_to_library(
    http: &reqwest::Client,
    url: &str,
    name: &str,
) -> AppResult<String> {
    let bytes = download_skin_png(http, url).await?;
    let safe: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .take(40)
        .collect();
    let safe = if safe.is_empty() { "skin".into() } else { safe };
    let path = crate::core::paths::skins_library_dir().join(format!("{safe}.png"));
    std::fs::write(&path, &bytes)?;
    Ok(path.to_string_lossy().to_string())
}

pub async fn upload_premium_skin(
    http: &reqwest::Client,
    account_id: &str,
    skin_path: &Path,
    variant: &str,
) -> AppResult<()> {
    let token = store::get_token(account_id).ok_or_else(|| AppError::msg("Sin token Premium"))?;
    let bytes = std::fs::read(skin_path)?;
    let part = multipart::Part::bytes(bytes)
        .file_name("skin.png".to_string())
        .mime_str("image/png")
        .map_err(|e| AppError::msg(e.to_string()))?;
    let form = multipart::Form::new()
        .part("file", part)
        .text("variant", variant.to_string());

    let resp = http
        .post(UPLOAD_URL)
        .header("Authorization", format!("Bearer {}", token.mc_access_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Subida Mojang: {e}")))?;

    if resp.status().is_success() {
        return Ok(());
    }
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::msg(
            "Token expirado. Volvé a iniciar sesión con Microsoft.",
        ));
    }
    Err(AppError::msg(format!(
        "Mojang rechazó la skin (HTTP {status}): {}",
        body.chars().take(120).collect::<String>()
    )))
}

pub fn active_can_upload_premium() -> bool {
    accounts::active_account().is_some_and(|a| a.premium)
}
