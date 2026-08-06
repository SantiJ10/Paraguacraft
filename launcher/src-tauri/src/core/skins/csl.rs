//! CustomSkinLoader + Ely.by para cuentas no-premium.
//!
//! Inyecta el mod en `mods/` cuando el usuario lanza offline, para cargar skins
//! desde Ely.by (visibles en servidores third-party para quien tenga CSL).

use std::path::{Path, PathBuf};

use reqwest::Client;
use tauri::AppHandle;

use crate::core::store::modrinth;
use crate::error::AppResult;

const CSL_PROJECT: &str = "idMHQ4n2"; // CustomSkinLoader en Modrinth

fn csl_loader_tag(loader: &str) -> Option<&'static str> {
    let l = loader.to_ascii_lowercase();
    if l.contains("fabric") || l.contains("pvp-modern") || l.contains("iris") {
        Some("fabric")
    } else if l.contains("neoforge") {
        Some("neoforge")
    } else if l.contains("forge") || l.contains("pvp") || l.contains("optifine") {
        // 1.8–1.12 usan Forge; OptiFine+Forge también.
        Some("forge")
    } else if l.contains("optimized") {
        // Optimized 1.20.1+ = Fabric (salvo legacy OptiFine).
        Some("fabric")
    } else {
        None
    }
}

fn has_csl_jar(mods_dir: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(mods_dir) else {
        return false;
    };
    rd.flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_ascii_lowercase();
        n.ends_with(".jar") && n.contains("customskinloader")
    })
}

/// Escribe preferencias mínimas para priorizar Ely.by (además del loadlist default del mod).
fn write_ely_config(game_dir: &Path) {
    let cfg_dir = game_dir.join("CustomSkinLoader");
    let _ = std::fs::create_dir_all(&cfg_dir);
    let path = cfg_dir.join("CustomSkinLoader.json");
    if path.is_file() {
        return;
    }
    // Formato compatible con CSL moderno: loadlist con Mojang + ElyBy.
    let body = r#"{
  "version": "14.19",
  "buildNumber": 0,
  "loadlist": [
    {
      "name": "Mojang",
      "type": "MojangAPI"
    },
    {
      "name": "ElyBy",
      "type": "ElyByAPI",
      "root": "http://skinsystem.ely.by/"
    },
    {
      "name": "LocalSkin",
      "type": "Legacy",
      "checkPNG": false,
      "skin": "LocalSkin/skins/{USERNAME}.png",
      "cape": "LocalSkin/capes/{USERNAME}.png"
    }
  ]
}
"#;
    let _ = std::fs::write(path, body);
}

/// Instala CustomSkinLoader si la cuenta es offline y el loader lo soporta.
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
    let Some(loader_tag) = csl_loader_tag(loader) else {
        return Ok(());
    };
    // Vanilla pure: no hay carpeta mods.
    let mods_dir = game_dir.join("mods");
    if !mods_dir.is_dir() {
        // Si hay profile de Forge/Fabric el launcher ya creó mods/; si no, no forzar.
        let has_versions = game_dir.join("versions").is_dir();
        if !has_versions {
            return Ok(());
        }
        let _ = std::fs::create_dir_all(&mods_dir);
    }

    write_ely_config(game_dir);

    if has_csl_jar(&mods_dir) {
        return Ok(());
    }

    match modrinth::install(
        app,
        client,
        CSL_PROJECT,
        "mod",
        mc,
        loader_tag,
        mods_dir.clone(),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("[paraguacraft] CustomSkinLoader (Ely.by): {e}");
            // No bloquear el launch si Modrinth falla.
            Ok(())
        }
    }
}
