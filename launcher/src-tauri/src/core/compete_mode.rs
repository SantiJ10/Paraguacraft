//! Modo Competir: orquesta optimizaciones pre-lanzamiento sin menús extra.

use std::collections::HashMap;
use std::path::Path;

use crate::core::hardware;
use crate::core::instances::InstanceMeta;
use crate::core::launch;
use crate::core::loaders;
use crate::core::performance;
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct CompeteLaunchPlan {
    pub close_on_launch: bool,
    pub overlay_ipc: bool,
    pub ram_mb: u32,
    pub java_priority: String,
}

/// Aplica Game Mode, gráficos, perfil PvP y RAM recomendada antes de lanzar.
pub fn apply_pre_launch(
    game_dir: &Path,
    loader: &str,
    meta: &InstanceMeta,
    turbo: bool,
) -> AppResult<CompeteLaunchPlan> {
    let _ = crate::core::extras::game_mode::activate();
    if turbo {
        let _ = crate::core::extras::turbo::activate();
    }

    let hw = hardware::detect();
    let ram = if meta.ram_mb > 0 && !meta.auto_managed {
        meta.ram_mb
    } else {
        hw.recommended_ram_mb
    };

    let _ = performance::optimize_instance_options(game_dir);
    let _ = performance::apply_min_graphics(game_dir);

    let mut overlay_ipc = false;
    if loaders::normalize(loader) == "paraguacraft-pvp" {
        overlay_ipc = apply_pvp_compete_profile(game_dir, &hw.perfil_sugerido)?;
    }

    Ok(CompeteLaunchPlan {
        close_on_launch: true,
        overlay_ipc,
        ram_mb: ram,
        java_priority: "high".into(),
    })
}

/// IPC overlay solo si el mod PvP está instalado y algún HUD in-game está activo.
pub fn overlay_ipc_needed(game_dir: &Path) -> bool {
    if !launch::has_paraguacraft_pvp_mod(game_dir) {
        return false;
    }
    let props = read_properties(&game_dir.join("paraguacraft_v2.properties"));
    let music = props
        .get("showMusicHud")
        .is_none_or(|v| v.eq_ignore_ascii_case("true"));
    let hw_hud = props
        .get("showHardwareHud")
        .is_none_or(|v| v.eq_ignore_ascii_case("true"));
    music || hw_hud
}

fn apply_pvp_compete_profile(game_dir: &Path, tier: &str) -> AppResult<bool> {
    let path = game_dir.join("paraguacraft_v2.properties");
    let mut props = read_properties(&path);

    props.insert("boostFps".into(), "true".into());
    props.insert("entityCull".into(), "true".into());
    props.insert("nametagCull".into(), "true".into());
    props.insert("entityAnimCull".into(), "true".into());
    props.insert("blockEntityCull".into(), "true".into());
    props.insert("particleLimit".into(), "true".into());
    props.insert("hardwareAutoPreset".into(), "true".into());
    props.insert("applyVanillaPreset".into(), "true".into());
    props.insert("memoryCleanup".into(), "true".into());
    props.insert("skipCombatFx".into(), "true".into());
    props.insert("armorStandCull".into(), "true".into());
    props.insert("itemFrameCull".into(), "true".into());

    match tier {
        "baja" => {
            props.insert("particleMode".into(), "MINIMAL".into());
            props.insert("showMusicHud".into(), "true".into());
            props.insert("showHardwareHud".into(), "false".into());
            props.insert("showMusicAlbumArt".into(), "false".into());
        }
        "alta" => {
            props.insert("particleMode".into(), "REDUCED".into());
            props.insert("showMusicHud".into(), "true".into());
            props.insert("showHardwareHud".into(), "true".into());
            props.insert("showMusicAlbumArt".into(), "true".into());
        }
        _ => {
            props.insert("particleMode".into(), "REDUCED".into());
            props.insert("showMusicHud".into(), "true".into());
            props.insert("showHardwareHud".into(), "false".into());
            props.insert("showMusicAlbumArt".into(), "false".into());
        }
    }

    write_properties(&path, &props)?;

    let music = props
        .get("showMusicHud")
        .is_some_and(|v| v.eq_ignore_ascii_case("true"));
    let hw_hud = props
        .get("showHardwareHud")
        .is_some_and(|v| v.eq_ignore_ascii_case("true"));
    Ok(music || hw_hud)
}

fn read_properties(path: &Path) -> HashMap<String, String> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            out.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

fn write_properties(path: &Path, props: &HashMap<String, String>) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut keys: Vec<_> = props.keys().collect();
    keys.sort();
    let body: Vec<String> = keys
        .into_iter()
        .map(|k| format!("{k}={}", props[k]))
        .collect();
    std::fs::write(path, format!("{}\n", body.join("\n")))?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceBudget {
    pub launcher_mb: u32,
    pub java_mb: u32,
    pub system_free_mb: u32,
    pub total_ram_mb: u32,
    pub tier: String,
    pub profile_label: String,
}

pub fn resource_budget(java_ram_mb: u32) -> ResourceBudget {
    use sysinfo::{Pid, ProcessesToUpdate, System};

    let hw = hardware::detect();
    let mut sys = System::new();
    sys.refresh_memory();

    let launcher_mb = {
        sys.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(std::process::id())]), true);
        sys.process(Pid::from_u32(std::process::id()))
            .map(|p| (p.memory() / 1024 / 1024) as u32)
            .unwrap_or(80)
    };

    let system_free_mb = (sys.available_memory() / 1024 / 1024) as u32;
    let total_ram_mb = (sys.total_memory() / 1024 / 1024) as u32;

    let profile_label = match hw.perfil_sugerido.as_str() {
        "baja" => "Gama baja",
        "alta" => "Gama alta",
        _ => "Gama media",
    }
    .into();

    ResourceBudget {
        launcher_mb,
        java_mb: java_ram_mb,
        system_free_mb,
        total_ram_mb,
        tier: hw.perfil_sugerido,
        profile_label,
    }
}
