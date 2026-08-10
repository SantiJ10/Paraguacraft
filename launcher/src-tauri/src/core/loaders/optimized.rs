//! **Paraguacraft Optimized** — preset de FPS (mods tipo Keo + shaders por gama + options).
//!
//! Versiones:
//! - 1.8.9 / 1.12.2 → Forge + OptiFine (mod) + FoamFix/VanillaFix/etc. + shaders OptiFine
//! Pack de optimización curado multi-MC:
//! - 1.18.2 / 1.20.1 / 1.21.11 / 26.2 → Fabric + mods Keo-like + Iris shaders
//! - 1.8.9 / 1.12.2 → Forge + OptiFine + mods de rendimiento
//!
//! Nota: Optimized NeoForge (1.20.1) se retiró por incompatibilidades / no disponible.
//!
//! En esas MCs Fabric, **reemplaza** a Fabric+Iris en el selector.

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;
use tauri::AppHandle;

use crate::config::keys;
use crate::core::hardware;
use crate::core::net::{self, DownloadItem};
use crate::core::performance;
use crate::core::store::curseforge;
use crate::error::{AppError, AppResult};

use super::{fabric, forge, neoforge, optifine};

pub const ID: &str = "paraguacraft-optimized";
pub const ID_NEOFORGE: &str = "paraguacraft-optimized-neoforge";

const FABRIC_MCS: &[&str] = &["1.18.2", "1.20.1", "1.21.11", "26.2"];
const OPTIFINE_MCS: &[&str] = &["1.8.9", "1.12.2"];
const NEOFORGE_MCS: &[&str] = &[]; // Optimized NeoForge retirado (1.20.1 no se ofrece).

/// Forge recomendado para Optimized legacy (mismo criterio que packs estables).
const FORGE_1_8_9: &str = "11.15.1.2318";
const FORGE_1_12_2: &str = "14.23.5.2860";

const MODRINTH: &str = "https://api.modrinth.com/v2";
const TUNED_MARKER: &str = ".paraguacraft_optimized_tuned";
const TUNED_VERSION: &str = "v9";
/// Única versión del pack Optimized que ve el usuario (el loader real se resuelve solo).
pub const PACK_VERSION: &str = "1.0.0";

/// Pin Modrinth (`version_id`) — sets cerrados y compatibles entre sí.
struct Pin {
    slug: &'static str,
    version_id: &'static str,
}

/// Pin CurseForge (project + file) para mods legacy Forge.
struct CfPin {
    slug: &'static str,
    project_id: u64,
    file_id: u64,
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
    // zfastnoise (Fast Noise) removido: conflicto/crash con noisium 2.3.0 en 1.20.1.
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
    // Conflictivo con noisium en Optimized 1.20.1 (Fast Noise / zfastnoise).
    "zfastnoise",
    "fastnoise",
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

/// Mods Forge de rendimiento para Optimized 1.12.2 (Modrinth — sin depender de CurseForge).
const MR_PINS_1_12_2: &[Pin] = &[
    Pin { slug: "foamfix", version_id: "oEGIQQnQ" },       // 0.10.15
    Pin { slug: "vanillafix", version_id: "1MMmIOiX" },    // 1.0.10-150
    Pin { slug: "clumps", version_id: "nZvGITpT" },        // 3.1.2
    Pin { slug: "ai-improvements", version_id: "kOyZhvg3" }, // 0.0.1b3
];

/// Mods Forge de rendimiento para Optimized 1.8.9 (Modrinth).
const MR_PINS_1_8_9: &[Pin] = &[
    Pin { slug: "foamfix", version_id: "MqLKfrk2" }, // 0.6.3a anarchy
];

/// Extras solo en CurseForge (si hay API key válida).
const CF_PINS_1_12_2: &[CfPin] = &[
    // Phosphor 0.2.6 — no está en Modrinth
    CfPin { slug: "phosphor", project_id: 306770, file_id: 2912855 },
    // BetterFps 1.4.8
    CfPin { slug: "betterfps", project_id: 229891, file_id: 2483393 },
    // FastWorkbench 1.7.3
    CfPin { slug: "fastworkbench", project_id: 288885, file_id: 2803428 },
    // FastFurnace 1.3.1
    CfPin { slug: "fastfurnace", project_id: 299540, file_id: 2746053 },
];

/// Extras CurseForge para 1.8.9.
const CF_PINS_1_8_9: &[CfPin] = &[
    // BetterFps 1.2.1 (1.8.x)
    CfPin { slug: "betterfps", project_id: 229891, file_id: 2283238 },
    // The 5zig Mod 3.11.3
    CfPin { slug: "the-5zig-mod", project_id: 231387, file_id: 2389910 },
];

fn mr_pins_for_legacy_mc(mc: &str) -> Option<&'static [Pin]> {
    match mc {
        "1.12.2" => Some(MR_PINS_1_12_2),
        "1.8.9" => Some(MR_PINS_1_8_9),
        _ => None,
    }
}

