//! Inyeccion de skins No-Premium (resource pack + SkinsRestorer).
//! Replica la logica de `core.py` / `paragua.py`.

use std::path::{Path, PathBuf};

use crate::core::branding;
use crate::core::instances;
use crate::core::paths;
use crate::core::servers;
use crate::error::{AppError, AppResult};

pub const OFFLINE_SKIN_FILE: &str = "paraguacraft_offline_skin.png";

/// Ruta global donde se guarda la skin offline del usuario.
pub fn global_skin_path() -> PathBuf {
    paths::default_minecraft_dir().join(OFFLINE_SKIN_FILE)
}

pub fn has_global_skin() -> bool {
    global_skin_path().is_file()
}

/// Copia la skin al almacen global.
pub fn store_global_skin(src: &Path) -> AppResult<PathBuf> {
    if !src.is_file() {
        return Err(AppError::msg("Archivo de skin no encontrado"));
    }
    let dest = global_skin_path();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, &dest)?;
    Ok(dest)
}

fn pack_format_for_mc(mc: &str) -> u32 {
    let parts: Vec<&str> = mc.split('.').collect();
    let minor = parts.get(1).and_then(|p| p.parse::<u32>().ok()).unwrap_or(21);
    let patch = parts.get(2).and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);
    match (minor, patch) {
        (7, _) | (8, _) => 1,
        (9, 0..=2) => 2,
        (9, _) => 3,
        (10, _) => 4,
        (11, _) => 5,
        (12, _) => 6,
        (13, _) => 7,
        (14, _) => 8,
        (15, _) => 9,
        (16, _) => 10,
        (17, _) => 12,
        (18, _) => 15,
        (19, _) => 18,
        (20, _) => 22,
        _ => 34,
    }
}

