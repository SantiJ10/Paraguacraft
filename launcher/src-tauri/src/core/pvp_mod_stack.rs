//! Stack de mods **fijado** para Paraguacraft PvP Modern 1.21.11 (Fabric 0.19.x).
//!
//! Iris 1.10.7 (última en Modrinth para 1.21.11) exige Sodium **0.8.7** y Sodium 0.8.13+
//! declara `breaks iris <= 1.10.7`. No existe hoy un par Iris+Sodium 0.8.13 estable en Modrinth.

/// Mod con versión Modrinth fijada para evitar mezclas incompatibles.
pub struct PinnedMod {
    pub slug: &'static str,
    pub version_id: &'static str,
    pub filename: &'static str,
    /// 0=baja, 1=media, 2=alta. `None` = siempre (bundle base).
    pub min_tier: Option<u8>,
}

const STACK_1_21_11: &[PinnedMod] = &[
    // --- Render / optimización base (bundle) ---
    PinnedMod {
        slug: "sodium",
        version_id: "UddlN6L4",
        filename: "sodium-fabric-0.8.7+mc1.21.11.jar",
        min_tier: None,
    },
    PinnedMod {
        slug: "iris",
        version_id: "fDpuVzVr",
        filename: "iris-fabric-1.10.7+mc1.21.11.jar",
        min_tier: None,
    },
    PinnedMod {
        slug: "fabric-api",
        version_id: "zGF3drOQ",
        filename: "fabric-api-0.141.5+1.21.11.jar",
        min_tier: None,
    },
    PinnedMod {
        slug: "lithium",
        version_id: "Ow7wA0kG",
        filename: "lithium-fabric-0.21.4+mc1.21.11.jar",
        min_tier: None,
    },
    PinnedMod {
        slug: "ferrite-core",
        version_id: "Ii0gP3D8",
        filename: "ferritecore-8.2.0-fabric.jar",
        min_tier: None,
    },
    PinnedMod {
        slug: "entityculling",
        version_id: "sP0vNbeN",
        filename: "entityculling-fabric-1.10.5-mc1.21.11.jar",
        min_tier: None,
    },
    PinnedMod {
        slug: "immediatelyfast",
        version_id: "4EwhsTu7",
        filename: "ImmediatelyFast-Fabric-1.14.3+1.21.11.jar",
        min_tier: None,
    },
    PinnedMod {
        slug: "modmenu",
        version_id: "j2vTurvl",
        filename: "modmenu-17.0.1-beta.1.jar",
        min_tier: None,
    },
    // --- Sodium extras (requieren Sodium 0.8.7 con Iris 1.10.7) ---
    PinnedMod {
        slug: "sodium-extra",
        version_id: "yqY1efrC",
        filename: "sodium-extra-fabric-0.8.3+mc1.21.11.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "reeses-sodium-options",
        version_id: "P0MH4cn0",
        filename: "reeses-sodium-options-fabric-2.2.3+mc1.21.11.jar",
        min_tier: Some(0),
    },
    // --- HUD / QoL tier baja ---
    PinnedMod {
        slug: "searchables",
        version_id: "8t7XWQgt",
        filename: "Searchables-fabric-1.21.11-1.0.4.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "fabric-language-kotlin",
        version_id: "bdhiINYC",
        filename: "fabric-language-kotlin-1.13.13+kotlin.2.4.10.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "cloth-config",
        version_id: "xuX40TN5",
        filename: "cloth-config-21.11.153-fabric.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "yacl",
        version_id: "pHWDw3Vc",
        filename: "yet_another_config_lib_v3-3.8.2+1.21.11-fabric.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "gamma-utils",
        version_id: "aVJkWMQl",
        filename: "Gamma-Utils-2.5.10+mc1.21.11.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "appleskin",
        version_id: "59ti1rvg",
        filename: "appleskin-fabric-mc1.21.11-3.0.8.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "controlling",
        version_id: "A6W4m3vi",
        filename: "Controlling-fabric-1.21.11-29.0.1.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "smooth-scroll",
        version_id: "OVornAB5",
        filename: "smoothscroll-2.6.7.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "zoomify",
        version_id: "gI5KZI8V",
        filename: "zoomify-2.15.2+1.21.11.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "chat-heads",
        version_id: "gg00zA0j",
        filename: "chat_heads-1.2.5-fabric-1.21.11.jar",
        min_tier: Some(0),
    },
    PinnedMod {
        slug: "fast-ip-ping",
        version_id: "E3Ei5xUe",
        filename: "fast-ip-ping-v1.0.11-mc1.21.11-fabric.jar",
        min_tier: Some(0),
    },
    // --- tier media ---
    PinnedMod {
        slug: "better-ping-display-fabric",
        version_id: "HPJcBg0P",
        filename: "better-ping-display-fabric-1.21.11-1.2.0.jar",
        min_tier: Some(1),
    },
    PinnedMod {
        slug: "shulkerboxtooltip",
        version_id: "rZovgkWT",
        filename: "shulkerboxtooltip-fabric-5.2.16+1.21.11.jar",
        min_tier: Some(1),
    },
    PinnedMod {
        slug: "krypton",
        version_id: "O9LmWYR7",
        filename: "krypton-0.2.10.jar",
        min_tier: Some(1),
    },
    PinnedMod {
        slug: "debugify",
        version_id: "xVevtAGn",
        filename: "debugify-1.21.11+1.1.jar",
        min_tier: Some(1),
    },
    PinnedMod {
        slug: "moreculling",
        version_id: "wOzykoLV",
        filename: "moreculling-fabric-1.21.11-1.6.2.jar",
        min_tier: Some(1),
    },
    // --- tier alta ---
    PinnedMod {
        slug: "dynamic-fps",
        version_id: "Fab7e5Th",
        filename: "dynamic-fps-3.11.6+minecraft-1.21.11-fabric.jar",
        min_tier: Some(2),
    },
];

