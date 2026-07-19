//! Perfil **Paraguacraft PvP 1.21.11**: loader dedicado `paraguacraft-pvp-modern`.
//!
//! Distinto de `fabric-iris` (solo optimización genérica). Incluye mod Paraguacraft + HUD por tier.
//! Fase 4: JVM Java 21, options.txt PvP, configs Sodium/Lithium y preset de rendimiento en el mod.

use std::collections::HashMap;
use std::path::Path;

use tauri::AppHandle;

use crate::core::hardware;
use crate::core::instances::{self, profiles};
use crate::core::launch::modern_pvp_jvm;
use crate::core::loaders::{self, pvp_modern};
use crate::core::modern_packs;
use crate::core::modern_servers;
use crate::core::performance;
use crate::core::store::modrinth;
use crate::error::{AppError, AppResult};
use crate::models::Instance;

pub const MC: &str = pvp_modern::MC;
pub const LOADER: &str = "paraguacraft-pvp-modern";
pub const DISPLAY_NAME: &str = "Paraguacraft PvP 1.21.11";

/// Mods extra (HUD/QoL) vía Modrinth — además del bundle Iris y del mod Paraguacraft.
struct HudMod {
    slug: &'static str,
    /// 0=baja, 1=media, 2=alta
    min_tier: u8,
}

const HUD_MODS: &[HudMod] = &[
    // Dependencias de Controlling / Zoomify (instalar antes que los mods que las usan).
    HudMod {
        slug: "searchables",
        min_tier: 0,
    },
    HudMod {
        slug: "fabric-language-kotlin",
        min_tier: 0,
    },
    HudMod {
        slug: "yacl",
        min_tier: 0,
    },
    HudMod {
        slug: "modmenu",
        min_tier: 0,
    },
    HudMod {
        slug: "appleskin",
        min_tier: 0,
    },
    HudMod {
        slug: "controlling",
        min_tier: 0,
    },
    HudMod {
        slug: "smooth-scrolling",
        min_tier: 0,
    },
    HudMod {
        slug: "zoomify",
        min_tier: 0,
    },
    HudMod {
        slug: "better-ping-display-fabric",
        min_tier: 1,
    },
    HudMod {
        slug: "shulkerboxtooltip",
        min_tier: 1,
    },
    HudMod {
        slug: "dynamic-fps",
        min_tier: 2,
    },
];

const TIER_BAJA_OFF: &[&str] = &[
    "lithium",
    "ferrite",
    "entityculling",
    "immediatelyfast",
];

const TIER_MEDIA_OFF: &[&str] = &["entityculling"];

pub fn pack_summary(tier: &str) -> String {
    match tier {
        "baja" => "Cliente PvP + Iris básico · Java 21 · PCs 4–8 GB".into(),
        "media" => "Cliente PvP + optimización media · G1 · PCs 8–16 GB".into(),
        _ => "Cliente PvP completo + ZGC · HUD · PCs 16+ GB".into(),
    }
}

pub fn tier_from_hardware() -> String {
    hardware::detect().perfil_sugerido
}

fn tier_level(tier: &str) -> u8 {
    match tier {
        "baja" => 0,
        "media" => 1,
        _ => 2,
    }
}

pub fn is_pvp_modern_instance(instance_id: &str) -> bool {
    instances::read_meta(instance_id)
        .or_else(|| instances::resolve_meta(instance_id))
        .map(|m| m.mc_version == MC && loaders::normalize(&m.loader) == LOADER)
        .unwrap_or(false)
}

pub fn find_instance(instances: &[Instance]) -> Option<&Instance> {
    let norm = loaders::normalize(LOADER);
    let mut matches: Vec<&Instance> = instances
        .iter()
        .filter(|i| i.mc_version == MC && loaders::normalize(&i.loader) == norm)
        .collect();
    if matches.is_empty() {
        return None;
    }
    matches.sort_by(|a, b| b.total_play_minutes.cmp(&a.total_play_minutes));
    Some(matches[0])
}

