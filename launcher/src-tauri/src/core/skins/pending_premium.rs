//! Cola de skins Premium: aplica localmente al instante y sube a Mojang cuando hay red.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::accounts;
use crate::core::paths;
use crate::error::{AppError, AppResult};

use super::mojang;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingPremiumSkin {
    account_id: String,
    variant: String,
}

fn pending_meta_path() -> PathBuf {
    paths::data_dir().join("pending_premium_skin.json")
}

fn pending_png_path() -> PathBuf {
    paths::data_dir().join("pending_premium_skin.png")
}

pub fn has_pending() -> bool {
    pending_meta_path().is_file() && pending_png_path().is_file()
}

pub fn save_pending(account_id: &str, src: &Path, variant: &str) -> AppResult<()> {
    if !src.is_file() {
        return Err(AppError::msg("Archivo de skin no encontrado"));
    }
    std::fs::copy(src, pending_png_path())?;
    let data = PendingPremiumSkin {
        account_id: account_id.to_string(),
        variant: if variant == "slim" {
            "slim".into()
        } else {
            "classic".into()
        },
    };
    std::fs::write(
        pending_meta_path(),
        serde_json::to_string_pretty(&data)?,
    )?;
    Ok(())
}

pub fn clear_pending() {
    let _ = std::fs::remove_file(pending_meta_path());
    let _ = std::fs::remove_file(pending_png_path());
}

/// Sube la skin pendiente a Mojang si hay conexión y token válido.
pub async fn flush_pending(http: &reqwest::Client) -> AppResult<Option<String>> {
    if !has_pending() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(pending_meta_path())?;
    let pending: PendingPremiumSkin = serde_json::from_str(&text)?;
    let _ = accounts::ensure_valid_token(http, &pending.account_id).await?;
    mojang::upload_premium_skin(
        http,
        &pending.account_id,
        &pending_png_path(),
        &pending.variant,
    )
    .await?;
    clear_pending();
    Ok(Some(
        "Skin Premium pendiente subida a Mojang correctamente.".into(),
    ))
}
