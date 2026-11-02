//! Skins sin cuenta oficial (No-Premium):
//! - Modded (Fabric/Quilt/Forge/NeoForge/OptiFine/Optimized/PvP): LocalSkin por nick en
//!   `CustomSkinLoader/LocalSkin/skins/{USERNAME}.png` (con prioridad #1 sobre Ely.by/Mojang)
//!   + modelo `classic`/`slim` + sincronización en servidores locales (`SkinsRestorer`).
//! - Vanilla puro: pack limpio dedicado `Paraguacraft-MiSkin.zip` (cubriendo los 9 modelos
//!   default de MC 1.19.3+ `wide/` y `slim/` además de `steve.png`/`alex.png` legacy), que se
//!   desactiva/limpia automáticamente en instancias con mods para no afectar otros jugadores.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::accounts;
use crate::core::branding;
use crate::core::instances;
use crate::core::loaders;
use crate::core::paths;
use crate::core::servers;
use crate::error::{AppError, AppResult};

pub const OFFLINE_SKIN_FILE: &str = "paraguacraft_offline_skin.png";
pub const OFFLINE_CAPE_FILE: &str = "paraguacraft_offline_cape.png";
pub const VANILLA_SKIN_PACK_ZIP: &str = "Paraguacraft-MiSkin.zip";
const FACE_CACHE_FILE: &str = "paraguacraft_offline_skin_face.png";
const OFFLINE_SKIN_META_FILE: &str = "paraguacraft_offline_skin_meta.json";

const MODERN_DEFAULT_SKINS: &[&str] = &[
    "steve", "alex", "ari", "efe", "kai", "makena", "noor", "sunny", "zuri",
];

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct OfflineSkinMeta {
    #[serde(default)]
    variant: String,
    #[serde(default)]
    per_user_variant: std::collections::HashMap<String, String>,
}

fn meta_path() -> PathBuf {
    paths::default_minecraft_dir().join(OFFLINE_SKIN_META_FILE)
}

fn read_skin_meta() -> OfflineSkinMeta {
    crate::config::read_json::<OfflineSkinMeta>(&meta_path()).unwrap_or_default()
}

fn save_skin_meta(meta: &OfflineSkinMeta) {
    let _ = crate::config::write_json_atomic(&meta_path(), meta);
}

fn per_user_skins_dir() -> PathBuf {
    paths::default_minecraft_dir().join("paraguacraft_offline_skins")
}

fn sanitize_nick(username: &str) -> String {
    username
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

pub fn user_skin_path(username: &str) -> Option<PathBuf> {
    let clean = sanitize_nick(username);
    if clean.is_empty() {
        return None;
    }
    Some(per_user_skins_dir().join(format!("{clean}.png")))
}

pub fn effective_skin_path_for(username: &str) -> Option<PathBuf> {
    if let Some(p) = user_skin_path(username) {
        if p.is_file() {
            return Some(p);
        }
    }
    let global = global_skin_path();
    if global.is_file() {
        Some(global)
    } else {
        None
    }
}

pub fn variant_for_user(username: &str) -> String {
    let meta = read_skin_meta();
    let key = username.trim().to_ascii_lowercase();
    if let Some(v) = meta.per_user_variant.get(&key) {
        if v == "slim" {
            return "slim".into();
        }
        if v == "classic" {
            return "classic".into();
        }
    }
    if meta.variant == "slim" {
        "slim".into()
    } else {
        "classic".into()
    }
}

pub fn active_skin_variant() -> String {
    let username = accounts::active_account()
        .map(|a| a.username)
        .unwrap_or_else(|| "Steve".into());
    variant_for_user(&username)
}

/// Ruta global donde se guarda la skin offline del usuario.
pub fn global_skin_path() -> PathBuf {
    paths::default_minecraft_dir().join(OFFLINE_SKIN_FILE)
}

pub fn global_cape_path() -> PathBuf {
    paths::default_minecraft_dir().join(OFFLINE_CAPE_FILE)
}

/// Cache de cara 2D (avatar launcher) recortada de la skin global.
pub fn global_face_path() -> PathBuf {
    paths::default_minecraft_dir().join(FACE_CACHE_FILE)
}

pub fn has_global_skin() -> bool {
    global_skin_path().is_file()
}

pub fn has_skin_for_user(username: &str) -> bool {
    effective_skin_path_for(username).is_some()
}

/// Copia la skin al almacén global (+ por nick) y regenera el recorte de cara.
pub fn store_global_skin(src: &Path) -> AppResult<PathBuf> {
    store_global_skin_with_variant(src, "", "classic")
}

pub fn store_global_skin_with_variant(
    src: &Path,
    username: &str,
    variant: &str,
) -> AppResult<PathBuf> {
    if !src.is_file() {
        return Err(AppError::msg("Archivo de skin no encontrado"));
    }
    let norm_variant = if variant == "slim" { "slim" } else { "classic" };
    let dest = global_skin_path();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, &dest)?;

    if let Some(user_dest) = user_skin_path(username) {
        if let Some(parent) = user_dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::copy(src, &user_dest);
    }

    let mut meta = read_skin_meta();
    meta.variant = norm_variant.to_string();
    if !username.trim().is_empty() {
        meta.per_user_variant
            .insert(username.trim().to_ascii_lowercase(), norm_variant.to_string());
    }
    save_skin_meta(&meta);

    refresh_face_cache();
    Ok(dest)
}

