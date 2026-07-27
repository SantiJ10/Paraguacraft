//! **Paraguacraft Optimized** — preset de FPS (mods tipo Keo + shaders por gama + options).
//!
//! Versiones:
//! - 1.8.9 / 1.12.2 → OptiFine + shaders compatibles
//! - 1.18.2 / 1.20.1 / 1.21.11 / 26.2 → Fabric + mods Keo-like + Iris shaders
//! - 1.20.1 → también NeoForge + Embeddium/Oculus + shaders
//!
//! En esas MCs Fabric, **reemplaza** a Fabric+Iris en el selector.

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::hardware;
use crate::core::net::{self, DownloadItem};
use crate::core::performance;
use crate::error::{AppError, AppResult};

use super::{fabric, neoforge, optifine};

pub const ID: &str = "paraguacraft-optimized";
pub const ID_NEOFORGE: &str = "paraguacraft-optimized-neoforge";

const FABRIC_MCS: &[&str] = &["1.18.2", "1.20.1", "1.21.11", "26.2"];
const OPTIFINE_MCS: &[&str] = &["1.8.9", "1.12.2"];
const NEOFORGE_MCS: &[&str] = &["1.20.1"];

const MODRINTH: &str = "https://api.modrinth.com/v2";
const TUNED_MARKER: &str = ".paraguacraft_optimized_tuned";
const TUNED_VERSION: &str = "v4";
/// Única versión del pack Optimized que ve el usuario (el loader real se resuelve solo).
pub const PACK_VERSION: &str = "1.0.0";

/// Pin Modrinth (`version_id`) — sets cerrados y compatibles entre sí.
struct Pin {
    slug: &'static str,
    version_id: &'static str,
}

/// 1.21.11: Sodium 0.8.12 + Iris 1.10.7 (0.8.13 rompe Iris ≤1.10.7).
const PINS_1_21_11: &[Pin] = &[
    Pin { slug: "sodium", version_id: "NFkjnzWE" },
    Pin { slug: "iris", version_id: "fDpuVzVr" },
    Pin { slug: "fabric-api", version_id: "zGF3drOQ" },
    Pin { slug: "lithium", version_id: "Ow7wA0kG" },
    Pin { slug: "ferrite-core", version_id: "Ii0gP3D8" },
    Pin { slug: "entityculling", version_id: "sP0vNbeN" },
    Pin { slug: "immediatelyfast", version_id: "4EwhsTu7" },
    Pin { slug: "modmenu", version_id: "j2vTurvl" },
    Pin { slug: "sodium-extra", version_id: "yqY1efrC" },
    Pin { slug: "reeses-sodium-options", version_id: "P0MH4cn0" },
    Pin { slug: "fabric-language-kotlin", version_id: "bdhiINYC" },
    Pin { slug: "cloth-config", version_id: "xuX40TN5" },
    Pin { slug: "moreculling", version_id: "wOzykoLV" },
    Pin { slug: "clumps", version_id: "OgBE8Rz4" },
    Pin { slug: "lmd", version_id: "7gmpSYHk" },
    Pin { slug: "particle-core", version_id: "kIlv5noY" },
    Pin { slug: "fastquit", version_id: "ip2tVKLp" },
    Pin { slug: "scalablelux", version_id: "ju27pK32" },
    Pin { slug: "fzzy-config", version_id: "nSB6xGOS" },
    Pin { slug: "krypton", version_id: "O9LmWYR7" },
    Pin { slug: "modernfix-mvus", version_id: "yPCwXBn8" },
    Pin { slug: "noisiumforked", version_id: "VyMvRQKq" },
    Pin { slug: "smooth-boot", version_id: "gqFyBDHt" },
    Pin { slug: "better-block-entities", version_id: "VTUyYQyY" },
    Pin { slug: "fpsdisplay", version_id: "Tiz1VFDa" },
    Pin { slug: "renderscale", version_id: "e5ZPKfY7" },
    Pin { slug: "almanac", version_id: "Tcl38ycb" },
    Pin { slug: "placeholder-api", version_id: "qxjzQ9xY" },
    Pin { slug: "dynamic-fps", version_id: "Fab7e5Th" },
    Pin { slug: "badoptimizations", version_id: "Q3Dusz2j" },
];

