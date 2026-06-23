//! Import de instancias externas a la biblioteca Paraguacraft.
//!
//! Crea una instancia local equivalente y, cuando el launcher de origen tiene
//! una carpeta de juego propia (Prism/PolyMC/MultiMC), copia el contenido del
//! jugador (mods, config, resourcepacks, options). Para launchers que comparten
//! `.minecraft` (Vanilla/TLauncher/SK/Lunar) solo se registra el perfil para no
//! duplicar los mundos globales.

use std::path::PathBuf;

use crate::core::paths;
use crate::error::{AppError, AppResult};
use crate::models::Instance;

use super::{instance_dir, profiles, scan};

/// Subcarpetas/archivos del jugador que se copian al importar.
const COPY_ITEMS: [&str; 5] = ["mods", "config", "resourcepacks", "shaderpacks", "options.txt"];

pub fn import(ext_id: &str) -> AppResult<Instance> {
    let inst = scan::scan_external()
        .into_iter()
        .find(|i| i.id == ext_id)
        .ok_or_else(|| AppError::msg("Instancia externa no encontrada"))?;

    let local = profiles::create(
        &format!("{} (importada)", inst.name),
        &inst.mc_version,
        &inst.loader,
        &inst.loader_version,
        &inst.icon,
        4096,
    )?;

    if let Some(src_game) = source_game_dir(ext_id) {
        let dst = instance_dir(&local.id);
        for item in COPY_ITEMS {
            let from = src_game.join(item);
            let to = dst.join(item);
            if from.is_dir() {
                let _ = profiles::copy_dir_all(&from, &to);
            } else if from.is_file() {
                let _ = std::fs::copy(&from, &to);
            }
        }
    }

    // Releer para reflejar mod_count tras la copia.
    let dir = instance_dir(&local.id);
    let meta = super::read_meta(&local.id).ok_or_else(|| AppError::msg("Sin metadata"))?;
    Ok(meta.into_instance(&local.id, &dir))
}

/// Carpeta de juego del origen externo (para importar o instalar desde la tienda).
pub fn external_game_dir(ext_id: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = ext_id.splitn(3, "::").collect();
    if parts.len() != 3 {
        return None;
    }
    let (source, key) = (parts[1], parts[2]);
    match source {
        "prism" => prism_game_dir(key),
        // Vanilla, TLauncher, SK y Lunar comparten `.minecraft` global.
        "vanilla" | "tlauncher" | "sklauncher" | "lunar" => Some(paths::default_minecraft_dir()),
        _ => Some(paths::default_minecraft_dir()),
    }
}

/// Alias usado por import().
pub fn source_game_dir(ext_id: &str) -> Option<PathBuf> {
    external_game_dir(ext_id)
}

fn prism_game_dir(key: &str) -> Option<PathBuf> {
    for (s, root) in paths::external_launcher_roots() {
        if s != "prism" {
            continue;
        }
        let game = root.join(key).join(".minecraft");
        if game.is_dir() {
            return Some(game);
        }
        // Algunas versiones usan `minecraft` sin punto.
        let alt = root.join(key).join("minecraft");
        if alt.is_dir() {
            return Some(alt);
        }
    }
    None
}
