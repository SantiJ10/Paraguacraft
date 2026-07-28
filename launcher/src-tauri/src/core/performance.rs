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
            ("renderDistance".into(), "12".into()),
            ("simulationDistance".into(), "10".into()),
            ("particles".into(), "1".into()),
            ("graphicsMode".into(), "FAST".into()),
            ("cloudRenderMode".into(), "OFF".into()),
            ("entityDistanceScaling".into(), "0.75".into()),
            ("entityShadows".into(), "false".into()),
            ("ao".into(), "1".into()),
            ("biomeBlendRadius".into(), "2".into()),
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

/// Fusiona claves en `options.txt` (formato `clave:valor`).
pub fn merge_options_keys(path: &Path, opciones: HashMap<String, String>) -> AppResult<HashMap<String, String>> {
    patch_options_file(path, opciones)
}

/// Optimiza `options.txt` global (`.minecraft/options.txt`) según perfil efectivo.
pub fn optimize_global_options() -> AppResult<OptionsOptimizeResult> {
    let settings: AppSettings =
        crate::config::read_json(&paths::config_file()).unwrap_or_default();
    let tier = resolve_tier(&settings, None);
    let opciones = tier_options(if tier == "custom" { "media" } else { &tier });
    let path = paths::default_minecraft_dir().join("options.txt");
    let applied = patch_options_file(&path, opciones)?;
    Ok(OptionsOptimizeResult {
        tier,
        applied,
        path: path.to_string_lossy().into(),
    })
}