fn cf_pins_for_mc(mc: &str) -> Option<&'static [CfPin]> {
    match mc {
        "1.12.2" => Some(CF_PINS_1_12_2),
        "1.8.9" => Some(CF_PINS_1_8_9),
        _ => None,
    }
}

fn recommended_forge(mc: &str) -> Option<&'static str> {
    match mc {
        "1.8.9" => Some(FORGE_1_8_9),
        "1.12.2" => Some(FORGE_1_12_2),
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
    if backend == "forge" {
        if let Some(rec) = recommended_forge(mc) {
            return Ok(rec.to_string());
        }
    }
    let vers = match backend {
        "optifine" => optifine::versions(client, mc).await?,
        "neoforge" => neoforge::versions(client, mc).await?,
        "forge" => forge::versions(client, mc).await?,
        _ => fabric::versions(client, mc).await?,
    };
    if backend == "optifine" {
        return optifine::pick_best_version(mc, &vers)
            .ok_or_else(|| AppError::msg(format!("Sin versión de OptiFine para Minecraft {mc}")));
    }
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
    // 1.8.9 / 1.12.2: Forge + OptiFine como mod (permite FoamFix/VanillaFix/etc.).
    if is_optifine_mc(mc) {
        let ver = resolve_backend_loader_version(client, mc, loader_version, "forge").await?;
        return forge::install(app, client, mc, &ver).await;
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
        install_pinned_mods_required(
            app,
            client,
            PINS_1_20_1_NEOFORGE,
            instance_dir,
            REQUIRED_NEOFORGE_PINS,
        )
        .await?;
        // Oculus = Iris para NeoForge; no bajar packs pensados solo para OptiFine legacy.
        install_shaders_for_tier(app, client, mc, &["iris"], &tier, instance_dir).await?;
        apply_preconfig_once(instance_dir, &tier, "neoforge", mc)?;
    } else if is_fabric_mc(mc) {
        purge_incompatible_jars(instance_dir);
        install_fabric_compatible_bundle(app, client, mc, instance_dir).await?;
        install_shaders_for_tier(app, client, mc, &["iris"], &tier, instance_dir).await?;
        apply_preconfig_once(instance_dir, &tier, "fabric", mc)?;
    } else if is_optifine_mc(mc) {
        install_legacy_optifine_bundle(app, client, mc, instance_dir).await?;
        purge_non_optifine_shaderpacks(instance_dir);
        install_shaders_for_tier(app, client, mc, &["optifine"], &tier, instance_dir).await?;
        apply_preconfig_once(instance_dir, &tier, "optifine", mc)?;
    }

    Ok(())
}

async fn install_legacy_optifine_bundle(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    instance_dir: &Path,
) -> AppResult<()> {
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;

    // OptiFine estable (G5 / M5) como JAR en mods/ — oficial, luego BMCL.
    if let Some((of_type, of_patch)) = optifine::preferred_type_patch(mc) {
        let fname = optifine::mod_jar_filename(mc, of_type, of_patch);
        purge_slug_jars(&mods_dir, "optifine", &fname);
        let dest = mods_dir.join(&fname);
        if !dest.is_file() {
            purge_slug_jars(&mods_dir, "optifine", &fname);
            match optifine::download_mod_jar_official(client, mc, of_type, of_patch, &dest).await {
                Ok(()) => {}
                Err(e1) => {
                    eprintln!("[optimized] OptiFine oficial {fname}: {e1}");
                    match optifine::download_mod_jar_bmcl(client, mc, of_type, of_patch, &dest)
                        .await
                    {
                        Ok(()) => {}
                        Err(e2) => {
                            eprintln!("[optimized] OptiFine BMCL {fname}: {e2}");
                            return Err(AppError::msg(format!(
                                "No se pudo descargar OptiFine {fname}. Probá de nuevo o Reinstalar loader. ({e2})"
                            )));
                        }
                    }
                }
            }
        }
    }

    // Core de rendimiento vía Modrinth (no requiere CurseForge).
    if let Some(pins) = mr_pins_for_legacy_mc(mc) {
        install_pinned_mods(app, client, pins, instance_dir).await?;
    }
    // Extras CF opcionales (Phosphor, BetterFps…) si hay key válida.
    if let Some(pins) = cf_pins_for_mc(mc) {
        install_cf_pins(app, client, pins, &mods_dir).await?;
    }
    Ok(())
}

