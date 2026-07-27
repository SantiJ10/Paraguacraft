//! Iconos personalizados de instancia (128×128, PNG/JPG/WebP).

use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::GenericImageView;
use serde::Serialize;

use crate::core::paths;
use crate::error::{AppError, AppResult};

/// Resolución estándar (launcher oficial de Minecraft).
pub const ICON_SIZE: u32 = 128;
pub const MIN_ICON_SIZE: u32 = 64;
const MAX_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportIconResult {
    pub icon_id: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
}

pub fn icons_dir() -> PathBuf {
    let dir = paths::data_dir().join("instance-icons");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn is_custom_icon(icon: &str) -> bool {
    icon.starts_with("custom:")
}

pub fn resolve_path(icon: &str) -> Option<PathBuf> {
    let id = icon.strip_prefix("custom:")?;
    if id.is_empty() || id.contains("..") || id.contains('/') || id.contains('\\') {
        return None;
    }
    let path = icons_dir().join(format!("{id}.png"));
    path.is_file().then_some(path)
}

fn crop_center_square(img: image::DynamicImage) -> image::DynamicImage {
    let (w, h) = img.dimensions();
    let side = w.min(h);
    let x = (w - side) / 2;
    let y = (h - side) / 2;
    img.crop_imm(x, y, side, side)
}

pub fn import_from_path(source: &Path) -> AppResult<ImportIconResult> {
    let meta = std::fs::metadata(source)?;
    if meta.len() > MAX_BYTES {
        return Err(AppError::msg(format!(
            "La imagen es muy grande (máx {} MB)",
            MAX_BYTES / 1024 / 1024
        )));
    }

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
        return Err(AppError::msg("Formato no soportado. Usa PNG, JPG o WebP."));
    }

    let img = image::open(source).map_err(|e| AppError::msg(format!("No se pudo leer la imagen: {e}")))?;
    let (w, h) = img.dimensions();
    if w < MIN_ICON_SIZE || h < MIN_ICON_SIZE {
        return Err(AppError::msg(format!(
            "La imagen debe ser al menos {MIN_ICON_SIZE}×{MIN_ICON_SIZE} px (actual: {w}×{h})"
        )));
    }

    let processed = crop_center_square(img).resize_exact(ICON_SIZE, ICON_SIZE, FilterType::Lanczos3);
    let id = uuid::Uuid::new_v4().to_string();
    let icon_id = format!("custom:{id}");
    let dest = icons_dir().join(format!("{id}.png"));
    processed
        .save_with_format(&dest, image::ImageFormat::Png)
        .map_err(|e| AppError::msg(format!("No se pudo guardar el icono: {e}")))?;

    Ok(ImportIconResult {
        icon_id,
        path: dest.to_string_lossy().to_string(),
        width: ICON_SIZE,
        height: ICON_SIZE,
    })
}

/// Importa un icono desde bytes (PNG/JPG/WebP/GIF) ya descargados.
pub fn import_from_bytes(bytes: &[u8]) -> AppResult<ImportIconResult> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err(AppError::msg(format!(
            "La imagen es muy grande (máx {} MB)",
            MAX_BYTES / 1024 / 1024
        )));
    }
    let img = image::load_from_memory(bytes)
        .map_err(|e| AppError::msg(format!("No se pudo leer la imagen: {e}")))?;
    let (w, h) = img.dimensions();
    if w < 16 || h < 16 {
        return Err(AppError::msg("La imagen del icono es demasiado pequeña"));
    }
    let processed = crop_center_square(img).resize_exact(ICON_SIZE, ICON_SIZE, FilterType::Lanczos3);
    let id = uuid::Uuid::new_v4().to_string();
    let icon_id = format!("custom:{id}");
    let dest = icons_dir().join(format!("{id}.png"));
    processed
        .save_with_format(&dest, image::ImageFormat::Png)
        .map_err(|e| AppError::msg(format!("No se pudo guardar el icono: {e}")))?;
    Ok(ImportIconResult {
        icon_id,
        path: dest.to_string_lossy().to_string(),
        width: ICON_SIZE,
        height: ICON_SIZE,
    })
}

/// Descarga un icono remoto y lo guarda como `custom:<uuid>`. Si falla, None.
pub async fn import_from_url(client: &reqwest::Client, url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    import_from_bytes(&bytes).ok().map(|r| r.icon_id)
}

/// Devuelve `data:image/png;base64,...` para mostrar en WebView sin asset protocol.
pub fn as_data_url(icon: &str) -> Option<String> {
    let path = resolve_path(icon)?;
    let bytes = std::fs::read(path).ok()?;
    use base64::Engine;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

/// Lee un PNG local (p.ej. pack.png) como data URL.
pub fn file_as_data_url(path: &Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes.is_empty() {
        return None;
    }
    use base64::Engine;
    let mime = if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
    {
        "image/jpeg"
    } else if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("webp"))
    {
        "image/webp"
    } else {
        "image/png"
    };
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