/// Optimiza `options.txt` de una instancia concreta.
pub fn optimize_instance_options(
    game_dir: &Path,
    instance_tier: Option<&str>,
) -> AppResult<OptionsOptimizeResult> {
    let settings: AppSettings =
        crate::config::read_json(&paths::config_file()).unwrap_or_default();
    let tier = resolve_tier(&settings, instance_tier);
    let opciones = tier_options(if tier == "custom" { "media" } else { &tier });
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

/// Options.txt para Paraguacraft Optimized: rendimiento primero, sin saturar la PC.
/// Importante: NO forzar fullscreen exclusivo — rompe captura/Discord y crashea GLFW en algunas GPUs.
fn tier_options_optimized(tier: &str) -> HashMap<String, String> {
    match tier {
        "alta" => HashMap::from([
            ("renderDistance".into(), "10".into()),
            ("simulationDistance".into(), "8".into()),
            ("particles".into(), "1".into()),
            ("graphicsMode".into(), "1".into()),
            ("ao".into(), "true".into()),
            ("biomeBlendRadius".into(), "1".into()),
            ("maxFps".into(), "260".into()),
            ("enableVsync".into(), "false".into()),
            ("entityShadows".into(), "false".into()),
            ("entityDistanceScaling".into(), "0.85".into()),
            ("renderClouds".into(), "fast".into()),
            ("mipmapLevels".into(), "3".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
            ("exclusiveFullscreen".into(), "false".into()),
            ("prioritizeChunkUpdates".into(), "1".into()),
        ]),
        "media" => HashMap::from([
            ("renderDistance".into(), "8".into()),
            ("simulationDistance".into(), "6".into()),
            ("particles".into(), "1".into()),
            ("graphicsMode".into(), "1".into()),
            ("ao".into(), "true".into()),
            ("biomeBlendRadius".into(), "1".into()),
            ("maxFps".into(), "260".into()),
            ("enableVsync".into(), "false".into()),
            ("entityShadows".into(), "false".into()),
            ("entityDistanceScaling".into(), "0.75".into()),
            ("renderClouds".into(), "fast".into()),
            ("mipmapLevels".into(), "2".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
            ("exclusiveFullscreen".into(), "false".into()),
            ("prioritizeChunkUpdates".into(), "1".into()),
        ]),
        _ => HashMap::from([
            ("renderDistance".into(), "5".into()),
            ("simulationDistance".into(), "4".into()),
            ("particles".into(), "2".into()),
            ("graphicsMode".into(), "0".into()),
            ("ao".into(), "false".into()),
            ("biomeBlendRadius".into(), "0".into()),
            ("maxFps".into(), "260".into()),
            ("enableVsync".into(), "false".into()),
            ("entityShadows".into(), "false".into()),
            ("entityDistanceScaling".into(), "0.5".into()),
            ("renderClouds".into(), "false".into()),
            ("mipmapLevels".into(), "1".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
            ("exclusiveFullscreen".into(), "false".into()),
            ("prioritizeChunkUpdates".into(), "0".into()),
        ]),
    }
}

/// Claves de `options.txt` para OptiFine 1.8.9 / 1.12.2 (no usan graphicsMode).
fn tier_options_optimized_legacy(tier: &str) -> HashMap<String, String> {
    match tier {
        "alta" => HashMap::from([
            ("renderDistance".into(), "10".into()),
            ("fancyGraphics".into(), "true".into()),
            ("ao".into(), "2".into()),
            ("particles".into(), "1".into()),
            ("maxFps".into(), "260".into()),
            ("enableVsync".into(), "false".into()),
            ("entityShadows".into(), "true".into()),
            ("clouds".into(), "true".into()),
            ("mipmapLevels".into(), "4".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
            ("gamma".into(), "1.0".into()),
            ("useVbo".into(), "true".into()),
            ("viewBobbing".into(), "true".into()),
        ]),
        "media" => HashMap::from([
            ("renderDistance".into(), "8".into()),
            ("fancyGraphics".into(), "false".into()),
            ("ao".into(), "1".into()),
            ("particles".into(), "1".into()),
            ("maxFps".into(), "260".into()),
            ("enableVsync".into(), "false".into()),
            ("entityShadows".into(), "false".into()),
            ("clouds".into(), "true".into()),
            ("mipmapLevels".into(), "2".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
            ("gamma".into(), "1.0".into()),
            ("useVbo".into(), "true".into()),
            ("viewBobbing".into(), "true".into()),
        ]),
        _ => HashMap::from([
            // Gama baja: Fast + poca distancia (evita Fancy por defecto de OptiFine).
            ("renderDistance".into(), "5".into()),
            ("fancyGraphics".into(), "false".into()),
            ("ao".into(), "0".into()),
            ("particles".into(), "2".into()),
            ("maxFps".into(), "120".into()),
            ("enableVsync".into(), "false".into()),
            ("entityShadows".into(), "false".into()),
            ("clouds".into(), "false".into()),
            ("mipmapLevels".into(), "0".into()),
            ("fboEnable".into(), "true".into()),
            ("fullscreen".into(), "false".into()),
            ("gamma".into(), "1.0".into()),
            ("useVbo".into(), "true".into()),
            ("viewBobbing".into(), "false".into()),
        ]),
    }
}

fn is_legacy_optifine_mc(mc: &str) -> bool {
    matches!(mc, "1.8.9" | "1.12.2")
}

/// Optimiza options.txt de una instancia Paraguacraft Optimized según gama (+ MC).
pub fn optimize_optimized_options(
    game_dir: &Path,
    tier: &str,
    mc: &str,
) -> AppResult<OptionsOptimizeResult> {
    let opciones = if is_legacy_optifine_mc(mc) {
        tier_options_optimized_legacy(tier)
    } else {
        tier_options_optimized(tier)
    };
    let path = game_dir.join("options.txt");
    let applied = patch_options_file(&path, opciones)?;
    Ok(OptionsOptimizeResult {
        tier: tier.into(),
        applied,
        path: path.to_string_lossy().into(),
    })
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

const DYNAMIC_FPS_VALID_STATES: &[&str] = &[
    "focused", "hovered", "unfocused", "invisible", "unplugged", "abandoned",
];

fn default_dynamic_fps_json() -> &'static str {
    r#"{
  "states": {
    "invisible": { "frame_rate_target": 5 },
    "unfocused": { "frame_rate_target": 30 }
  }
}
"#
}

/// Dynamic FPS 3.x renombró `minimized` → `invisible`. Repara configs rotas del launcher.
fn repair_dynamic_fps_config(path: &Path) -> AppResult<()> {
    if !path.is_file() {
        std::fs::write(path, default_dynamic_fps_json())?;
        return Ok(());
    }

    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => {
            let _ = std::fs::remove_file(path);
            std::fs::write(path, default_dynamic_fps_json())?;
            return Ok(());
        }
    };

    let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&text) else {
        let _ = std::fs::remove_file(path);
        std::fs::write(path, default_dynamic_fps_json())?;
        return Ok(());
    };

    let Some(states) = root.get_mut("states").and_then(|s| s.as_object_mut()) else {
        std::fs::write(path, default_dynamic_fps_json())?;
        return Ok(());
    };

    let mut changed = false;
    for legacy in ["minimized", "MINIMIZED"] {
        if let Some(value) = states.remove(legacy) {
            if !states.contains_key("invisible") {
                states.insert("invisible".into(), value);
            }
            changed = true;
        }
    }

    let invalid: Vec<String> = states
        .keys()
        .filter(|k| !DYNAMIC_FPS_VALID_STATES.contains(&k.as_str()))
        .cloned()
        .collect();
    for key in invalid {
        states.remove(&key);
        changed = true;
    }

    if changed {
        std::fs::write(path, format!("{}\n", serde_json::to_string_pretty(&root)?))?;
    }

    Ok(())
}

fn default_gammautils_json() -> &'static str {
    r#"{
  "gamma": {
    "defaultGamma": 100,
    "toggledGamma": 1500,
    "updateToggle": false,
    "gammaStep": 10,
    "showStatusEffect": false,
    "resetOnClose": false,
    "transition": {
      "smoothTransition": false,
      "transitionSpeed": 3000
    },
    "dynamic": {
      "enabled": false,
      "minGamma": 100,
      "maxGamma": 1000,
      "transitionSpeed": 200,
      "averagingLightRange": 8,
      "skyBrightnessOverride": 0
    },
    "dimensionPreference": {
      "enabled": false,
      "overworldPreference": 1500,
      "netherPreference": 1500,
      "endPreference": 1500
    },
    "limiter": {
      "limitCheck": true,
      "minGamma": -750,
      "maxGamma": 1500
    },
    "hudMessage": {
      "showMessage": true,
      "defaultColor": 43520,
      "positiveColor": 16755200,
      "negativeColor": 11141120
    }
  },
  "nightVision": {
    "toggledNightVision": 100,
    "updateToggle": false,
    "nightVisionStep": 2,
    "brightenFogColor": true,
    "showStatusEffect": false,
    "resetOnClose": false,
    "transition": {
      "smoothTransition": false,
      "transitionSpeed": 200
    },
    "dynamic": {
      "enabled": false,
      "minNightVision": 0,
      "maxNightVision": 100,
      "transitionSpeed": 15,
      "averagingLightRange": 8,
      "skyBrightnessOverride": 0
    },
    "dimensionPreference": {
      "enabled": false,
      "overworldPreference": 100,
      "netherPreference": 100,
      "endPreference": 100
    },
    "limiter": {
      "limitCheck": true,
      "minNightVision": 0,
      "maxNightVision": 100
    },
    "hudMessage": {
      "showMessage": true,
      "defaultColor": 43520,
      "positiveColor": 16755200,
      "negativeColor": 11141120,
      "enabledColor": 43520,
      "disabledColor": 11141120
    }
  },
  "other": {
    "namespacedCommands": false
  }
}
"#
}

fn seed_gammautils_config(config: &Path) -> AppResult<()> {
    let path = config.join("gammautils.json");
    if path.is_file() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            let _ = std::fs::remove_file(&path);
            std::fs::write(&path, default_gammautils_json())?;
            return Ok(());
        };
        if text.trim().is_empty() || serde_json::from_str::<serde_json::Value>(&text).is_err() {
            let _ = std::fs::remove_file(&path);
            std::fs::write(&path, default_gammautils_json())?;
        }
        return Ok(());
    }
    std::fs::write(&path, default_gammautils_json())?;
    Ok(())
}

