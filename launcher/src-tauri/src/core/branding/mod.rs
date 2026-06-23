//! Branding Paraguacraft — logos en menú Java y Bedrock.
//!
//! Los resource packs se **pre-generan en compile time** (`build.rs`).
//! En cada lanzamiento solo se copia el zip embebido correcto y se
//! asegura que `options.txt` lo tenga activo (sin reprocesar imágenes).

mod deploy;
mod options;
mod version;

use std::path::Path;

pub use version::parse_mc_version;

pub const PACK_NAME: &str = "ParaguacraftBrandPack";

/// Logo 512×128 pre-renderizado para Bedrock (`build.rs`).
pub const BEDROCK_TITLE_PNG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bedrock_title.png"));

/// `paraguacraft-pvp` incluye su propio menú personalizado.
pub fn should_apply(loader: &str) -> bool {
    loader != "paraguacraft-pvp"
}

/// Instala el pack de branding pre-generado antes del lanzamiento.
pub fn inject_logos(game_dir: &Path, mc_version: &str, min_graphics: bool) -> crate::error::AppResult<()> {
    let ver = parse_mc_version(mc_version);
    let profile = version::pack_profile(ver);
    deploy::deploy(game_dir, profile)?;
    options::ensure_enabled(game_dir, ver, profile, min_graphics)?;
    Ok(())
}

/// Tras modificar texturas del pack (skin offline), reconstruye el zip si la instancia lo usa.
pub fn rebuild_pack_zip(game_dir: &Path) -> crate::error::AppResult<()> {
    let zip_path = game_dir
        .join("resourcepacks")
        .join(format!("{PACK_NAME}.zip"));
    if zip_path.is_file() {
        deploy::rebuild_zip_if_needed(game_dir)?;
    }
    Ok(())
}