pub fn stack_for_mc(mc: &str) -> Option<&'static [PinnedMod]> {
    if mc == "1.21.11" {
        Some(STACK_1_21_11)
    } else {
        None
    }
}

pub fn pins_for_tier(mc: &str, tier: u8) -> Vec<&'static PinnedMod> {
    let Some(all) = stack_for_mc(mc) else {
        return Vec::new();
    };
    all.iter()
        .filter(|p| p.min_tier.is_none_or(|t| t <= tier))
        .collect()
}

pub fn bundle_pins(mc: &str) -> Vec<&'static PinnedMod> {
    let Some(all) = stack_for_mc(mc) else {
        return Vec::new();
    };
    all.iter().filter(|p| p.min_tier.is_none()).collect()
}

/// Slugs de mods pinneados que faltan en `mods/`.
pub fn missing_pins(mods_dir: &std::path::Path, mc: &str, tier: u8) -> Vec<&'static str> {
    pins_for_tier(mc, tier)
        .into_iter()
        .filter(|p| !mods_dir.join(p.filename).is_file())
        .map(|p| p.slug)
        .collect()
}

pub fn missing_pin_filenames(mods_dir: &std::path::Path, mc: &str, tier: u8) -> Vec<&'static str> {
    pins_for_tier(mc, tier)
        .into_iter()
        .filter(|p| !mods_dir.join(p.filename).is_file())
        .map(|p| p.filename)
        .collect()
}

/// Quita JARs de mods pinneados cuya versión no coincide (p.ej. Sodium 0.8.13 con Iris 1.10.7).
pub fn enforce_pinned_stack(mods_dir: &std::path::Path, mc: &str, tier: u8) {
    let pins = pins_for_tier(mc, tier);
    if pins.is_empty() {
        return;
    }
    let allowed: std::collections::HashSet<String> = pins
        .iter()
        .map(|p| p.filename.to_lowercase())
        .collect();

    for pin in &pins {
        purge_slug_except(mods_dir, pin.slug, pin.filename);
    }

    // Seguridad extra: cualquier archivo pinneado con nombre distinto al permitido.
    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !name.ends_with(".jar") && !name.ends_with(".jar.disabled") {
            continue;
        }
        if name.contains("paraguacraftpvp-modern") {
            continue;
        }
        for pin in &pins {
            if slug_matches(&name, pin.slug) && !allowed.contains(name.trim_end_matches(".disabled")) {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

fn slug_matches(filename: &str, slug: &str) -> bool {
    let slug = slug.to_lowercase();
    match slug.as_str() {
        "sodium" => {
            filename.contains("sodium-fabric")
                && !filename.contains("sodium-extra")
                && !filename.contains("reeses-sodium")
        }
        "fabric-api" => filename.contains("fabric-api"),
        "cloth-config" => filename.contains("cloth-config") || filename.contains("clothconfig"),
        "fabric-language-kotlin" => filename.contains("fabric-language-kotlin"),
        "ferrite-core" => filename.contains("ferritecore"),
        "immediatelyfast" => filename.contains("immediatelyfast"),
        "better-ping-display-fabric" => filename.contains("better-ping-display"),
        "smooth-scroll" => filename.contains("smoothscroll") || filename.contains("smooth-scroll"),
        "yacl" => filename.contains("yet_another_config_lib"),
        "gamma-utils" => filename.contains("gamma-utils") || filename.contains("gamma_utils"),
        "chat-heads" => filename.contains("chat_heads") || filename.contains("chat-heads"),
        "fast-ip-ping" => filename.contains("fast-ip-ping"),
        "reeses-sodium-options" => filename.contains("reeses-sodium-options"),
        _ => filename.contains(&slug.replace('-', "")) || filename.contains(&slug),
    }
}

fn purge_slug_except(mods_dir: &std::path::Path, slug: &str, keep_filename: &str) {
    let keep = keep_filename.to_lowercase();
    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !name.ends_with(".jar") && !name.ends_with(".jar.disabled") {
            continue;
        }
        if !slug_matches(&name, slug) {
            continue;
        }
        if name.trim_end_matches(".disabled") != keep {
            let _ = std::fs::remove_file(path);
        }
    }
}