async fn install_cf_pins(
    app: &AppHandle,
    client: &reqwest::Client,
    pins: &[CfPin],
    mods_dir: &Path,
) -> AppResult<()> {
    let key = keys::curseforge_api_key();
    if key.trim().is_empty() {
        eprintln!("[optimized] sin CurseForge API key: se omite FoamFix/VanillaFix/etc.");
        return Ok(());
    }
    for pin in pins {
        match curseforge::install_file_id(
            app,
            client,
            &key,
            &pin.project_id.to_string(),
            &pin.file_id.to_string(),
            mods_dir.to_path_buf(),
        )
        .await
        {
            Ok(fname) => {
                purge_slug_jars(mods_dir, pin.slug, &fname);
            }
            Err(e) => eprintln!("[optimized] CF pin {} failed: {e}", pin.slug),
        }
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
    install_pinned_mods_required(app, client, pins, instance_dir, REQUIRED_FABRIC_PINS).await
}

/// Mods críticos: si fallan, el cliente Fabric/NeoForge no arranca.
const REQUIRED_FABRIC_PINS: &[&str] = &["fabric-api", "sodium", "iris", "lithium"];
const REQUIRED_NEOFORGE_PINS: &[&str] = &["embeddium", "oculus", "modernfix"];

async fn install_pinned_mods(
    app: &AppHandle,
    client: &reqwest::Client,
    pins: &[Pin],
    instance_dir: &Path,
) -> AppResult<()> {
    install_pinned_mods_inner(app, client, pins, instance_dir, &[]).await
}

async fn install_pinned_mods_required(
    app: &AppHandle,
    client: &reqwest::Client,
    pins: &[Pin],
    instance_dir: &Path,
    required: &[&str],
) -> AppResult<()> {
    install_pinned_mods_inner(app, client, pins, instance_dir, required).await
}

async fn install_pinned_mods_inner(
    app: &AppHandle,
    client: &reqwest::Client,
    pins: &[Pin],
    instance_dir: &Path,
    required: &[&str],
) -> AppResult<()> {
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let mut items = Vec::new();
    let mut failed_required = Vec::new();

    for pin in pins {
        let is_required = required.iter().any(|s| *s == pin.slug);
        match resolve_modrinth_version_file(client, pin.version_id).await {
            Ok((url, fname, sha1)) => {
                purge_slug_jars(&mods_dir, pin.slug, &fname);
                let dest = mods_dir.join(&fname);
                if dest.is_file() {
                    continue;
                }
                items.push(DownloadItem::new(url, dest).with_sha1(sha1));
            }
            Err(e) => {
                eprintln!("[optimized] pin {} failed: {e}", pin.slug);
                if is_required {
                    failed_required.push(format!("{} ({e})", pin.slug));
                }
            }
        }
    }

    if !failed_required.is_empty() {
        return Err(AppError::msg(format!(
            "No se pudieron resolver mods críticos de Optimized: {}. Probá de nuevo o Reparar instancia.",
            failed_required.join(", ")
        )));
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

    // Tras descargar: verificar que los JARs críticos existen en mods/.
    for slug in required {
        if !mod_jar_present_for_slug(&mods_dir, slug) {
            return Err(AppError::msg(format!(
                "Falta el mod crítico «{slug}» en Optimized. Revisá la conexión y usá Reparar instancia."
            )));
        }
    }
    Ok(())
}

fn mod_jar_present_for_slug(mods_dir: &Path, slug: &str) -> bool {
    let slug_l = slug.to_lowercase();
    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let n = e.file_name().to_string_lossy().to_lowercase();
        if !(n.ends_with(".jar") || n.ends_with(".jar.disabled")) {
            return false;
        }
        match slug_l.as_str() {
            "sodium" => {
                n.contains("sodium")
                    && !n.contains("sodium-extra")
                    && !n.contains("reeses")
                    && !n.contains("options")
            }
            "iris" => n.contains("iris") && !n.contains("oculus"),
            "oculus" => n.contains("oculus"),
            "embeddium" => n.contains("embeddium"),
            "fabric-api" => n.contains("fabric-api") || n.contains("fabric_api"),
            "lithium" => n.contains("lithium"),
            "modernfix" => n.contains("modernfix"),
            other => n.contains(other) || n.replace(['-', '_'], "").contains(&other.replace('-', "")),
        }
    })
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

const SHADER_LOW: &[&str] = &[
    // Livianos / laptops (Iris / Fabric / NeoForge+Oculus)
    "truelight-fx-lite",
    "makeup-ultra-fast-shaders",
    "potato-shaders",
    "essentials-shader",
    "lite-shaders",
    "super-duper-vanilla",
    "miniature-shader",
    "blocky-shader",
    "vanilla-plus-shader",
    "sildurs-basic-shaders",
];
const SHADER_MID: &[&str] = &[
    "solas-shader",
    "bsl-shaders",
    "complementary-reimagined",
    "mellow",
    "sildurs-vibrant-shaders",
    "daybreak-shader",
    "truelightfx",
];
const SHADER_HIGH: &[&str] = &[
    "complementary-unbound",
    "rethinking-voxels",
    "photon-shader",
    "insanity-shader",
    "hysteria-shaders",
];

/// Solo OptiFine (1.8.9 / 1.12.2): packs clásicos, no catálogo Iris moderno.
const SHADER_OPTIFINE_LOW: &[&str] = &[
    "sildurs-basic-shaders",
    "sildurs-vibrant-shaders",
];
const SHADER_OPTIFINE_MID: &[&str] = &["bsl-shaders", "sildurs-vibrant-shaders"];
const SHADER_OPTIFINE_HIGH: &[&str] = &["bsl-shaders"];

fn shader_slugs_for_tier_and_backend(tier: &str, backend_loaders: &[&str]) -> Vec<&'static str> {
    let optifine_only = backend_loaders.len() == 1 && backend_loaders[0] == "optifine";
    if optifine_only {
        return match tier {
            "alta" => {
                let mut v = Vec::new();
                v.extend_from_slice(SHADER_OPTIFINE_LOW);
                v.extend_from_slice(SHADER_OPTIFINE_MID);
                v.extend_from_slice(SHADER_OPTIFINE_HIGH);
                v
            }
            "media" => {
                let mut v = Vec::new();
                v.extend_from_slice(SHADER_OPTIFINE_LOW);
                v.extend_from_slice(SHADER_OPTIFINE_MID);
                v
            }
            _ => SHADER_OPTIFINE_LOW.to_vec(),
        };
    }
    shader_slugs_for_tier(tier)
}

