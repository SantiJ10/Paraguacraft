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