/// 26.2: Sodium 0.9.1 + Iris 1.11.2 (pin Iris→Sodium).
const PINS_26_2: &[Pin] = &[
    Pin { slug: "sodium", version_id: "2Yom1N68" },
    Pin { slug: "iris", version_id: "oaD6KQls" },
    Pin { slug: "fabric-api", version_id: "lVXlbH4w" },
    Pin { slug: "lithium", version_id: "UPNexAfy" },
    Pin { slug: "ferrite-core", version_id: "d5ddUdiB" },
    Pin { slug: "entityculling", version_id: "iiF6U3Ne" },
    Pin { slug: "immediatelyfast", version_id: "uJHxuQxy" },
    Pin { slug: "modmenu", version_id: "njXb639R" },
    Pin { slug: "moreculling", version_id: "SYFaYeMK" },
    Pin { slug: "sodium-extra", version_id: "Fu02wj4x" },
    Pin { slug: "reeses-sodium-options", version_id: "PH4SPorH" },
    Pin { slug: "cloth-config", version_id: "Nv3xnWXd" },
    Pin { slug: "clumps", version_id: "dEMopoOJ" },
    Pin { slug: "lmd", version_id: "B2nrDb9C" },
    Pin { slug: "particle-core", version_id: "VFv6uQKM" },
    Pin { slug: "scalablelux", version_id: "EKLUURiy" },
    Pin { slug: "fzzy-config", version_id: "EQSFgLYw" },
    Pin { slug: "fabric-language-kotlin", version_id: "bdhiINYC" },
    Pin { slug: "modernfix-mvus", version_id: "TUWH6NZu" },
    Pin { slug: "noisiumforked", version_id: "rWMnuBfv" },
    Pin { slug: "dynamic-fps", version_id: "pC2JjFw1" },
    Pin { slug: "better-block-entities", version_id: "Sr2VjbpG" },
    Pin { slug: "badoptimizations", version_id: "JmPs4Wie" },
    Pin { slug: "asynclogger", version_id: "MiNRI6LE" },
    Pin { slug: "ixeris", version_id: "6I2BEXfJ" },
    Pin { slug: "gnetum", version_id: "FYk1Bn5B" },
    Pin { slug: "zfastnoise", version_id: "QBcjDrhr" },
    Pin { slug: "fism", version_id: "4HgtdJ7f" },
    Pin { slug: "forge-config-api-port", version_id: "rSd3GiG8" },
    Pin { slug: "get-it-together-drops", version_id: "BOGqc3kp" },
    Pin { slug: "noxesium", version_id: "Qx0oq0L0" },
    Pin { slug: "almanac", version_id: "meYgadd9" },
    Pin { slug: "fpsdisplay", version_id: "inkvL2AV" },
    Pin { slug: "renderscale", version_id: "EJ7KZw6k" },
    Pin { slug: "placeholder-api", version_id: "NDqH16LT" },
];

