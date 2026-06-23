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

/// Aplica RAM + GC recomendados según hardware detectado.
pub fn apply_hardware_defaults(settings: &mut AppSettings) -> HardwareInfo {
    let hw = hardware::detect();
    settings.ram_mb = hw.recommended_ram_mb;
    settings.gc_type = hw.recommended_gc.clone();
    settings.hardware_defaults_applied = true;
    hw
}