/// Configs de mods para Paraguacraft Optimized (More Culling, Sodium Extra, BRD, BBE, etc.).
pub fn apply_optimized_mod_configs(game_dir: &Path, tier: &str) -> AppResult<()> {
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
    let _ = repair_dynamic_fps_config(&config.join("dynamic_fps.json"));

    write_moreculling_toml(&config, tier)?;
    write_sodium_options_json(&config)?;
    write_sodium_extra_options_json(&config, tier)?;
    write_entityculling_json(&config, tier)?;
    write_immediatelyfast_json(&config)?;
    write_bbe_configs(&config, tier)?;
    write_better_render_distance_json(&config, tier)?;
    write_renderscale_json5(&config, tier)?;
    write_particle_core_toml(&config, tier)?;
    write_badoptimizations_txt(&config)?;

    Ok(())
}

fn write_text(path: &Path, contents: &str) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    Ok(())
}

fn write_moreculling_toml(config: &Path, tier: &str) -> AppResult<()> {
    let (leaves_mode, lod_range, item_frame_3face) = match tier {
        "alta" => ("DEFAULT", 16, 2.0),
        "media" => ("GAP", 12, 2.0),
        _ => ("GAP", 8, 3.0),
    };
    let body = format!(
        r#"version = 1
enableSodiumMenu = true
dontCull = []
cloudCulling = true
signTextCulling = true
rainCulling = true
useBlockStateCulling = true
useCustomItemFrameRenderer = true
itemFrameMapCulling = true
useItemFrameLOD = true
itemFrameLODRange = {lod_range}
useItemFrame3FaceCulling = true
itemFrame3FaceCullingRange = {item_frame_3face}
paintingCulling = true
leavesCullingMode = "{leaves_mode}"
leavesCullingAmount = 2
includeMangroveRoots = false
endGatewayCulling = false
beaconBeamCulling = true
useOnModdedBlocksByDefault = true

[modCompatibility]
minecraft = true
"#
    );
    write_text(&config.join("moreculling.toml"), &body)
}

