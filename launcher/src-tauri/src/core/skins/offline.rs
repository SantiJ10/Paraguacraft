//! Skins No-Premium: local por nick + SkinsRestorer (NO reescribir default Steve/Alex).
//!
//! Antes se copiaba la skin del usuario a `steve.png`/`alex.png` del BrandPack, y **todos**
//! los jugadores offline del cliente local se veían con esa skin. Ahora:
//! - LocalSkin de CustomSkinLoader: solo `{USERNAME}.png` del player activo
//! - SkinsRestorer en servidores locales
//! - Brand pack se **limpia** de texturas de jugador envenenadas

use std::path::{Path, PathBuf};

use crate::core::accounts;
use crate::core::branding;
use crate::core::instances;
use crate::core::loaders;
use crate::core::paths;
use crate::core::servers;
use crate::error::{AppError, AppResult};

pub const OFFLINE_SKIN_FILE: &str = "paraguacraft_offline_skin.png";
const FACE_CACHE_FILE: &str = "paraguacraft_offline_skin_face.png";

/// Ruta global donde se guarda la skin offline del usuario.
pub fn global_skin_path() -> PathBuf {
    paths::default_minecraft_dir().join(OFFLINE_SKIN_FILE)
}

/// Cache de cara 2D (avatar launcher) recortada de la skin global.
pub fn global_face_path() -> PathBuf {
    paths::default_minecraft_dir().join(FACE_CACHE_FILE)
}

pub fn has_global_skin() -> bool {
    global_skin_path().is_file()
}

/// Copia la skin al almacen global y regenera el recorte de cara.
pub fn store_global_skin(src: &Path) -> AppResult<PathBuf> {
    if !src.is_file() {
        return Err(AppError::msg("Archivo de skin no encontrado"));
    }
    let dest = global_skin_path();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, &dest)?;
    refresh_face_cache();
    Ok(dest)
}

/// Recorta la cara y la escribe en `global_face_path`. Fallos silenciosos.
pub fn refresh_face_cache() {
    let skin = global_skin_path();
    let Ok(bytes) = std::fs::read(&skin) else {
        return;
    };
    let Some(face_png) = crate::core::skins::mojang::helm_png_bytes_from_skin_png(&bytes) else {
        let _ = std::fs::remove_file(global_face_path());
        return;
    };
    let dest = global_face_path();
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(dest, face_png);
}

/// Elimina steve/alex inyectados por versiones viejas del launcher (veneno multiplayer).
fn purge_default_player_textures(pack_path: &Path) {
    let victims = [
        pack_path.join("assets/minecraft/textures/entity/player/wide/steve.png"),
        pack_path.join("assets/minecraft/textures/entity/player/slim/alex.png"),
        pack_path.join("assets/minecraft/textures/entity/player/slim/steve.png"),
        pack_path.join("assets/minecraft/textures/entity/player/wide/alex.png"),
        pack_path.join("assets/minecraft/textures/entity/steve.png"),
        pack_path.join("assets/minecraft/textures/entity/alex.png"),
    ];
    for p in victims {
        let _ = std::fs::remove_file(p);
    }
}

fn rebuild_brand_pack_zip(game_dir: &Path) -> AppResult<()> {
    branding::rebuild_pack_zip(game_dir)
}

/// Ruta LocalSkin de CustomSkinLoader para un nick.
pub fn local_skin_path(game_dir: &Path, username: &str) -> PathBuf {
    game_dir
        .join("CustomSkinLoader")
        .join("LocalSkin")
        .join("skins")
        .join(format!("{}.png", username.trim()))
}

/// Escribe solo la skin del usuario activo (por nick). No toca defaults de Minecraft.
pub fn write_local_skin(game_dir: &Path, username: &str, skin_path: &Path) -> AppResult<()> {
    let user = username.trim();
    if user.is_empty() || !skin_path.is_file() {
        return Ok(());
    }
    let dest = local_skin_path(game_dir, user);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Limpia otras LocalSkin residuales del historial del usuario (evita nicks viejos).
    // Solo borra skins del account actual? Better: keep all but ensure current is correct.
    // History collision: if someone else has nick from old cache wrong - we only write current user.
    std::fs::copy(skin_path, &dest)?;
    // Asegura carpeta Capes vacia por si acaso.
    let capes = game_dir.join("CustomSkinLoader/LocalSkin/capes");
    let _ = std::fs::create_dir_all(capes);
    Ok(())
}

/// Quita del BrandPack las texturas de jugador sobrescritas y reescribe LocalSkin.
pub fn apply_to_game_dir(game_dir: &Path, skin_path: &Path, mc_version: &str) -> AppResult<()> {
    if !skin_path.is_file() {
        return Ok(());
    }

    // 1) Limpiar enveneno BrandPack (crítico)
    let pack_path = game_dir.join("resourcepacks").join(branding::PACK_NAME);
    if pack_path.is_dir() {
        purge_default_player_textures(&pack_path);
        let _ = rebuild_brand_pack_zip(game_dir);
    }

    // 2) LocalSkin por nick activo
    let username = accounts::active_account()
        .map(|a| a.username)
        .unwrap_or_else(|| "Steve".into());
    write_local_skin(game_dir, &username, skin_path)?;

    // 3) PvP Modern: property solo para el cliente local (SkinManager lo usa si hay mixin)
    write_modern_skin_property(game_dir, skin_path);

    // Alinear pack format brand (sin reinyectar skin)
    if pack_path.is_dir() {
        let mcmeta = pack_path.join("pack.mcmeta");
        if !mcmeta.is_file() {
            let meta = crate::core::branding::pack_mcmeta_json(mc_version);
            let _ = std::fs::write(mcmeta, meta);
        }
        let _ = branding::sync_brand_options(game_dir, mc_version, false);
    }

    Ok(())
}