/// 1.20.1 Fabric: Sodium 0.5.12-beta.2 (exigido por Iris 1.7.6). Sin Indium (pin distinto).
const PINS_1_20_1_FABRIC: &[Pin] = &[
    Pin { slug: "sodium", version_id: "ryOMVRuG" },
    Pin { slug: "iris", version_id: "s5eFLITc" },
    Pin { slug: "fabric-api", version_id: "xhLT3C5f" },
    Pin { slug: "lithium", version_id: "iEcXOkz4" },
    Pin { slug: "ferrite-core", version_id: "unerR5MN" },
    Pin { slug: "entityculling", version_id: "infkTCSN" },
    Pin { slug: "immediatelyfast", version_id: "iwYUrQJO" },
    Pin { slug: "modmenu", version_id: "lEkperf6" },
    Pin { slug: "moreculling", version_id: "3wkuUDPy" },
    Pin { slug: "sodium-extra", version_id: "mDbF0LZT" },
    Pin { slug: "reeses-sodium-options", version_id: "Rc9pkPug" },
    Pin { slug: "cloth-config", version_id: "2xQdCMyG" },
    Pin { slug: "clumps", version_id: "hefSwtn6" },
    Pin { slug: "lmd", version_id: "rOkgwJ12" },
    Pin { slug: "particle-core", version_id: "6es9W10B" },
    Pin { slug: "fastquit", version_id: "tNgyOUMr" },
    Pin { slug: "fzzy-config", version_id: "qkBkQTfU" },
    Pin { slug: "fabric-language-kotlin", version_id: "bdhiINYC" },
    Pin { slug: "krypton", version_id: "jiDwS0W1" },
    Pin { slug: "modernfix", version_id: "rPmgLeZC" },
    Pin { slug: "noisium", version_id: "erSJnRcq" },
    Pin { slug: "smooth-boot", version_id: "t9nlpa0M" },
    Pin { slug: "dynamic-fps", version_id: "QwPQBhiQ" },
    Pin { slug: "badoptimizations", version_id: "DIugITgU" },
    Pin { slug: "asynclogger", version_id: "2HCbK9wC" },
    Pin { slug: "ixeris", version_id: "R0Ia5zWt" },
    Pin { slug: "zfastnoise", version_id: "K3nDfeZE" },
    Pin { slug: "fism", version_id: "JZF3sYNu" },
    Pin { slug: "forge-config-api-port", version_id: "HvR3IdRE" },
    Pin { slug: "get-it-together-drops", version_id: "ATcsrMNy" },
    Pin { slug: "almanac", version_id: "QM6nx1Sa" },
    Pin { slug: "fpsdisplay", version_id: "WaO5IB1q" },
    Pin { slug: "renderscale", version_id: "u7FyypwR" },
];

/// 1.18.2 Fabric: Sodium 0.4.1 + Iris 1.6.11 + Indium (mismo Sodium).
const PINS_1_18_2: &[Pin] = &[
    Pin { slug: "sodium", version_id: "74Y5Z8fo" },
    Pin { slug: "iris", version_id: "ogIRhnAJ" },
    Pin { slug: "indium", version_id: "H45YVREb" },
    Pin { slug: "fabric-api", version_id: "qk28POfr" },
    Pin { slug: "lithium", version_id: "ALnv7Npy" },
    Pin { slug: "ferrite-core", version_id: "776Z5oW9" },
    Pin { slug: "entityculling", version_id: "d3CbGntl" },
    Pin { slug: "immediatelyfast", version_id: "D73h6MNI" },
    Pin { slug: "modmenu", version_id: "nVxObSbX" },
    Pin { slug: "moreculling", version_id: "d2OS47y6" },
    Pin { slug: "sodium-extra", version_id: "DwCPxThW" },
    Pin { slug: "reeses-sodium-options", version_id: "BZU4WdR5" },
    Pin { slug: "cloth-config", version_id: "BLMp2TRt" },
    Pin { slug: "clumps", version_id: "hwWceR4m" },
    Pin { slug: "lmd", version_id: "YjlZw4Eo" },
    Pin { slug: "fastquit", version_id: "GKzeP8Zr" },
    Pin { slug: "fabric-language-kotlin", version_id: "bdhiINYC" },
    Pin { slug: "krypton", version_id: "vJQ7plH2" },
    Pin { slug: "modernfix", version_id: "LJ5N4YSl" },
    Pin { slug: "dynamic-fps", version_id: "maKzAqnY" },
    Pin { slug: "ixeris", version_id: "70H5sXBn" },
    Pin { slug: "forge-config-api-port", version_id: "XGKEYlsw" },
    Pin { slug: "get-it-together-drops", version_id: "v3JhWu9o" },
    Pin { slug: "fpsdisplay", version_id: "pxyG0qAH" },
];