fn write_sodium_options_json(config: &Path) -> AppResult<()> {
    // Esquema compatible con Sodium 0.6+/0.9 (Keo).
    // `use_no_error_g_l_context: true` crashea el arranque en varias GPUs/drivers Windows.
    let path = config.join("sodium-options.json");
    let body = r#"{
  "quality": {
    "hidden_fluid_culling": true,
    "improved_fluid_shaping": false,
    "use_closest_point_entity_sort": false,
    "pixel_filtering_mode": "NEAREST"
  },
  "performance": {
    "chunk_builder_threads": 0,
    "chunk_build_defer_mode": "ALWAYS",
    "animate_only_visible_textures": true,
    "use_entity_culling": true,
    "use_fog_occlusion": true,
    "use_block_face_culling": true,
    "use_no_error_g_l_context": false,
    "quad_splitting_mode": "SAFE"
  },
  "advanced": {
    "enable_memory_tracing": false,
    "use_advanced_staging_buffers": true,
    "cpu_render_ahead_limit": 3
  },
  "notifications": {
    "has_cleared_donation_button": false,
    "has_seen_donation_prompt": true
  },
  "debug": {
    "terrain_sorting_enabled": true
  }
}
"#;
    if path.is_file() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(perf) = root.get_mut("performance").and_then(|p| p.as_object_mut()) {
                    if perf.get("use_no_error_g_l_context").and_then(|v| v.as_bool())
                        == Some(true)
                    {
                        perf.insert(
                            "use_no_error_g_l_context".into(),
                            serde_json::Value::Bool(false),
                        );
                        std::fs::write(
                            &path,
                            format!("{}\n", serde_json::to_string_pretty(&root)?),
                        )?;
                    }
                }
                return Ok(());
            }
        }
        let _ = std::fs::remove_file(&path);
    }
    write_text(&path, body)
}