/// Recorta la cara y la escribe en `global_face_path`. Fallos silenciosos.
pub fn refresh_face_cache() {
    let active_user = accounts::active_account()
        .map(|a| a.username)
        .unwrap_or_default();
    let skin = effective_skin_path_for(&active_user).unwrap_or_else(global_skin_path);
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

/// Elimina steve/alex inyectados por versiones viejas del launcher en BrandPack.
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

/// Escribe solo la skin del usuario activo (por nick) y actualiza el modelo en CSL.
pub fn write_local_skin(game_dir: &Path, username: &str, skin_path: &Path) -> AppResult<()> {
    let user = username.trim();
    if user.is_empty() || !skin_path.is_file() {
        return Ok(());
    }
    let variant = variant_for_user(user);
    crate::core::skins::csl::write_csl_config_with_variant(game_dir, &variant);
    let dest = local_skin_path(game_dir, user);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(skin_path, &dest)?;
    let capes = game_dir.join("CustomSkinLoader/LocalSkin/capes");
    let _ = std::fs::create_dir_all(&capes);
    Ok(())
}

pub fn write_local_cape(game_dir: &Path, username: &str, cape_path: &Path) -> AppResult<()> {
    let user = username.trim();
    if user.is_empty() || !cape_path.is_file() {
        return Ok(());
    }
    crate::core::skins::csl::write_ely_config(game_dir);
    let dest = game_dir
        .join("CustomSkinLoader")
        .join("LocalSkin")
        .join("capes")
        .join(format!("{user}.png"));
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(cape_path, &dest)?;
    Ok(())
}

/// En Vanilla puro offline genera `resourcepacks/Paraguacraft-MiSkin.zip` y lo activa.
/// En instancias con mods (donde CustomSkinLoader maneja tu nick sin tocar otros players) o
/// con cuenta oficial, desactiva y elimina el zip si existía.
pub fn sync_vanilla_offline_skin_pack(
    game_dir: &Path,
    mc_version: &str,
    loader: &str,
    is_offline_auth: bool,
    skin_path: Option<&Path>,
) {
    let rp_dir = game_dir.join("resourcepacks");
    let zip_path = rp_dir.join(VANILLA_SKIN_PACK_ZIP);
    let norm_loader = loaders::normalize(loader);

    let should_have_pack = norm_loader == "vanilla"
        && is_offline_auth
        && skin_path.map(|p| p.is_file()).unwrap_or(false);

    if !should_have_pack {
        if zip_path.is_file() {
            let _ = instances::resource_packs::set_enabled(
                game_dir,
                mc_version,
                VANILLA_SKIN_PACK_ZIP,
                false,
                false,
            );
            let _ = std::fs::remove_file(&zip_path);
        }
        return;
    }

    let Some(skin) = skin_path else {
        return;
    };
    let Ok(skin_bytes) = std::fs::read(skin) else {
        return;
    };
    if std::fs::create_dir_all(&rp_dir).is_err() {
        return;
    }

    if build_vanilla_skin_zip(&zip_path, mc_version, &skin_bytes).is_ok() {
        let _ = instances::resource_packs::set_enabled(
            game_dir,
            mc_version,
            VANILLA_SKIN_PACK_ZIP,
            false,
            true,
        );
    }
}

fn build_vanilla_skin_zip(dest: &Path, mc_version: &str, skin_bytes: &[u8]) -> AppResult<()> {
    let file = std::fs::File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mcmeta = crate::core::branding::pack_mcmeta_json(mc_version);
    zip.start_file("pack.mcmeta", opts)
        .map_err(|e| AppError::msg(format!("zip mcmeta: {e}")))?;
    zip.write_all(mcmeta.as_bytes())?;

    if let Some(face) = crate::core::skins::mojang::helm_png_bytes_from_skin_png(skin_bytes) {
        if zip.start_file("pack.png", opts).is_ok() {
            let _ = zip.write_all(&face);
        }
    }

    // Legacy 1.6–1.19.2
    for legacy in [
        "assets/minecraft/textures/entity/steve.png",
        "assets/minecraft/textures/entity/alex.png",
    ] {
        zip.start_file(legacy, opts)
            .map_err(|e| AppError::msg(format!("zip {legacy}: {e}")))?;
        zip.write_all(skin_bytes)?;
    }

    // Modern 1.19.3+ / 1.20+ / 1.21+ / 26.3 (todos los 9 skins offline por hash UUID)
    for name in MODERN_DEFAULT_SKINS {
        for kind in ["wide", "slim"] {
            let entry = format!("assets/minecraft/textures/entity/player/{kind}/{name}.png");
            zip.start_file(&entry, opts)
                .map_err(|e| AppError::msg(format!("zip {entry}: {e}")))?;
            zip.write_all(skin_bytes)?;
        }
    }

    zip.finish()
        .map_err(|e| AppError::msg(format!("zip finish: {e}")))?;
    Ok(())
}

/// Quita del BrandPack las texturas de jugador sobrescritas y reescribe LocalSkin (+ pack Vanilla si aplica).
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
    let active_acc = accounts::active_account();
    let username = active_acc
        .as_ref()
        .map(|a| a.username.clone())
        .unwrap_or_else(|| "Steve".into());
    let is_offline = active_acc.as_ref().map(|a| !a.premium).unwrap_or(true);
    write_local_skin(game_dir, &username, skin_path)?;
    if global_cape_path().is_file() {
        let _ = write_local_cape(game_dir, &username, &global_cape_path());
    }

    // 3) PvP Modern: property solo para el cliente local
    write_modern_skin_property(game_dir, skin_path);

    // 4) Vanilla puro offline: pack dedicado Paraguacraft-MiSkin.zip
    let inst_loader = game_dir
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(instances::read_meta)
        .map(|m| m.loader)
        .unwrap_or_default();
    if !inst_loader.is_empty() {
        sync_vanilla_offline_skin_pack(game_dir, mc_version, &inst_loader, is_offline, Some(skin_path));
    }

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

/// Al cambiar nick offline: copiar PNG SkinsRestorer, almacén por nick y LocalSkin al nick nuevo.
pub fn on_offline_username_changed(old_username: &str, new_username: &str) {
    let old = old_username.trim();
    let new = new_username.trim();
    if old.is_empty() || new.is_empty() || old.eq_ignore_ascii_case(new) {
        return;
    }

    if let (Some(old_p), Some(new_p)) = (user_skin_path(old), user_skin_path(new)) {
        if old_p.is_file() {
            let _ = std::fs::copy(&old_p, &new_p);
        }
    }
    let mut meta = read_skin_meta();
    if let Some(v) = meta.per_user_variant.get(&old.to_ascii_lowercase()).cloned() {
        meta.per_user_variant.insert(new.to_ascii_lowercase(), v);
        save_skin_meta(&meta);
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

    if let Some(skin) = effective_skin_path_for(new) {
        let _ = sync_to_local_servers(new, &skin);
        if let Ok(rd) = std::fs::read_dir(paths::instances_dir()) {
            for entry in rd.flatten() {
                if entry.path().is_dir() {
                    let _ = write_local_skin(&entry.path(), new, &skin);
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

/// Flujo completo: guardar global + por nick, LocalSkin + pack Vanilla + servidores.
pub fn apply_offline_skin(src: &Path, username: &str) -> AppResult<ApplySkinResult> {
    apply_offline_skin_with_variant(src, username, "classic")
}

pub fn apply_offline_skin_with_variant(
    src: &Path,
    username: &str,
    variant: &str,
) -> AppResult<ApplySkinResult> {
    store_global_skin_with_variant(src, username, variant)?;
    let global = effective_skin_path_for(username).unwrap_or_else(global_skin_path);
    let instances = apply_to_all_instances(&global)?;
    let server_sync = sync_to_local_servers(username, &global);
    let model_label = if variant == "slim" { "Delgado/Alex" } else { "Ancho/Steve" };
    let mut msg = if instances > 0 {
        format!(
            "Skin personalizada ({model_label}) aplicada en {instances} instancia(s) sin necesitar cuenta oficial."
        )
    } else {
        format!("Skin personalizada ({model_label}) guardada: se aplicará automáticamente al iniciar el juego.")
    };
    if server_sync > 0 {
        msg.push_str(&format!(" Sincronizada en {server_sync} carpeta(s) de servidor local."));
    }
    Ok(ApplySkinResult {
        ok: true,
        message: msg,
        instances,
        server_sync,
        premium: false,
    })
}

pub fn apply_offline_cape(src: &Path, username: &str) -> AppResult<ApplySkinResult> {
    if !src.is_file() {
        return Err(AppError::msg("Archivo de capa no encontrado"));
    }
    let dest = global_cape_path();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, &dest)?;
    let mut instances = 0u32;
    let root = paths::instances_dir();
    if root.is_dir() {
        for entry in std::fs::read_dir(&root)?.flatten() {
            if entry.path().is_dir() {
                write_local_cape(&entry.path(), username, &dest)?;
                instances += 1;
            }
        }
    }
    Ok(ApplySkinResult {
        ok: true,
        message: if instances > 0 {
            format!("Capa aplicada en {instances} instancia(s). Reentrá al mundo para verla.")
        } else {
            "Capa guardada: se aplica automáticamente al abrir el juego.".into()
        },
        instances,
        server_sync: 0,
        premium: false,
    })
}

/// Antes de lanzar: limpia brand pack, escribe LocalSkin de la cuenta activa y sincroniza pack Vanilla si corresponde.
pub fn ensure_for_launch(game_dir: &Path, mc_version: &str) -> AppResult<()> {
    ensure_for_launch_with_loader(game_dir, mc_version, "")
}

pub fn ensure_for_launch_with_loader(
    game_dir: &Path,
    mc_version: &str,
    loader: &str,
) -> AppResult<()> {
    let pack_path = game_dir.join("resourcepacks").join(branding::PACK_NAME);
    if pack_path.is_dir() {
        purge_default_player_textures(&pack_path);
        let _ = rebuild_brand_pack_zip(game_dir);
    }

    let active_acc = accounts::active_account();
    let username = active_acc
        .as_ref()
        .map(|a| a.username.clone())
        .unwrap_or_else(|| "Steve".into());
    let is_offline = active_acc.as_ref().map(|a| !a.premium).unwrap_or(true);
    let skin_opt = effective_skin_path_for(&username);

    if let Some(ref skin) = skin_opt {
        apply_to_game_dir(game_dir, skin, mc_version)?;
        if !loader.is_empty() {
            sync_vanilla_offline_skin_pack(game_dir, mc_version, loader, is_offline, Some(skin));
        }
    } else {
        if !loader.is_empty() {
            sync_vanilla_offline_skin_pack(game_dir, mc_version, loader, is_offline, None);
        }
        if global_cape_path().is_file() {
            let _ = write_local_cape(game_dir, &username, &global_cape_path());
        }
    }
    Ok(())
}