/// 1.20.1 NeoForge/Forge: Embeddium + Oculus + stack Keo-like.
const PINS_1_20_1_NEOFORGE: &[Pin] = &[
    Pin { slug: "embeddium", version_id: "UTbfe5d1" },
    Pin { slug: "oculus", version_id: "iQ1SwGc3" },
    Pin { slug: "modernfix", version_id: "QroNBg6X" },
    Pin { slug: "ferrite-core", version_id: "DG5Fn9Sz" },
    Pin { slug: "entityculling", version_id: "MloBcsQQ" },
    Pin { slug: "immediatelyfast", version_id: "hGriwiGl" },
    Pin { slug: "clumps", version_id: "nAHGB5ls" },
    Pin { slug: "noisium", version_id: "gbYUKrDP" },
    Pin { slug: "cull-less-leaves-reforged", version_id: "wPOb8yEG" },
    Pin { slug: "memoryleakfix", version_id: "3w0IxNtk" },
    Pin { slug: "smooth-boot-reloaded", version_id: "HkfL3iGO" },
    Pin { slug: "cloth-config", version_id: "t8TXrZvZ" },
    Pin { slug: "chloride", version_id: "8cnn9uOM" },
    Pin { slug: "starlight-forge", version_id: "cNa0vkNj" },
    Pin { slug: "fastload", version_id: "5caSj7kt" },
    Pin { slug: "kotlin-for-forge", version_id: "Zsh14XeQ" },
    Pin { slug: "lmd", version_id: "a1JZWhla" },
    Pin { slug: "sodium-options-api", version_id: "d0EFLitO" },
    Pin { slug: "ai-improvements", version_id: "eJihmpNQ" },
    Pin { slug: "alternate-current", version_id: "kC6SY4Zp" },
    Pin { slug: "log-begone", version_id: "kNRTeHhj" },
    Pin { slug: "carbon-config", version_id: "LWHzkwbX" },
    Pin { slug: "lmft", version_id: "XJdD8eB6" },
    Pin { slug: "get-it-together-drops", version_id: "csPzTtJp" },
    Pin { slug: "immersive-optimization", version_id: "fWotFHdM" },
    Pin { slug: "badoptimizations", version_id: "DIugITgU" },
];

/// JARs a purgar siempre (incompatibles / Java 25+).
const PURGE_NAME_NEEDLES: &[&str] = &[
    "voxy",
    "c2me",
    "concurrentchunkmanagement",
];

fn fabric_pins_for_mc(mc: &str) -> Option<&'static [Pin]> {
    match mc {
        "1.21.11" => Some(PINS_1_21_11),
        "26.2" => Some(PINS_26_2),
        "1.20.1" => Some(PINS_1_20_1_FABRIC),
        "1.18.2" => Some(PINS_1_18_2),
        _ => None,
    }
}

pub fn is_fabric_mc(mc: &str) -> bool {
    FABRIC_MCS.iter().any(|v| *v == mc)
}

pub fn is_optifine_mc(mc: &str) -> bool {
    OPTIFINE_MCS.iter().any(|v| *v == mc)
}

pub fn is_neoforge_mc(mc: &str) -> bool {
    NEOFORGE_MCS.iter().any(|v| *v == mc)
}

/// True si Optimized sustituye a Fabric+Iris para esta MC.
pub fn replaces_fabric_iris(mc: &str) -> bool {
    is_fabric_mc(mc)
}

pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    // Una sola versión de pack; el backend (Fabric/OptiFine) se elige solo al instalar.
    if is_optifine_mc(mc) || is_fabric_mc(mc) {
        let _ = client; // keep signature
        return Ok(vec![PACK_VERSION.to_string()]);
    }
    Ok(vec![])
}

pub async fn versions_neoforge(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    if is_neoforge_mc(mc) {
        let _ = client;
        return Ok(vec![PACK_VERSION.to_string()]);
    }
    Ok(vec![])
}

async fn resolve_backend_loader_version(
    client: &reqwest::Client,
    mc: &str,
    pack_or_loader_version: &str,
    backend: &str,
) -> AppResult<String> {
    let v = pack_or_loader_version.trim();
    if !v.is_empty() && v != PACK_VERSION {
        return Ok(v.to_string());
    }
    let vers = match backend {
        "optifine" => optifine::versions(client, mc).await?,
        "neoforge" => neoforge::versions(client, mc).await?,
        _ => fabric::versions(client, mc).await?,
    };
    vers.first()
        .cloned()
        .ok_or_else(|| AppError::msg(format!("Sin versión de {backend} para Minecraft {mc}")))
}

pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader_version: &str,
) -> AppResult<String> {
    if is_optifine_mc(mc) {
        let ver = resolve_backend_loader_version(client, mc, loader_version, "optifine").await?;
        return optifine::install(app, client, mc, &ver).await;
    }
    if is_fabric_mc(mc) {
        let ver = resolve_backend_loader_version(client, mc, loader_version, "fabric").await?;
        return fabric::install(app, client, mc, &ver).await;
    }
    Err(AppError::msg(format!(
        "Paraguacraft Optimized no soporta Minecraft {mc}"
    )))
}

pub async fn install_neoforge(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader_version: &str,
) -> AppResult<String> {
    if !is_neoforge_mc(mc) {
        return Err(AppError::msg(format!(
            "Paraguacraft Optimized (NeoForge) no soporta Minecraft {mc}"
        )));
    }
    let ver = resolve_backend_loader_version(client, mc, loader_version, "neoforge").await?;
    neoforge::install(app, client, mc, &ver).await
}

/// Sync mods + shaders + preconfig al lanzar.
pub async fn install_bundle_for_launch(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader: &str,
    instance_dir: &Path,
) -> AppResult<()> {
    let kind = loader.trim().to_lowercase();
    let hw = hardware::detect();
    let tier = hw.perfil_sugerido.clone();

    if kind.contains("neoforge") {
        purge_incompatible_jars(instance_dir);
        install_pinned_mods(app, client, PINS_1_20_1_NEOFORGE, instance_dir).await?;
        install_shaders_for_tier(app, client, mc, &["iris", "optifine"], &tier, instance_dir)
            .await?;
        apply_preconfig_once(instance_dir, &tier, "neoforge")?;
    } else if is_fabric_mc(mc) {
        purge_incompatible_jars(instance_dir);
        install_fabric_compatible_bundle(app, client, mc, instance_dir).await?;
        install_shaders_for_tier(app, client, mc, &["iris", "optifine"], &tier, instance_dir)
            .await?;
        apply_preconfig_once(instance_dir, &tier, "fabric")?;
    } else if is_optifine_mc(mc) {
        install_shaders_for_tier(app, client, mc, &["optifine"], &tier, instance_dir).await?;
        apply_preconfig_once(instance_dir, &tier, "optifine")?;
    }

    Ok(())
}

fn purge_incompatible_jars(instance_dir: &Path) {
    let mods_dir = instance_dir.join("mods");
    let Ok(entries) = std::fs::read_dir(&mods_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !(name.ends_with(".jar") || name.ends_with(".jar.disabled")) {
            continue;
        }
        let compact = name.replace(['-', '_', ' '], "");
        if PURGE_NAME_NEEDLES
            .iter()
            .any(|n| name.contains(n) || compact.contains(n))
        {
            let _ = std::fs::remove_file(&path);
            eprintln!("[optimized] purged incompatible {}", path.display());
        }
    }
}

async fn install_fabric_compatible_bundle(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    instance_dir: &Path,
) -> AppResult<()> {
    let Some(pins) = fabric_pins_for_mc(mc) else {
        return Err(AppError::msg(format!(
            "Paraguacraft Optimized no tiene set pineado para Minecraft {mc}"
        )));
    };
    install_pinned_mods(app, client, pins, instance_dir).await
}

async fn install_pinned_mods(
    app: &AppHandle,
    client: &reqwest::Client,
    pins: &[Pin],
    instance_dir: &Path,
) -> AppResult<()> {
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let mut items = Vec::new();
    for pin in pins {
        match resolve_modrinth_version_file(client, pin.version_id).await {
            Ok((url, fname, sha1)) => {
                // Quitar otras builds del mismo slug.
                purge_slug_jars(&mods_dir, pin.slug, &fname);
                let dest = mods_dir.join(&fname);
                if dest.is_file() {
                    continue;
                }
                items.push(DownloadItem::new(url, dest).with_sha1(sha1));
            }
            Err(e) => eprintln!("[optimized] pin {} failed: {e}", pin.slug),
        }
    }
    if !items.is_empty() {
        net::download_all(
            client,
            items,
            8,
            app,
            "optimized-pins",
            "Paraguacraft Optimized (pins)",
        )
        .await?;
    }
    Ok(())
}