fn write_sodium_extra_options_json(config: &Path, tier: &str) -> AppResult<()> {
    let (stars, rain, toast, particles) = match tier {
        "alta" => (true, true, true, true),
        "media" => (true, true, true, true),
        _ => (false, false, false, true),
    };
    let body = format!(
        r#"{{
  "animation_settings": {{
    "animation": true,
    "water": true,
    "lava": true,
    "fire": true,
    "portal": true,
    "block_animations": true,
    "sculk_sensor": true
  }},
  "particle_settings": {{
    "particles": {particles},
    "rain_splash": {rain},
    "block_break": true,
    "block_breaking": true
  }},
  "detail_settings": {{
    "sky": true,
    "sun": true,
    "moon": true,
    "stars": {stars},
    "rain_snow": {rain},
    "biome_colors": true,
    "sky_colors": true
  }},
  "render_settings": {{
    "light_updates": true,
    "item_frame": true,
    "armor_stand": true,
    "painting": true,
    "piston": true,
    "beacon_beam": true,
    "limit_beacon_beam_height": false,
    "enchanting_table_book": true,
    "item_frame_name_tag": true,
    "player_name_tag": true
  }},
  "extra_settings": {{
    "overlay_corner": "TOP_LEFT",
    "text_contrast": "NONE",
    "show_fps": true,
    "show_f_p_s_extended": false,
    "show_coords": false,
    "reduce_resolution_on_mac": false,
    "use_adaptive_sync": false,
    "cloud_height": 192,
    "toasts": {toast},
    "advancement_toast": {toast},
    "recipe_toast": {toast},
    "system_toast": {toast},
    "tutorial_toast": false,
    "instant_sneak": false,
    "prevent_shaders": false,
    "steady_debug_hud": true,
    "steady_debug_hud_refresh_interval": 1
  }}
}}
"#
    );
    write_text(&config.join("sodium-extra-options.json"), &body)
}

fn write_entityculling_json(config: &Path, tier: &str) -> AppResult<()> {
    let (tracing, hitbox, capture, sleep) = match tier {
        "alta" => (128, 50, 5, 10),
        "media" => (96, 40, 5, 10),
        _ => (64, 30, 4, 15),
    };
    let body = format!(
        r#"{{
  "configVersion": 8,
  "renderNametagsThroughWalls": true,
  "blockEntityWhitelist": ["minecraft:beacon"],
  "entityWhitelist": [],
  "tracingDistance": {tracing},
  "debugMode": false,
  "sleepDelay": {sleep},
  "hitboxLimit": {hitbox},
  "captureRate": {capture},
  "tickCulling": true,
  "tickCullingWhitelist": [
    "minecraft:boat",
    "minecraft:chest_boat",
    "minecraft:firework_rocket",
    "minecraft:item_display",
    "minecraft:text_display",
    "minecraft:block_display"
  ],
  "disableF3": false,
  "skipEntityCulling": false,
  "skipBlockEntityCulling": false,
  "blockEntityFrustumCulling": true,
  "forceDisplayCulling": false,
  "solidLeaves": false
}}
"#
    );
    write_text(&config.join("entityculling.json"), &body)
}