/// Shaders hospedados en el repo (no están en Modrinth). CDN = raw GitHub.
struct HostedShader {
    /// Id interno (catálogo / matching de nombre).
    id: &'static str,
    /// Archivo en `bundled/optimized-shaders/`.
    filename: &'static str,
    sha256: &'static str,
    /// "baja" | "media" | "alta"
    min_tier: &'static str,
    label: &'static str,
}

const HOSTED_SHADERS: &[HostedShader] = &[
    // HighPerformance Low: pensado para iGPU / laptops.
    HostedShader {
        id: "chocapic13-hp-low",
        filename: "Chocapic13_HighPerformance_Low.zip",
        sha256: "52fd15ad39b9b3011a30f308b998df4e8f019330cbdf1965595e90b15a93d570",
        min_tier: "baja",
        label: "baja (~alta FPS, sombras básicas)",
    },
    // Skygleam: visuales buenas con foco en rendimiento (Keo lo trae).
    HostedShader {
        id: "skygleam",
        filename: "Skygleam_Shaders_V3.1.zip",
        sha256: "8cde9f45071193eab7e346f20ec695754a40d3398fcf153f47dd8a397e5166ef",
        min_tier: "baja",
        label: "baja–media (equilibrio FPS/calidad)",
    },
    // SOLAR: más pesado visualmente → media+.
    HostedShader {
        id: "solar-shader",
        filename: "SOLAR_Shader_v1.4.zip",
        sha256: "3a2906ce13f3b4e3af57f900247a46a9c1c5e2bfeba52920d27ef341609d5d07",
        min_tier: "media",
        label: "media (~estilo fantasy, más costo GPU)",
    },
];

const GITHUB_REPO: &str = "SantiJ10/Paraguacraft";