pub async fn ensure_instance(
    app: &AppHandle,
    client: &reqwest::Client,
    state: &crate::state::AppState,
) -> AppResult<String> {
    let local = instances::list_local();
    if let Some(inst) = find_instance(&local) {
        let mut meta = instances::ensure_meta(&inst.id)?;
        if meta.version_id.as_ref().is_none_or(|v| {
            crate::core::versions::read_local_json(v).is_none()
        }) {
            let lv = if meta.loader_version.is_empty() {
                loaders::resolve_loader_version(client, MC, LOADER, "")
                    .await
                    .unwrap_or_default()
            } else {
                meta.loader_version.clone()
            };
            let vid = loaders::install_loader(app, client, MC, LOADER, &lv).await?;
            meta.version_id = Some(vid);
            if meta.loader_version.is_empty() {
                meta.loader_version = lv;
            }
            meta.loader = LOADER.into();
            instances::write_meta(&inst.id, &meta)?;
        }
        sync_instance_bundles(app, client, &inst.id).await?;
        let dir = instances::instance_dir(&inst.id);
        let _ = modern_servers::write_default_servers(&dir);
        let _ = modern_packs::sync_instance_packs(app, client, &dir).await;
        return Ok(inst.id.clone());
    }

    let hw = hardware::detect();
    let ram = hw.recommended_ram_mb.max(4096);
    let inst = profiles::create(DISPLAY_NAME, MC, LOADER, "", "loader:paraguacraft-pvp-modern", ram)?;
    let lv = loaders::resolve_loader_version(client, MC, LOADER, "")
        .await
        .unwrap_or_default();
    let vid = loaders::install_loader(app, client, MC, LOADER, &lv).await?;
    let mut meta = instances::ensure_meta(&inst.id)?;
    meta.version_id = Some(vid);
    meta.loader_version = lv;
    meta.loader = LOADER.into();
    meta.auto_managed = true;
    meta.ram_mb = ram;
    instances::write_meta(&inst.id, &meta)?;
    let _ = crate::core::java::resolve::ensure_installer_java(app, state, MC).await;
    sync_instance_bundles(app, client, &inst.id).await?;
    let dir = instances::instance_dir(&inst.id);
    let _ = modern_servers::write_default_servers(&dir);
    let _ = modern_packs::sync_instance_packs(app, client, &dir).await;
    Ok(inst.id)
}

pub async fn sync_instance_content(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_id: &str,
) -> AppResult<()> {
    let dir = instances::instance_dir(instance_id);
    let _ = modern_servers::write_default_servers(&dir);
    let _ = modern_packs::sync_instance_packs(app, client, &dir).await?;
    Ok(())
}

pub async fn sync_instance_bundles(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_id: &str,
) -> AppResult<()> {
    let dir = instances::instance_dir(instance_id);
    pvp_modern::sync_instance(app, client, MC, &dir).await
}

pub fn apply_hardware_profile(instance_id: &str) -> AppResult<()> {
    let tier = tier_from_hardware();
    apply_launch_properties(instance_id, &tier)?;
    apply_performance_profile(instance_id, &tier)?;
    let dir = instances::instance_dir(instance_id);
    let mods = dir.join("mods");
    if !mods.is_dir() {
        return Ok(());
    }
    let off_list: &[&str] = match tier.as_str() {
        "baja" => TIER_BAJA_OFF,
        "media" => TIER_MEDIA_OFF,
        _ => &[],
    };
    for entry in std::fs::read_dir(&mods)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jar") {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        if name.contains("paraguacraftpvp-modern") {
            continue;
        }
        let should_disable = off_list.iter().any(|k| name.contains(k));
        set_jar_enabled(&path, !should_disable)?;
    }
    Ok(())
}

/// options.txt PvP + configs de mods + RAM JVM (superior a 1.8.9).
pub fn apply_performance_profile(instance_id: &str, tier: &str) -> AppResult<()> {
    let dir = instances::instance_dir(instance_id);
    let _ = performance::optimize_modern_pvp_options(&dir, tier)?;
    let _ = performance::apply_modern_pvp_mod_configs(&dir, tier)?;

    let hw = hardware::detect();
    let ram_mb = modern_pvp_jvm::resolve_ram_mb(hw.ram_gb);
    let mut meta = instances::ensure_meta(instance_id)?;
    meta.ram_mb = ram_mb;
    instances::write_meta(instance_id, &meta)?;
    Ok(())
}

/// Propiedades que lee el mod Fabric (`paraguacraft_modern.properties`).
pub fn apply_launch_properties(instance_id: &str, tier: &str) -> AppResult<()> {
    merge_launch_properties(instance_id, tier)
}

/// Solo añade claves faltantes; no pisa opciones del usuario.
pub fn merge_launch_properties(instance_id: &str, tier: &str) -> AppResult<()> {
    let path = instances::instance_dir(instance_id).join("paraguacraft_modern.properties");
    let mut props = read_properties(&path);
    for (k, v) in default_launch_props(tier) {
        props.entry(k).or_insert(v);
    }
    write_properties(&path, &props)
}

const PVP_TUNED_MARKER: &str = ".paraguacraft_pvp_tuned";

/// Sync del mod sin resetear options.txt / configs del usuario.
pub fn ensure_launch_defaults(instance_id: &str, tier: &str) -> AppResult<()> {
    merge_launch_properties(instance_id, tier)?;
    let dir = instances::instance_dir(instance_id);
    let marker = dir.join(PVP_TUNED_MARKER);
    if marker.is_file() {
        return Ok(());
    }
    if dir.join("options.txt").is_file() {
        let _ = std::fs::write(&marker, "existing");
        return Ok(());
    }
    let _ = apply_performance_profile(instance_id, tier);
    let _ = std::fs::write(&marker, tier);
    Ok(())
}