fn write_immediatelyfast_json(config: &Path) -> AppResult<()> {
    let body = r#"{
  "enhanced_batching": true,
  "font_atlas_resizing": true,
  "font_atlas_size": 1024,
  "map_atlas_generation": true,
  "map_atlas_size": 2048,
  "skip_text_translucency_sorting": true,
  "fast_text_lookup": true,
  "avoid_redundant_framebuffer_switching": true,
  "fix_slow_buffer_upload_on_apple_gpu": true,
  "experimental_disable_resource_pack_conflict_handling": false,
  "experimental_sign_text_buffering": false,
  "experimental_sign_atlas_size": 4096,
  "debug_only_and_not_recommended_disable_mod_conflict_handling": false,
  "debug_only_and_not_recommended_disable_hardware_conflict_handling": false,
  "debug_only_print_additional_error_information": false
}
"#;
    write_text(&config.join("immediatelyfast.json"), body)
}

fn write_bbe_configs(config: &Path, tier: &str) -> AppResult<()> {
    let sign_dist = match tier {
        "alta" => 24,
        "media" => 16,
        _ => 10,
    };
    let anim = tier != "baja";
    let bbe = format!(
        r#"{{
  "bbe.config.storage.main": [
    {{ "option": "optimize.master", "value": true }},
    {{ "option": "optimize.chest", "value": true }},
    {{ "option": "optimize.shulker", "value": true }},
    {{ "option": "optimize.sign", "value": true }},
    {{ "option": "optimize.decoratedpot", "value": true }},
    {{ "option": "optimize.banner", "value": true }},
    {{ "option": "optimize.bell", "value": true }},
    {{ "option": "optimize.bed", "value": true }},
    {{ "option": "optimize.shelf", "value": true }},
    {{ "option": "optimize.campfire", "value": true }},
    {{ "option": "animation.chest", "value": {anim} }},
    {{ "option": "animation.shulker", "value": {anim} }},
    {{ "option": "animation.bell", "value": {anim} }},
    {{ "option": "animation.decoratedpot", "value": {anim} }},
    {{ "option": "misc.banner_graphics", "value": 1 }},
    {{ "option": "misc.christmas_chest", "value": false }},
    {{ "option": "misc.sign_text_distance", "value": {sign_dist} }},
    {{ "option": "misc.sign_text", "value": true }},
    {{ "option": "misc.sign_text_culling", "value": true }},
    {{ "option": "misc.update_scheduler", "value": 0 }}
  ],
  "bbe.config.storage.experimental": []
}}
"#
    );
    write_text(&config.join("BBEConfig.json"), &bbe)?;

    let legacy = format!(
        r#"{{
  "master_optimize": true,
  "optimize_chests": true,
  "optimize_signs": true,
  "optimize_shulkers": true,
  "optimize_beds": true,
  "optimize_bells": true,
  "optimize_decoratedpots": true,
  "chest_animations": {anim},
  "render_sign_text": true,
  "shulker_animations": {anim},
  "bell_animations": {anim},
  "pot_animations": {anim},
  "sign_text_render_distance": {sign_dist},
  "updateType": 0,
  "smoothness_slider": 25
}}
"#
    );
    write_text(&config.join("betterblockentities.json"), &legacy)
}

fn write_better_render_distance_json(config: &Path, tier: &str) -> AppResult<()> {
    // Ayuda a no saturar CPU/GPU en distancias altas; útil con o sin shaders.
    let (enabled, scale, preset) = match tier {
        "alta" => (true, 0.65, "BALANCED"),
        "media" => (true, 0.5, "BALANCED"),
        _ => (true, 0.35, "PERFORMANCE"),
    };
    let body = format!(
        r#"{{
  "enabled": {enabled},
  "verticalScale": {scale},
  "verticalScaleManual": {scale},
  "verticalScaleAuto": true,
  "verticalScalePreset": "{preset}",
  "cornerShrinkHorizontal": 0.25
}}
"#
    );
    write_text(&config.join("betterrenderdistance.json"), &body)
}