fn hosted_shader_urls(filename: &str) -> Vec<String> {
    vec![
        format!(
            "https://raw.githubusercontent.com/{GITHUB_REPO}/main/bundled/optimized-shaders/{filename}"
        ),
        format!("https://github.com/{GITHUB_REPO}/raw/main/bundled/optimized-shaders/{filename}"),
        format!(
            "https://cdn.jsdelivr.net/gh/{GITHUB_REPO}@main/bundled/optimized-shaders/{filename}"
        ),
    ]
}

fn tier_allows_hosted(tier: &str, min_tier: &str) -> bool {
    match (tier, min_tier) {
        (_, "baja") => true,
        ("media" | "alta", "media") => true,
        ("alta", "alta") => true,
        _ => false,
    }
}

fn shader_slugs_for_tier(tier: &str) -> Vec<&'static str> {
    // baja: solo livianos | media: baja+media | alta: todos
    match tier {
        "alta" => {
            let mut v = Vec::new();
            v.extend_from_slice(SHADER_LOW);
            v.extend_from_slice(SHADER_MID);
            v.extend_from_slice(SHADER_HIGH);
            v
        }
        "media" => {
            let mut v = Vec::new();
            v.extend_from_slice(SHADER_LOW);
            v.extend_from_slice(SHADER_MID);
            v
        }
        _ => SHADER_LOW.to_vec(),
    }
}

fn default_shader_for_tier(tier: &str, backend: &str) -> &'static str {
    if backend == "optifine" {
        return match tier {
            "alta" | "media" => "bsl-shaders",
            _ => "sildurs-basic-shaders",
        };
    }
    match tier {
        "alta" | "media" => "solas-shader",
        _ => "makeup-ultra-fast-shaders",
    }
}

/// Nombres de packs Iris/modernos que no sirven con OptiFine 1.8.9 / 1.12.2.
const IRIS_ONLY_SHADER_NEEDLES: &[&str] = &[
    "complementary",
    "reimagined",
    "unbound",
    "solas",
    "rethinking",
    "photon",
    "insanity",
    "hysteria",
    "truelight",
    "makeup",
    "potato",
    "essentials-shader",
    "lite-shader",
    "super-duper",
    "miniature",
    "blocky-shader",
    "vanilla-plus",
    "daybreak",
    "mellow",
];