/// Skins unificadas: en PvP Modern, `customSkinUrl=file://…` para SkinManager (solo local).
fn write_modern_skin_property(game_dir: &Path, skin_path: &Path) {
    if accounts::active_account().map(|a| a.premium).unwrap_or(false) {
        return;
    }
    let folder = game_dir.file_name().and_then(|n| n.to_str());
    let meta = folder.and_then(instances::read_meta);
    let is_modern = meta
        .map(|m| loaders::normalize(&m.loader) == "paraguacraft-pvp-modern")
        .unwrap_or(false);
    if !is_modern {
        return;
    }
    let props_path = game_dir.join("config").join("paraguacraftpvp-modern.properties");
    let Some(parent) = props_path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let mut lines: Vec<String> = Vec::new();
    let mut replaced = false;
    if let Ok(existing) = std::fs::read_to_string(&props_path) {
        for line in existing.lines() {
            if line.trim_start().starts_with("customSkinUrl=") {
                lines.push(format!("customSkinUrl=file://{}", skin_path.to_string_lossy()));
                replaced = true;
            } else {
                lines.push(line.to_string());
            }
        }
    }
    if !replaced {
        lines.push(format!("customSkinUrl=file://{}", skin_path.to_string_lossy()));
    }
    let _ = std::fs::write(&props_path, format!("{}\n", lines.join("\n")));
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

/// Al cambiar nick offline: copiar PNG SkinsRestorer y LocalSkin al nick nuevo.
pub fn on_offline_username_changed(old_username: &str, new_username: &str) {
    let old = old_username.trim();
    let new = new_username.trim();
    if old.is_empty() || new.is_empty() || old.eq_ignore_ascii_case(new) {
        return;
    }

    let rel_paths = [
        PathBuf::from("plugins/SkinsRestorer/skins"),
        PathBuf::from("plugins/SkinsRestorer/Skins"),
        PathBuf::from("config/skinsrestorer/skins"),
        PathBuf::from("config/SkinsRestorer/skins"),
        PathBuf::from("mods/skinsrestorer/skins"),
    ];
    for server_dir in servers::list_server_dirs() {
        for rel in &rel_paths {
            let dir = server_dir.join(rel);
            let src = dir.join(format!("{old}.png"));
            let dest = dir.join(format!("{new}.png"));
            if src.is_file() {
                let _ = std::fs::copy(&src, &dest);
            }
        }
    }

    let global = global_skin_path();
    if global.is_file() {
        let _ = sync_to_local_servers(new, &global);
        // Actualizar LocalSkin en todas las instancias
        if let Ok(rd) = std::fs::read_dir(paths::instances_dir()) {
            for entry in rd.flatten() {
                if entry.path().is_dir() {
                    let _ = write_local_skin(&entry.path(), new, &global);
                    // Quitar archivo del nick viejo para no dejar residual
                    let old_local = local_skin_path(&entry.path(), old);
                    let _ = std::fs::remove_file(old_local);
                }
            }
        }
    }
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

/// Flujo completo: guardar global, LocalSkin + limpiar brand, servidores.
pub fn apply_offline_skin(src: &Path, username: &str) -> AppResult<ApplySkinResult> {
    store_global_skin(src)?;
    let global = global_skin_path();
    let instances = apply_to_all_instances(&global)?;
    let server_sync = sync_to_local_servers(username, &global);
    let mut msg = if instances > 0 {
        format!("Skin aplicada en {instances} instancia(s) (solo tu nick).")
    } else {
        "Skin guardada: se aplicara al abrir el juego.".into()
    };
    if server_sync > 0 {
        msg.push_str(&format!(" SkinsRestorer actualizado en {server_sync} carpeta(s)."));
    }
    msg.push_str(" Multiplayer: otros se ven por Ely.by / servidor, no por tu historial. Reentra al mundo.");
    Ok(ApplySkinResult {
        ok: true,
        message: msg,
        instances,
        server_sync,
        premium: false,
    })
}

/// Antes de lanzar: limpia brand pack y escribe LocalSkin de la cuenta activa.
pub fn ensure_for_launch(game_dir: &Path, mc_version: &str) -> AppResult<()> {
    // Siempre purgar defaults envenenados (aunque no haya skin actual)
    let pack_path = game_dir.join("resourcepacks").join(branding::PACK_NAME);
    if pack_path.is_dir() {
        purge_default_player_textures(&pack_path);
        let _ = rebuild_brand_pack_zip(game_dir);
    }

    let global = global_skin_path();
    if global.is_file() {
        apply_to_game_dir(game_dir, &global, mc_version)?;
    }
    Ok(())
}
