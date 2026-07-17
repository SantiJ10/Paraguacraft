//! Optimización de rendimiento: options.txt y perfiles según hardware.
//!
//! Espejo simplificado de `optimizar_opciones_mc` y `aplicar_rendimiento_recomendado`
//! del launcher Python.

use std::collections::HashMap;
use std::path::Path;

use crate::core::hardware;
use crate::core::paths;
use crate::error::AppResult;
use crate::models::{AppSettings, HardwareInfo};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionsOptimizeResult {
    pub tier: String,
    pub applied: HashMap<String, String>,
    pub path: String,
}

fn tier_options(tier: &str) -> HashMap<String, String> {
    match tier {
        "alta" => HashMap::from([
            ("renderDistance".into(), "14".into()),
            ("simulationDistance".into(), "12".into()),
            ("particles".into(), "0".into()),
            ("fboEnable".into(), "true".into()),
            ("ao".into(), "2".into()),
            ("biomeBlendRadius".into(), "4".into()),
            ("maxFps".into(), "260".into()),
            ("fullscreen".into(), "false".into()),
        ]),
        "media" => HashMap::from([
            ("renderDistance".into(), "10".into()),
            ("simulationDistance".into(), "8".into()),
            ("particles".into(), "1".into()),
            ("fboEnable".into(), "true".into()),
            ("ao".into(), "1".into()),
            ("biomeBlendRadius".into(), "2".into()),
            ("maxFps".into(), "120".into()),
            ("fullscreen".into(), "false".into()),
        ]),
        _ => HashMap::from([
            ("renderDistance".into(), "6".into()),
            ("simulationDistance".into(), "6".into()),
            ("particles".into(), "2".into()),
            ("fboEnable".into(), "false".into()),
            ("ao".into(), "0".into()),
            ("biomeBlendRadius".into(), "0".into()),
            ("maxFps".into(), "60".into()),
            ("fullscreen".into(), "false".into()),
        ]),
    }
}

fn min_graphics_options() -> HashMap<String, String> {
    HashMap::from([
        ("renderDistance".into(), "6".into()),
        ("simulationDistance".into(), "5".into()),
        ("graphicsMode".into(), "FAST".into()),
        ("particles".into(), "2".into()),
        ("entityDistanceScaling".into(), "0.5".into()),
        ("biomeBlendRadius".into(), "0".into()),
        ("maxFps".into(), "60".into()),
        ("enableVsync".into(), "false".into()),
    ])
}