fn purge_non_optifine_shaderpacks(instance_dir: &Path) {
    let dir = instance_dir.join("shaderpacks");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !(name.ends_with(".zip") || name.ends_with(".zip.disabled")) {
            continue;
        }
        // Conservar clásicos OptiFine + hosted Keo.
        let keep = name.contains("sildur")
            || name.contains("bsl")
            || name.contains("chocapic")
            || name.contains("skygleam")
            || name.contains("solar");
        if keep {
            continue;
        }
        if IRIS_ONLY_SHADER_NEEDLES
            .iter()
            .any(|n| name.contains(n))
        {
            let _ = std::fs::remove_file(&path);
            eprintln!(
                "[optimized] purged Iris/modern shader incompatible with OptiFine: {}",
                path.display()
            );
        }
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

fn shader_tier_label(slug: &str) -> &'static str {
    if SHADER_HIGH.contains(&slug) {
        "alta (~40–70% FPS vs vanilla)"
    } else if SHADER_MID.contains(&slug) {
        "media (~60–90% FPS vs vanilla)"
    } else {
        "baja (~85–100% FPS vs vanilla)"
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
    let mut catalog = Vec::new();

    for slug in shader_slugs_for_tier_and_backend(tier, loaders) {
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
        // Sin fallback any-MC: evita bajar shaders de 1.21 en 1.18.2 / OptiFine viejo.
        let Some((url, fname, sha1)) = got else {
            eprintln!("[optimized] skip shader {slug}@{mc} (sin build compatible)");
            continue;
        };
        catalog.push(format!(
            "- {fname}  [{slug}]  → gama {}\n",
            shader_tier_label(slug)
        ));
        let dest = dir.join(&fname);
        if dest.is_file() {
            continue;
        }
        items.push(DownloadItem::new(url, dest).with_sha1(sha1));
    }

    // Extras Keo (Chocapic / Skygleam / SOLAR) solo con OptiFine legacy.
    // Fabric / NeoForge+Oculus usan el catálogo Iris de Modrinth.
    let use_hosted = loaders.iter().any(|l| *l == "optifine");
    if use_hosted {
        for hs in HOSTED_SHADERS {
            if !tier_allows_hosted(tier, hs.min_tier) {
                continue;
            }
            catalog.push(format!(
                "- {}  [{}]  → {}\n",
                hs.filename, hs.id, hs.label
            ));
            let dest = dir.join(hs.filename);
            if dest.is_file() {
                if let Ok(bytes) = std::fs::read(&dest) {
                    if sha256_hex_str(&bytes).eq_ignore_ascii_case(hs.sha256) {
                        continue;
                    }
                }
                let _ = std::fs::remove_file(&dest);
            }
            let mut ok = false;
            let mut last_err = None;
            for url in hosted_shader_urls(hs.filename) {
                match net::download_all(
                    client,
                    vec![DownloadItem::new(url, dest.clone())],
                    1,
                    app,
                    "optimized-hosted-shader",
                    hs.filename,
                )
                .await
                {
                    Ok(()) => {
                        if let Ok(bytes) = std::fs::read(&dest) {
                            if sha256_hex_str(&bytes).eq_ignore_ascii_case(hs.sha256) {
                                ok = true;
                                break;
                            }
                            let _ = std::fs::remove_file(&dest);
                            last_err = Some(format!("SHA-256 mismatch para {}", hs.filename));
                        } else {
                            ok = true;
                            break;
                        }
                    }
                    Err(e) => last_err = Some(e.to_string()),
                }
            }
            if !ok {
                eprintln!(
                    "[optimized] hosted shader {}: {}",
                    hs.filename,
                    last_err.unwrap_or_else(|| "falló".into())
                );
            }
        }
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

    if (tier == "alta" || tier == "media") && !catalog.is_empty() {
        let note = format!(
            "Paraguacraft Optimized — shaders descargados (DESACTIVADOS por defecto)\n\
             Gama PC: {tier}\n\
             Activá el que prefieras en Opciones → Video → Shaders.\n\n\
             Estimación relativa de rendimiento (orientativa):\n{}",
            catalog.join("")
        );
        let _ = std::fs::write(dir.join("PARAGUACRAFT_SHADERS.txt"), note);
    }
    let _ = mc;
    Ok(())
}

fn sha256_hex_str(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn apply_preconfig_once(instance_dir: &Path, tier: &str, backend: &str, mc: &str) -> AppResult<()> {
    let marker = instance_dir.join(TUNED_MARKER);
    if marker.is_file() {
        if let Ok(text) = std::fs::read_to_string(&marker) {
            if text.contains(TUNED_VERSION) {
                return Ok(());
            }
        }
    }

    // options + configs de mods (More Culling, Sodium Extra, BRD, etc.) por gama.
    let _ = performance::optimize_optimized_options(instance_dir, tier, mc);
    let _ = performance::apply_optimized_mod_configs(instance_dir, tier);
    if backend == "optifine" {
        write_optifine_optionsof(instance_dir, tier)?;
    }
    write_shader_default(instance_dir, tier, backend)?;

    let _ = std::fs::write(
        &marker,
        format!("version={TUNED_VERSION}\ntier={tier}\nbackend={backend}\nmc={mc}\n"),
    );
    Ok(())
}

fn write_optifine_optionsof(instance_dir: &Path, tier: &str) -> AppResult<()> {
    let path = instance_dir.join("optionsof.txt");
    let pairs: &[(&str, &str)] = match tier {
        "alta" => &[
            ("ofFastRender", "false"),
            ("ofFastMath", "true"),
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
            ("ofAnimatedWater", "0"),
            ("ofShowFps", "true"),
            ("ofLazyChunkLoading", "true"),
        ],
        "media" => &[
            ("ofFastRender", "true"),
            ("ofFastMath", "true"),
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
            ("ofAnimatedWater", "1"),
            ("ofShowFps", "true"),
            ("ofLazyChunkLoading", "true"),
        ],
        _ => &[
            ("ofFastRender", "true"),
            ("ofFastMath", "true"),
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
            ("ofAnimatedWater", "2"),
            ("ofAnimatedFire", "false"),
            ("ofAnimatedExplosion", "false"),
            ("ofShowFps", "true"),
            ("ofLazyChunkLoading", "true"),
            ("ofChunkLoading", "1"),
        ],
    };
    patch_kv_file(&path, pairs)
}

fn write_shader_default(instance_dir: &Path, tier: &str, backend: &str) -> AppResult<()> {
    // Shaders DESCARGADOS pero DESACTIVADOS por defecto (mejor FPS out-of-box).
    let want_slug = default_shader_for_tier(tier, backend);
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
