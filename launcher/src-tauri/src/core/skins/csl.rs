//! CustomSkinLoader + LocalSkin + Ely.by para cuentas sin cuenta oficial (offline).
//!
//! Prioridad en `CustomSkinLoader.json`:
//!   1. `LocalSkin` (`LocalSkin/skins/{USERNAME}.png` + modelo `default`/`slim`) → tu skin
//!      personalizada SIEMPRE prevalece en tu nick activo, aunque ese nombre exista en Mojang/Ely.by.
//!   2. `ElyBy` → otros jugadores no-premium registrados en Ely.by en multiplayer.
//!   3. `Mojang` → otros jugadores con cuenta oficial en multiplayer.

use std::path::Path;

use reqwest::Client;
use tauri::AppHandle;

use crate::core::loaders;
use crate::core::store::modrinth;
use crate::error::AppResult;

const CSL_PROJECT: &str = "idMHQ4n2"; // CustomSkinLoader en Modrinth

fn csl_loader_tag(loader: &str, mc: &str) -> Option<&'static str> {
    match loaders::store_loader_for(loader, mc).as_str() {
        "fabric" => Some("fabric"),
        "quilt" => Some("quilt"),
        "neoforge" => Some("neoforge"),
        "forge" | "optifine" => Some("forge"),
        _ => None,
    }
}

fn has_csl_jar(mods_dir: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(mods_dir) else {
        return false;
    };
    rd.flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_ascii_lowercase();
        n.ends_with(".jar") && !n.ends_with(".disabled") && n.contains("customskinloader")
    })
}

/// Escribe `CustomSkinLoader.json` con `LocalSkin` primero → `ElyBy` → `Mojang` y modelo `default`/`slim`.
pub fn write_ely_config(game_dir: &Path) {
    let variant = crate::core::skins::offline::active_skin_variant();
    write_csl_config_with_variant(game_dir, &variant);
}

pub fn write_csl_config_with_variant(game_dir: &Path, variant: &str) {
    let cfg_dir = game_dir.join("CustomSkinLoader");
    let _ = std::fs::create_dir_all(cfg_dir.join("LocalSkin/skins"));
    let _ = std::fs::create_dir_all(cfg_dir.join("LocalSkin/capes"));
    let path = cfg_dir.join("CustomSkinLoader.json");

    let csl_model = if variant == "slim" { "slim" } else { "default" };
    let expected_marker = format!("\"model\": \"{csl_model}\"");

    let need_write = match std::fs::read_to_string(&path) {
        Ok(s) => {
            !s.contains("\"buildNumber\": 2")
                || !s.contains("ElyBy")
                || !s.contains("LocalSkin")
                || !s.contains(&expected_marker)
        }
        Err(_) => true,
    };
    if !need_write {
        return;
    }

    let body = format!(
        r#"{{
  "version": "14.20",
  "buildNumber": 2,
  "loadlist": [
    {{
      "name": "LocalSkin",
      "type": "Legacy",
      "checkPNG": false,
      "model": "{csl_model}",
      "skin": "LocalSkin/skins/{{USERNAME}}.png",
      "cape": "LocalSkin/capes/{{USERNAME}}.png"
    }},
    {{
      "name": "ElyBy",
      "type": "ElyByAPI",
      "root": "http://skinsystem.ely.by/"
    }},
    {{
      "name": "Mojang",
      "type": "MojangAPI"
    }}
  ]
}}
"#
    );
    let _ = std::fs::write(path, body);
}

/// Instala CustomSkinLoader en `mods/` si la cuenta es offline y la instancia admite mods.
pub async fn ensure_for_offline_launch(
    app: &AppHandle,
    client: &Client,
    game_dir: &Path,
    mc: &str,
    loader: &str,
    auth_type: &str,
) -> AppResult<()> {
    let auth = auth_type.trim().to_ascii_lowercase();
    if auth != "legacy" && auth != "offline" {
        return Ok(());
    }
    let Some(loader_tag) = csl_loader_tag(loader, mc) else {
        return Ok(());
    };

    // En instancias Paraguacraft `versions/` es compartido globalmente, así que siempre
    // aseguramos `mods/` cuando el loader soporta mods (Fabric/Quilt/Forge/NeoForge/Optimized/PvP).
    let mods_dir = game_dir.join("mods");
    let _ = std::fs::create_dir_all(&mods_dir);

    write_ely_config(game_dir);

    if has_csl_jar(&mods_dir) {
        return Ok(());
    }

    let res = modrinth::install(
        app,
        client,
        CSL_PROJECT,
        "mod",
        mc,
        loader_tag,
        mods_dir.clone(),
    )
    .await;

    if res.is_err() && loader_tag == "quilt" {
        // Quilt carga mods Fabric nativamente si no hay build específico etiquetado como quilt.
        if let Err(e) = modrinth::install(
            app,
            client,
            CSL_PROJECT,
            "mod",
            mc,
            "fabric",
            mods_dir,
        )
        .await
        {
            eprintln!("[paraguacraft] CustomSkinLoader fallback fabric en quilt: {e}");
        }
        return Ok(());
    }

    if let Err(e) = res {
        eprintln!("[paraguacraft] CustomSkinLoader: {e}");
    }
    Ok(())
}

/// Solo escribe CustomSkinLoader.json (modo sin conexión).
pub fn ensure_local_config(game_dir: &Path, loader: &str, auth_type: &str) {
    let auth = auth_type.trim().to_ascii_lowercase();
    if auth != "legacy" && auth != "offline" {
        return;
    }
    if csl_loader_tag(loader, "").is_none() {
        return;
    }
    write_ely_config(game_dir);
}