fn purge_slug_jars(mods_dir: &Path, slug: &str, keep_filename: &str) {
    let keep = keep_filename.to_lowercase();
    let slug_l = slug.to_lowercase();
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
        if !(name.ends_with(".jar") || name.ends_with(".jar.disabled")) {
            continue;
        }
        let matches = match slug_l.as_str() {
            "sodium" => {
                name.contains("sodium-fabric")
                    && !name.contains("sodium-extra")
                    && !name.contains("reeses-sodium")
            }
            "embeddium" => name.contains("embeddium"),
            "ferrite-core" => name.contains("ferritecore"),
            "reeses-sodium-options" => {
                name.contains("reeses-sodium-options") || name.contains("reeses_sodium_options")
            }
            "sodium-extra" => name.contains("sodium-extra"),
            "fabric-api" => name.contains("fabric-api"),
            "modernfix-mvus" | "modernfix" => name.contains("modernfix"),
            other => name.contains(other) || name.contains(&other.replace('-', "")),
        };
        if matches && name.trim_end_matches(".disabled") != keep {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn shader_slugs_for_tier(tier: &str) -> &'static [&'static str] {
    // Alternativas por gama (desactivadas por defecto). Livianas primero.
    match tier {
        "alta" => &[
            "solas-shader",
            "complementary-reimagined",
            "bsl-shaders",
            "rethinking-voxels",
            "photon-shader",
            "mellow",
            "blocky-shader",
            "makeup-ultra-fast-shaders",
            "complementary-unbound",
        ],
        "media" => &[
            "solas-shader",
            "bsl-shaders",
            "complementary-reimagined",
            "makeup-ultra-fast-shaders",
            "super-duper-vanilla",
            "mellow",
            "blocky-shader",
            "miniature-shader",
        ],
        _ => &[
            "makeup-ultra-fast-shaders",
            "lite-shaders",
            "super-duper-vanilla",
            "blocky-shader",
            "miniature-shader",
            "solas-shader",
            "bsl-shaders",
        ],
    }
}

fn default_shader_for_tier(tier: &str) -> &'static str {
    match tier {
        "alta" => "solas-shader",
        "media" => "solas-shader",
        _ => "makeup-ultra-fast-shaders",
    }
}

fn iris_shadow_distance(tier: &str) -> &'static str {
    // Bajo a propósito: cuando el usuario active shaders, no satura tanto.
    match tier {
        "alta" => "12",
        "media" => "8",
        _ => "6",
    }
}

async fn install_shaders_for_tier(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loaders: &[&str],
    tier: &str,
    instance_dir: &Path,
) -> AppResult<()> {
    let dir = instance_dir.join("shaderpacks");
    std::fs::create_dir_all(&dir)?;
    let mut items = Vec::new();

    for slug in shader_slugs_for_tier(tier) {
        let mut got = None;
        for loader in loaders {
            match resolve_modrinth_file(client, slug, mc, loader).await {
                Ok(v) => {
                    got = Some(v);
                    break;
                }
                Err(_) => continue,
            }
        }
        if got.is_none() {
            for loader in loaders {
                if let Ok(v) = resolve_modrinth_file_any_mc(client, slug, loader).await {
                    got = Some(v);
                    break;
                }
            }
        }
        let Some((url, fname, sha1)) = got else {
            eprintln!("[optimized] skip shader {slug}@{mc}");
            continue;
        };
        let dest = dir.join(&fname);
        if dest.is_file() {
            continue;
        }
        items.push(DownloadItem::new(url, dest).with_sha1(sha1));
    }

    if !items.is_empty() {
        net::download_all(
            client,
            items,
            4,
            app,
            "optimized-shaders",
            &format!("Shaders gama {tier}"),
        )
        .await?;
    }
    Ok(())
}

fn apply_preconfig_once(instance_dir: &Path, tier: &str, backend: &str) -> AppResult<()> {
    let marker = instance_dir.join(TUNED_MARKER);
    if marker.is_file() {
        if let Ok(text) = std::fs::read_to_string(&marker) {
            if text.contains(TUNED_VERSION) {
                return Ok(());
            }
        }
    }

    // options + configs de mods (More Culling, Sodium Extra, BRD, etc.) por gama.
    let _ = performance::optimize_optimized_options(instance_dir, tier);
    let _ = performance::apply_optimized_mod_configs(instance_dir, tier);
    if backend == "optifine" {
        write_optifine_optionsof(instance_dir, tier)?;
    }
    write_shader_default(instance_dir, tier, backend)?;

    let _ = std::fs::write(
        &marker,
        format!("version={TUNED_VERSION}\ntier={tier}\nbackend={backend}\n"),
    );
    Ok(())
}