fn default_launch_props(tier: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();
    props.insert("showFps".into(), "true".into());
    props.insert("showPing".into(), "true".into());
    props.insert("showKeystrokes".into(), "true".into());
    props.insert("showPerfBadge".into(), "false".into());
    props.insert("hardwareTier".into(), tier.into());
    props.insert("boostFps".into(), "true".into());
    props.insert("applyVanillaPreset".into(), "true".into());
    props.insert("memoryCleanup".into(), "true".into());
    props.insert("skipCombatFx".into(), "true".into());
    props.insert("reduceFpsWhenMinimized".into(), "true".into());
    props.insert("minimizedFps".into(), "5".into());
    props.insert("showCoords".into(), "false".into());
    props.insert("showArmor".into(), "true".into());
    props.insert("showCps".into(), "true".into());
    props.insert("toggleSprint".into(), "true".into());
    props.insert("pvpTrainingAutoWorld".into(), "false".into());
    match tier {
        "baja" => {
            props.insert("showKeystrokes".into(), "false".into());
            props.insert("particleMode".into(), "MINIMAL".into());
            props.insert("renderDistance".into(), "8".into());
            props.insert("simulationDistance".into(), "6".into());
            props.insert("entityDistanceScaling".into(), "0.5".into());
        }
        "media" => {
            props.insert("particleMode".into(), "REDUCED".into());
            props.insert("renderDistance".into(), "12".into());
            props.insert("simulationDistance".into(), "10".into());
            props.insert("entityDistanceScaling".into(), "0.75".into());
        }
        _ => {
            props.insert("particleMode".into(), "REDUCED".into());
            props.insert("renderDistance".into(), "12".into());
            props.insert("simulationDistance".into(), "10".into());
            props.insert("entityDistanceScaling".into(), "0.75".into());
        }
    }
    props
}

/// Perfil practica PvP: HUD de entrenamiento y mundo flat automatico.
pub fn apply_training_profile(instance_id: &str, tier: &str, auto_world: bool) -> AppResult<()> {
    let path = instances::instance_dir(instance_id).join("paraguacraft_modern.properties");
    let mut props = read_properties(&path);
    if props.is_empty() {
        apply_launch_properties(instance_id, tier)?;
        props = read_properties(&path);
    }
    props.insert("pvpTrainingAutoWorld".into(), auto_world.to_string());
    props.insert("showCoords".into(), "true".into());
    props.insert("showArmor".into(), "true".into());
    props.insert("showCps".into(), "true".into());
    props.insert("toggleSprint".into(), "true".into());
    props.insert("showKeystrokes".into(), "true".into());
    props.insert("showFps".into(), "true".into());
    props.insert("showPing".into(), "true".into());
    props.insert("boostFps".into(), "true".into());
    match tier {
        "baja" => {
            props.insert("particleMode".into(), "MINIMAL".into());
        }
        _ => {
            props.insert("particleMode".into(), "REDUCED".into());
        }
    }
    write_properties(&path, &props)
}

fn read_properties(path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(text) = std::fs::read_to_string(path) else {
        return map;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().into(), v.trim().into());
        }
    }
    map
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

fn set_jar_enabled(path: &Path, enabled: bool) -> AppResult<()> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::msg("Nombre invalido"))?;
    let base = file_name.trim_end_matches(".disabled");
    let new_name = if enabled {
        base.to_string()
    } else if file_name.ends_with(".disabled") {
        return Ok(());
    } else {
        format!("{base}.disabled")
    };
    if new_name == file_name {
        return Ok(());
    }
    std::fs::rename(path, path.with_file_name(&new_name))?;
    Ok(())
}

pub async fn sync_hud_mods(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_id: &str,
) -> AppResult<u32> {
    let tier = tier_level(&tier_from_hardware());
    let mods_dir = instances::instance_dir(instance_id).join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let mut installed = 0u32;
    for spec in HUD_MODS {
        if spec.min_tier > tier {
            continue;
        }
        if mods_dir
            .read_dir()
            .into_iter()
            .flatten()
            .flatten()
            .any(|e| {
                let n = e.file_name().to_string_lossy().to_lowercase();
                n.contains(spec.slug) && n.ends_with(".jar")
            })
        {
            continue;
        }
        if modrinth::install(
            app,
            client,
            spec.slug,
            "mod",
            MC,
            "fabric",
            mods_dir.clone(),
        )
        .await
        .is_ok()
        {
            installed += 1;
        }
    }
    Ok(installed)
}