fn write_renderscale_json5(config: &Path, tier: &str) -> AppResult<()> {
    let scale = match tier {
        "alta" => "1.0",
        "media" => "0.9",
        _ => "0.75",
    };
    let body = format!(
        r#"{{
	"scale": {scale},
	"forceLinear": false,
	"irisScale": -1.0
}}
"#
    );
    write_text(&config.join("renderscale.json5"), &body)
}

fn write_particle_core_toml(config: &Path, tier: &str) -> AppResult<()> {
    let (culling, reduce_all, reduce_dec) = match tier {
        "alta" => ("VANILLA", "0.0", "0.0"),
        "media" => ("AGGRESSIVE", "0.0", "0.15"),
        _ => ("AGGRESSIVE", "0.25", "0.4"),
    };
    let body = format!(
        r#"# Don't change this! Version used to track needed updates.
version = 1
disableParticles = false
byTypeReductions = {{  }}
maxParticlesPerSheet = 16384
particleRenderDistanceMultiplier = 1.0
asynchronousTicking = true
cullingBlacklist = [  ]
cullingBehavior = "{culling}"
reduceAllChance = {reduce_all}
reduceDecreasedChance = {reduce_dec}
turnOffPotionParticlesV2 = [
    "NONE"
]
"#
    );
    write_text(&config.join("particle_core_config.toml"), &body)
}

fn write_badoptimizations_txt(config: &Path) -> AppResult<()> {
    let body = r#"# BadOptimizations configuration (Paraguacraft Optimized)
enable_lightmap_caching: true
lightmap_time_change_needed_for_update: 80
enable_sky_color_caching: true
skycolor_time_change_needed_for_update: 3
enable_debug_renderer_disable_if_not_needed: true
enable_particle_manager_optimization: true
enable_toast_optimizations: true
enable_sky_angle_caching_in_worldrenderer: true
enable_entity_renderer_caching: true
enable_block_entity_renderer_caching: true
enable_entity_flag_caching: true
enable_remove_redundant_fov_calculations: true
enable_remove_tutorial_if_not_demo: true
show_f3_text: true
ignore_mod_incompatibilities: false
ignore_mod_cache_hooks: false
log_config: false
config_version: 6
"#;
    write_text(&config.join("badoptimizations.txt"), body)
}

/// Ajusta configs de Sodium, Lithium, Dynamic FPS y Gamma Utils según tier PvP modern.
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

    repair_dynamic_fps_config(&config.join("dynamic_fps.json"))?;
    seed_gammautils_config(&config)?;

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

/// Resuelve el perfil de rendimiento efectivo (baja/media/alta/custom).
/// Prioridad: override de instancia → ajuste global → auto (hardware + preset de uso).
pub fn resolve_tier(settings: &AppSettings, instance_tier: Option<&str>) -> String {
    let inst = instance_tier
        .map(str::trim)
        .filter(|t| !t.is_empty() && *t != "auto");
    if let Some(t) = inst {
        return t.to_string();
    }
    match settings.performance_tier.as_str() {
        "baja" | "media" | "alta" | "custom" => settings.performance_tier.clone(),
        _ => {
            let hw = hardware::detect().perfil_sugerido;
            match settings.usage_preset.as_str() {
                "pvp" | "lightweight" => "baja".into(),
                "shaders" => {
                    if hw == "baja" {
                        "media".into()
                    } else {
                        "alta".into()
                    }
                }
                "gameplay" => {
                    if hw == "baja" {
                        "media".into()
                    } else {
                        hw
                    }
                }
                _ => hw,
            }
        }
    }
}

/// Aplica RAM + GC recomendados según hardware detectado.
pub fn apply_hardware_defaults(settings: &mut AppSettings) -> HardwareInfo {
    let hw = hardware::detect();
    settings.ram_mb = hw.recommended_ram_mb;
    settings.gc_type = hw.recommended_gc.clone();
    settings.hardware_defaults_applied = true;
    settings.performance_tier = "auto".into();
    hw
}