fn write_optifine_optionsof(instance_dir: &Path, tier: &str) -> AppResult<()> {
    let path = instance_dir.join("optionsof.txt");
    let pairs: &[(&str, &str)] = match tier {
        "alta" => &[
            ("ofFastRender", "false"),
            ("ofSmoothFps", "false"),
            ("ofSmoothWorld", "true"),
            ("ofAaLevel", "0"),
            ("ofAfLevel", "4"),
            ("ofClouds", "1"),
            ("ofTrees", "1"),
            ("ofDroppedItems", "1"),
            ("ofVignette", "1"),
            ("ofDynamicLights", "1"),
            ("ofAnimatedTerrain", "true"),
            ("ofAnimatedTextures", "true"),
            ("ofShowFps", "true"),
        ],
        "media" => &[
            ("ofFastRender", "false"),
            ("ofSmoothFps", "true"),
            ("ofSmoothWorld", "true"),
            ("ofAaLevel", "0"),
            ("ofAfLevel", "2"),
            ("ofClouds", "2"),
            ("ofTrees", "1"),
            ("ofDroppedItems", "1"),
            ("ofVignette", "1"),
            ("ofDynamicLights", "2"),
            ("ofAnimatedTerrain", "true"),
            ("ofAnimatedTextures", "true"),
            ("ofShowFps", "true"),
        ],
        _ => &[
            ("ofFastRender", "true"),
            ("ofSmoothFps", "true"),
            ("ofSmoothWorld", "false"),
            ("ofAaLevel", "0"),
            ("ofAfLevel", "1"),
            ("ofClouds", "3"),
            ("ofTrees", "0"),
            ("ofDroppedItems", "0"),
            ("ofVignette", "0"),
            ("ofDynamicLights", "3"),
            ("ofAnimatedTerrain", "false"),
            ("ofAnimatedTextures", "false"),
            ("ofShowFps", "true"),
        ],
    };
    patch_kv_file(&path, pairs)
}

fn write_shader_default(instance_dir: &Path, tier: &str, backend: &str) -> AppResult<()> {
    // Shaders DESCARGADOS pero DESACTIVADOS por defecto (mejor FPS out-of-box).
    let want_slug = default_shader_for_tier(tier);
    let pack_name = find_downloaded_shader_name(instance_dir, want_slug)
        .or_else(|| first_shaderpack_name(instance_dir))
        .unwrap_or_default();

    if backend == "optifine" {
        let path = instance_dir.join("optionsof.txt");
        // OptiFine: sin shader activo.
        patch_kv_file(&path, &[("ofShaderPack", "")])?;
        let options = instance_dir.join("options.txt");
        let mut map = HashMap::new();
        map.insert("shaderPack".into(), String::new());
        let _ = performance::merge_options_keys(&options, map);
    } else {
        let config = instance_dir.join("config");
        std::fs::create_dir_all(&config)?;
        let iris = config.join("iris.properties");
        let shadow = iris_shadow_distance(tier);
        let pack = if pack_name.is_empty() {
            "OFF"
        } else {
            pack_name.as_str()
        };
        patch_kv_file(
            &iris,
            &[
                ("shaderPack", pack),
                ("enableShaders", "false"),
                ("maxShadowRenderDistance", shadow),
                ("colorSpace", "SRGB"),
                ("allowUnknownShaders", "false"),
            ],
        )?;
        let oculus = config.join("oculus.properties");
        let _ = patch_kv_file(
            &oculus,
            &[
                ("shaderPack", pack),
                ("enableShaders", "false"),
                ("maxShadowRenderDistance", shadow),
            ],
        );
    }
    Ok(())
}