fn copy_skin_textures(pack_path: &Path, skin_path: &Path) -> AppResult<()> {
    let wide = pack_path.join("assets/minecraft/textures/entity/player/wide");
    let slim = pack_path.join("assets/minecraft/textures/entity/player/slim");
    let old = pack_path.join("assets/minecraft/textures/entity");
    for dir in [&wide, &slim, &old] {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::copy(skin_path, wide.join("steve.png"))?;
    std::fs::copy(skin_path, slim.join("alex.png"))?;
    std::fs::copy(skin_path, old.join("steve.png"))?;
    Ok(())
}

fn rebuild_brand_pack_zip(game_dir: &Path) -> AppResult<()> {
    branding::rebuild_pack_zip(game_dir)
}

fn sync_resource_pack_options(game_dir: &Path, mc_version: &str, use_zip: bool) -> AppResult<()> {
    let options_path = game_dir.join("options.txt");
    let mut lines = if options_path.is_file() {
        std::fs::read_to_string(&options_path)?
            .lines()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let pack_visible = if use_zip {
        format!("{}.zip", branding::PACK_NAME)
    } else {
        branding::PACK_NAME.to_string()
    };

    let major = mc_version.split('.').nth(1).and_then(|p| p.parse::<u32>().ok()).unwrap_or(21);

    lines.retain(|l| !l.starts_with("resourcePacks:") && !l.starts_with("texturepack:"));

    if major < 6 {
        lines.push(format!("texturepack:{pack_visible}"));
    } else if major < 13 {
        lines.push(format!("resourcePacks:[\"{pack_visible}\"]"));
    } else {
        lines.push(format!("resourcePacks:[\"vanilla\",\"file/{pack_visible}\"]"));
    }

    let content = lines
        .into_iter()
        .map(|l| {
            if l.ends_with('\n') {
                l
            } else {
                format!("{l}\n")
            }
        })
        .collect::<String>();
    std::fs::write(&options_path, content)?;
    Ok(())
}

/// Aplica la skin en el directorio de juego de una instancia.
pub fn apply_to_game_dir(game_dir: &Path, skin_path: &Path, mc_version: &str) -> AppResult<()> {
    if !skin_path.is_file() {
        return Ok(());
    }
    let pack_path = game_dir.join("resourcepacks").join(branding::PACK_NAME);
    std::fs::create_dir_all(&pack_path)?;
    let mcmeta = pack_path.join("pack.mcmeta");
    if !mcmeta.is_file() {
        let fmt = pack_format_for_mc(mc_version);
        std::fs::write(
            &mcmeta,
            format!(r#"{{"pack":{{"pack_format":{fmt},"description":"Paraguacraft Skin"}}}}"#),
        )?;
    }
    copy_skin_textures(&pack_path, skin_path)?;
    rebuild_brand_pack_zip(game_dir)?;
    let use_zip = game_dir
        .join("resourcepacks")
        .join(format!("{}.zip", branding::PACK_NAME))
        .is_file();
    let _ = sync_resource_pack_options(game_dir, mc_version, use_zip);
    Ok(())
}

/// Sincroniza la skin con SkinsRestorer en servidores locales.
pub fn sync_to_local_servers(username: &str, skin_path: &Path) -> u32 {
    if username.trim().is_empty() || !skin_path.is_file() {
        return 0;
    }
    let user = username.trim();
    let rel_paths = [
        PathBuf::from("plugins/SkinsRestorer/skins"),
        PathBuf::from("plugins/SkinsRestorer/Skins"),
        PathBuf::from("config/skinsrestorer/skins"),
        PathBuf::from("config/SkinsRestorer/skins"),
        PathBuf::from("mods/skinsrestorer/skins"),
    ];
    let mut applied = 0u32;
    for server_dir in servers::list_server_dirs() {
        for rel in &rel_paths {
            let dest_dir = server_dir.join(rel);
            if std::fs::create_dir_all(&dest_dir).is_err() {
                continue;
            }
            let dest = dest_dir.join(format!("{user}.png"));
            if std::fs::copy(skin_path, &dest).is_ok() {
                applied += 1;
            }
        }
    }
    applied
}

/// Aplica la skin global a todas las instancias Paraguacraft conocidas.
pub fn apply_to_all_instances(skin_path: &Path) -> AppResult<u32> {
    let mut count = 0u32;
    let root = paths::instances_dir();
    if !root.is_dir() {
        return Ok(0);
    }
    for entry in std::fs::read_dir(&root)?.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        let meta = instances::read_meta(&folder);
        let mc = meta
            .as_ref()
            .map(|m| m.mc_version.as_str())
            .unwrap_or("1.21.1");
        apply_to_game_dir(&entry.path(), skin_path, mc)?;
        count += 1;
    }
    Ok(count)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySkinResult {
    pub ok: bool,
    pub message: String,
    pub instances: u32,
    pub server_sync: u32,
    pub premium: bool,
}

/// Flujo completo: guardar global, instancias y servidores.
pub fn apply_offline_skin(src: &Path, username: &str) -> AppResult<ApplySkinResult> {
    store_global_skin(src)?;
    let global = global_skin_path();
    let instances = apply_to_all_instances(&global)?;
    let server_sync = sync_to_local_servers(username, &global);
    let mut msg = if instances > 0 {
        format!("Skin aplicada en {instances} instancia(s).")
    } else {
        "Skin guardada: se aplicara al abrir el juego.".into()
    };
    if server_sync > 0 {
        msg.push_str(&format!(" SkinsRestorer actualizado en {server_sync} carpeta(s)."));
    }
    msg.push_str(" Reentra al mundo/servidor para verla.");
    Ok(ApplySkinResult {
        ok: true,
        message: msg,
        instances,
        server_sync,
        premium: false,
    })
}

/// Antes de lanzar: asegura la skin en la instancia concreta.
pub fn ensure_for_launch(game_dir: &Path, mc_version: &str) -> AppResult<()> {
    let global = global_skin_path();
    if global.is_file() {
        apply_to_game_dir(game_dir, &global, mc_version)?;
    }
    Ok(())
}