/// Preset PvP 1.21.11 — más agresivo que `tier_options` genérico (Sodium/Iris ya cubren parte del render).
fn tier_options_modern_pvp(tier: &str) -> HashMap<String, String> {
    match tier {
        "alta" => HashMap::from([
            ("renderDistance".into(), "12".into()),
            ("simulationDistance".into(), "10".into()),
            ("particles".into(), "2".into()),
            ("graphicsMode".into(), "FAST".into()),
            ("cloudRenderMode".into(), "OFF".into()),
            ("entityDistanceScaling".into(), "0.75".into()),
            ("entityShadows".into(), "false".into()),
            ("ao".into(), "0".into()),
            ("biomeBlendRadius".into(), "2".into()),
            ("maxFps".into(), "260".into()),
            ("enableVsync".into(), "false".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
        ]),
        "media" => HashMap::from([
            ("renderDistance".into(), "10".into()),
            ("simulationDistance".into(), "8".into()),
            ("particles".into(), "2".into()),
            ("graphicsMode".into(), "FAST".into()),
            ("cloudRenderMode".into(), "OFF".into()),
            ("entityDistanceScaling".into(), "0.65".into()),
            ("entityShadows".into(), "false".into()),
            ("ao".into(), "0".into()),
            ("biomeBlendRadius".into(), "1".into()),
            ("maxFps".into(), "240".into()),
            ("enableVsync".into(), "false".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
        ]),
        _ => HashMap::from([
            ("renderDistance".into(), "8".into()),
            ("simulationDistance".into(), "6".into()),
            ("particles".into(), "2".into()),
            ("graphicsMode".into(), "FAST".into()),
            ("cloudRenderMode".into(), "OFF".into()),
            ("entityDistanceScaling".into(), "0.5".into()),
            ("entityShadows".into(), "false".into()),
            ("ao".into(), "0".into()),
            ("biomeBlendRadius".into(), "0".into()),
            ("maxFps".into(), "120".into()),
            ("enableVsync".into(), "false".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
        ]),
    }
}

fn patch_properties_file(path: &Path, entries: &[(&str, &str)]) -> AppResult<()> {
    let mut lines: Vec<String> = if path.is_file() {
        std::fs::read_to_string(path)?
            .lines()
            .map(String::from)
            .collect()
    } else {
        Vec::new()
    };
    let mut pending: HashMap<String, String> = entries
        .iter()
        .map(|(k, v)| ((*k).into(), (*v).into()))
        .collect();
    for line in lines.iter_mut() {
        if let Some((key, _)) = line.split_once('=') {
            let key = key.trim();
            if let Some(val) = pending.remove(key) {
                *line = format!("{key}={val}");
            }
        }
    }
    for (k, v) in pending {
        lines.push(format!("{k}={v}"));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{}\n", lines.join("\n")))?;
    Ok(())
}

fn patch_options_file(path: &Path, opciones: HashMap<String, String>) -> AppResult<HashMap<String, String>> {
    let mut updated = HashMap::new();
    let lines: Vec<String> = if path.is_file() {
        std::fs::read_to_string(path)?
            .lines()
            .map(String::from)
            .collect()
    } else {
        Vec::new()
    };

    let mut remaining = opciones;
    let mut new_lines = Vec::new();
    for line in lines {
        let key = line.split(':').next().unwrap_or("").trim().to_string();
        if let Some(val) = remaining.remove(&key) {
            new_lines.push(format!("{key}:{val}"));
            updated.insert(key, val);
        } else {
            new_lines.push(line);
        }
    }
    for (k, v) in remaining {
        new_lines.push(format!("{k}:{v}"));
        updated.insert(k, v);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{}\n", new_lines.join("\n")))?;
    Ok(updated)
}

/// Optimiza `options.txt` global (`.minecraft/options.txt`) según hardware.
pub fn optimize_global_options() -> AppResult<OptionsOptimizeResult> {
    let hw = hardware::detect();
    let tier = hw.perfil_sugerido.clone();
    let opciones = tier_options(&tier);
    let path = paths::default_minecraft_dir().join("options.txt");
    let applied = patch_options_file(&path, opciones)?;
    Ok(OptionsOptimizeResult {
        tier,
        applied,
        path: path.to_string_lossy().into(),
    })
}

/// Optimiza `options.txt` de una instancia concreta.
pub fn optimize_instance_options(game_dir: &Path) -> AppResult<OptionsOptimizeResult> {
    let hw = hardware::detect();
    let tier = hw.perfil_sugerido.clone();
    let opciones = tier_options(&tier);
    let path = game_dir.join("options.txt");
    let applied = patch_options_file(&path, opciones)?;
    Ok(OptionsOptimizeResult {
        tier,
        applied,
        path: path.to_string_lossy().into(),
    })
}

/// Aplica preset de gráficos mínimos (toggle «Optimizar gráficos»).
pub fn apply_min_graphics(game_dir: &Path) -> AppResult<()> {
    let path = game_dir.join("options.txt");
    let _ = patch_options_file(&path, min_graphics_options())?;
    Ok(())
}

/// Optimiza `options.txt` de una instancia **Paraguacraft PvP 1.21.11**.
pub fn optimize_modern_pvp_options(
    game_dir: &Path,
    tier: &str,
) -> AppResult<OptionsOptimizeResult> {
    let opciones = tier_options_modern_pvp(tier);
    let path = game_dir.join("options.txt");
    let applied = patch_options_file(&path, opciones)?;
    Ok(OptionsOptimizeResult {
        tier: tier.into(),
        applied,
        path: path.to_string_lossy().into(),
    })
}

/// Ajusta configs de Sodium, Lithium y Dynamic FPS según tier PvP modern.
pub fn apply_modern_pvp_mod_configs(game_dir: &Path, tier: &str) -> AppResult<()> {
    let config = game_dir.join("config");
    std::fs::create_dir_all(&config)?;

    let lithium_entries: &[(&str, &str)] = match tier {
        "alta" => &[
            ("mixin.ai.use_fast_exp_random", "true"),
            ("mixin.ai.poi.use_fast_search", "true"),
            ("mixin.entity.collisions.fluid", "true"),
            ("mixin.util.block_entity_sleep", "true"),
        ],
        "media" => &[
            ("mixin.ai.use_fast_exp_random", "true"),
            ("mixin.ai.poi.use_fast_search", "true"),
        ],
        _ => &[("mixin.ai.use_fast_exp_random", "true")],
    };
    patch_properties_file(&config.join("lithium.properties"), lithium_entries)?;

    let dynamic_fps = config.join("dynamic_fps.json");
    if !dynamic_fps.is_file() {
        let body = r#"{
  "states": {
    "minimized": { "fps": 5 },
    "unfocused": { "fps": 30 }
  }
}"#;
        std::fs::write(&dynamic_fps, body)?;
    }

    // No parchear sodium-options.json: el esquema cambia entre versiones y puede corromper el archivo.
    // Si quedó inválido de un parche anterior, borrarlo para que Sodium regenere defaults.
    let sodium = config.join("sodium-options.json");
    if sodium.is_file() {
        let invalid = std::fs::read_to_string(&sodium)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .is_none();
        if invalid {
            let _ = std::fs::remove_file(&sodium);
        }
    }

    Ok(())
}

/// Aplica RAM + GC recomendados según hardware detectado.
pub fn apply_hardware_defaults(settings: &mut AppSettings) -> HardwareInfo {
    let hw = hardware::detect();
    settings.ram_mb = hw.recommended_ram_mb;
    settings.gc_type = hw.recommended_gc.clone();
    settings.hardware_defaults_applied = true;
    hw
}