fn find_downloaded_shader_name(instance_dir: &Path, slug_hint: &str) -> Option<String> {
    let dir = instance_dir.join("shaderpacks");
    let rd = std::fs::read_dir(dir).ok()?;
    let hint = slug_hint.replace('-', "").to_lowercase();
    let mut files: Vec<String> = rd
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            if n.ends_with(".zip") || e.path().is_dir() {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files
        .iter()
        .find(|f| {
            let t = f.replace(['-', '_', ' '], "").to_lowercase();
            t.contains(&hint)
        })
        .cloned()
        .or_else(|| files.first().cloned())
}

fn first_shaderpack_name(instance_dir: &Path) -> Option<String> {
    let dir = instance_dir.join("shaderpacks");
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            if n.ends_with(".zip") || e.path().is_dir() {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files.into_iter().next()
}

fn patch_kv_file(path: &Path, pairs: &[(&str, &str)]) -> AppResult<()> {
    let mut map: HashMap<String, String> = HashMap::new();
    if path.is_file() {
        if let Ok(text) = std::fs::read_to_string(path) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    map.insert(k.trim().to_string(), v.trim().to_string());
                } else if let Some((k, v)) = line.split_once(':') {
                    map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
    }
    for (k, v) in pairs {
        map.insert((*k).into(), (*v).into());
    }
    let mut lines: Vec<String> = map
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    lines.sort();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, lines.join("\n") + "\n")?;
    Ok(())
}

async fn resolve_modrinth_version_file(
    client: &reqwest::Client,
    version_id: &str,
) -> AppResult<(String, String, Option<String>)> {
    let url = format!("{MODRINTH}/version/{version_id}");
    let ver: Value = net::fetch_json(client, &url).await?;
    let files = ver["files"].as_array().cloned().unwrap_or_default();
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool() == Some(true))
        .or_else(|| files.first())
        .ok_or_else(|| AppError::msg(format!("Sin archivo para version {version_id}")))?;
    let dl = file["url"]
        .as_str()
        .ok_or_else(|| AppError::msg("Sin URL de descarga"))?
        .to_string();
    let fname = file["filename"]
        .as_str()
        .unwrap_or("mod.jar")
        .to_string();
    let sha1 = file["hashes"]["sha1"].as_str().map(String::from);
    Ok((dl, fname, sha1))
}

async fn resolve_modrinth_file(
    client: &reqwest::Client,
    slug: &str,
    mc: &str,
    loader: &str,
) -> AppResult<(String, String, Option<String>)> {
    let url = format!(
        "{MODRINTH}/project/{slug}/version?loaders={}&game_versions={}",
        net::url_encode(&format!(r#"["{loader}"]"#)),
        net::url_encode(&format!(r#"["{mc}"]"#))
    );
    let versions: Value = net::fetch_json(client, &url).await?;
    pick_primary_file(&versions, slug)
}

async fn resolve_modrinth_file_any_mc(
    client: &reqwest::Client,
    slug: &str,
    loader: &str,
) -> AppResult<(String, String, Option<String>)> {
    let url = format!(
        "{MODRINTH}/project/{slug}/version?loaders={}",
        net::url_encode(&format!(r#"["{loader}"]"#)),
    );
    let versions: Value = net::fetch_json(client, &url).await?;
    pick_primary_file(&versions, slug)
}

fn pick_primary_file(versions: &Value, slug: &str) -> AppResult<(String, String, Option<String>)> {
    let arr = versions
        .as_array()
        .cloned()
        .ok_or_else(|| AppError::msg(format!("Sin versiones Modrinth para {slug}")))?;
    let ver = arr
        .first()
        .ok_or_else(|| AppError::msg(format!("Sin build de {slug}")))?;
    let files = ver["files"].as_array().cloned().unwrap_or_default();
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool() == Some(true))
        .or_else(|| files.first())
        .ok_or_else(|| AppError::msg(format!("Sin archivo para {slug}")))?;
    let dl = file["url"]
        .as_str()
        .ok_or_else(|| AppError::msg("Sin URL de descarga"))?
        .to_string();
    let fname = file["filename"]
        .as_str()
        .unwrap_or("mod.jar")
        .to_string();
    let sha1 = file["hashes"]["sha1"].as_str().map(String::from);
    Ok((dl, fname, sha1))
}
