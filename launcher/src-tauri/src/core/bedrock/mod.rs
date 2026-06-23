//! Minecraft: Bedrock Edition (Windows Store / Xbox).
//!
//! Mismo flujo que `lanzar_bedrock` del launcher Python: requiere cuenta premium
//! (Microsoft), instala el resource pack de branding, abre la app UWP y vigila
//! la sesión (renombrar ventana, RPC, minimizar launcher).

#[cfg(windows)]
mod windows;

use serde::Serialize;

use crate::core::accounts;
use crate::error::{AppError, AppResult};

const PACK_UUID: &str = "a1b2c3d4-e5f6-7890-abcd-ef1234567891";
const MODULE_UUID: &str = "b2c3d4e5-f6a7-8901-bcde-f12345678902";
const PACK_VERSION: [u32; 3] = [1, 0, 0];

/// Logo del menú principal Paraguacraft (mismo asset que Java).
const BRAND_LOGO: &[u8] = crate::core::branding::BEDROCK_TITLE_PNG;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockStatus {
    /// Bedrock solo se puede lanzar en Windows con la app de Microsoft Store.
    pub platform_supported: bool,
    /// Hay datos de Bedrock en `%LOCALAPPDATA%` o ejecutables conocidos.
    pub installed: bool,
    /// La cuenta activa es premium (Microsoft).
    pub premium_allowed: bool,
    pub username: Option<String>,
}

pub fn status() -> BedrockStatus {
    let acc = accounts::active_account();
    BedrockStatus {
        platform_supported: cfg!(windows),
        installed: is_installed(),
        premium_allowed: acc.as_ref().is_some_and(|a| a.premium),
        username: acc.map(|a| a.username),
    }
}

fn is_installed() -> bool {
    #[cfg(windows)]
    {
        windows::has_bedrock_data() || windows::find_exe_paths().iter().any(|p| p.is_file())
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Instala el resource pack «Paraguacraft Branding» en la carpeta Bedrock.
pub fn install_branding_pack() -> AppResult<()> {
    #[cfg(windows)]
    {
        let mojang = windows::mojang_dir()
            .ok_or_else(|| AppError::msg("Bedrock no encontrado (instalá desde Xbox / Microsoft Store)"))?;
        install_pack_at(&mojang)
    }
    #[cfg(not(windows))]
    {
        Err(AppError::msg("Bedrock solo está disponible en Windows"))
    }
}

fn install_pack_at(mojang_dir: &std::path::Path) -> AppResult<()> {
    use std::fs;

    let pack_dir = mojang_dir.join("resource_packs/ParaguacraftBranding");
    let tex_dir = pack_dir.join("textures/ui");
    fs::create_dir_all(&tex_dir)?;

    for name in ["title.png", "title2.png", "mojanglogo.png"] {
        fs::write(tex_dir.join(name), BRAND_LOGO)
            .map_err(|e| AppError::msg(format!("No se pudo escribir {name}: {e}")))?;
    }

    fs::write(pack_dir.join("pack_icon.png"), BRAND_LOGO)
        .map_err(|e| AppError::msg(format!("No se pudo escribir pack_icon.png: {e}")))?;

    let manifest = serde_json::json!({
        "format_version": 2,
        "header": {
            "name": "Paraguacraft Branding",
            "description": "Logo personalizado — Paraguacraft Launcher",
            "uuid": PACK_UUID,
            "version": PACK_VERSION,
            "min_engine_version": [1, 16, 0]
        },
        "modules": [{
            "type": "resources",
            "uuid": MODULE_UUID,
            "version": PACK_VERSION
        }]
    });
    fs::write(
        pack_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    let mcpe = mojang_dir.join("minecraftpe");
    fs::create_dir_all(&mcpe)?;
    let grp_path = mcpe.join("global_resource_packs.json");
    let mut existing: Vec<serde_json::Value> = if grp_path.is_file() {
        fs::read_to_string(&grp_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };
    if !existing.iter().any(|p| p.get("pack_id").and_then(|v| v.as_str()) == Some(PACK_UUID)) {
        existing.insert(
            0,
            serde_json::json!({ "pack_id": PACK_UUID, "version": PACK_VERSION }),
        );
        fs::write(&grp_path, serde_json::to_string_pretty(&existing)?)?;
    }

    Ok(())
}

/// Lanza Bedrock. Solo cuentas premium (Microsoft).
pub fn launch(username: &str) -> AppResult<()> {
    if !accounts::active_account().is_some_and(|a| a.premium) {
        return Err(AppError::msg(
            "Se necesita cuenta Premium (Microsoft) para jugar Minecraft: Bedrock Edition",
        ));
    }

    let _ = install_branding_pack();

    #[cfg(windows)]
    {
        windows::open_bedrock_app(username)
    }
    #[cfg(not(windows))]
    {
        let _ = username;
        Err(AppError::msg("Bedrock solo está disponible en Windows"))
    }
}

#[cfg(windows)]
pub use windows::watch_session;

#[cfg(not(windows))]
pub fn watch_session(
    _app: tauri::AppHandle,
    _username: String,
    _close_on_launch: bool,
) {
}
