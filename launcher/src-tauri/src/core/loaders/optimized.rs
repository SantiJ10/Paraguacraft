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
const TUNED_VERSION: &str = "v3";
/// Única versión del pack Optimized que ve el usuario (el loader real se resuelve solo).
pub const PACK_VERSION: &str = "1.0.0";

/// Pins estables 1.21.11: Sodium 0.8.12 + Iris 1.10.7 (0.8.13 rompe Iris ≤1.10.7).
struct Pin {
    slug: &'static str,
    version_id: &'static str,
}

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
];

/// Mods Fabric base (todas las MCs Fabric soportadas).
const FABRIC_MODS_CORE: &[&str] = &[
    "fabric-api",
    "fabric-language-kotlin",
    "sodium",
    "iris",
    "lithium",
    "ferrite-core",
    "entityculling",
    "immediatelyfast",
    "modmenu",
    "moreculling",
    "sodium-extra",
    "reeses-sodium-options",
    "cloth-config",
    "clumps",
    "lmd",
    "particle-core",
    "fastquit",
    "scalablelux",
    "fzzy-config",
];

/// Extras 1.21.11 (sin Voxy ni C2ME: rompen Sodium/Java).
const FABRIC_MODS_12111: &[&str] = &[
    "krypton",
    "modernfix-mvus",
    "noisiumforked",
    "smooth-boot",
    "better-block-entities",
    "fpsdisplay",
    "renderscale",
    "almanac",
    "placeholder-api",
];

/// Keo 26.2 extras.
const FABRIC_MODS_26_2: &[&str] = &[
    "modernfix",
    "bbe",
    "badoptimizations",
    "asynclogger",
    "ixeris",
    "gnetum",
    "zfastnoise",
    "fism",
    "forge-config-api-port",
    "get-it-together-drops",
    "noxesium",
    "almanac",
];

/// Extras 1.18.2 / 1.20.1.
const FABRIC_MODS_LEGACY: &[&str] = &[
    "krypton",
    "modernfix",
    "noisium",
    "smooth-boot",
    "dynamic-fps",
    "indium",
];

/// JARs a purgar siempre (incompatibles / Java 25+).
const PURGE_NAME_NEEDLES: &[&str] = &[
    "voxy",
    "c2me",
    "concurrentchunkmanagement",
];

/// Mods NeoForge/Forge 1.20.1 alineados a Keo Optimized (Forge) + Oculus.
const NEOFORGE_MODS: &[&str] = &[
    "embeddium",
    "oculus",
    "modernfix",
    "ferrite-core",
    "entityculling",
    "immediatelyfast",
    "clumps",
    "noisium",
    "cull-less-leaves-reforged",
    "memoryleakfix",
    "smooth-boot-reloaded",
    "cloth-config",
    "chloride",
    "starlight-forge",
    "fastload",
    "kotlin-for-forge",
    "lmd",
    "sodium-options-api",
    "ai-improvements",
    "alternate-current",
    "log-begone",
    "carbon-config",
    "lmft",
    "get-it-together-drops",
    "immersive-optimization",
];

fn fabric_mods_for_mc(mc: &str) -> Vec<&'static str> {
    let mut mods: Vec<&'static str> = FABRIC_MODS_CORE.to_vec();
    let extra: &[&str] = match mc {
        "26.2" => FABRIC_MODS_26_2,
        "1.21.11" => FABRIC_MODS_12111,
        _ => FABRIC_MODS_LEGACY,
    };
    mods.extend_from_slice(extra);
    mods
}

/// Si el slug primario no tiene build para esa MC, probar alternativas.
fn mod_slug_alternates(slug: &str) -> &'static [&'static str] {
    match slug {
        "modernfix-mvus" => &["modernfix"],
        "noisiumforked" => &["noisium"],
        "bbe" => &["better-block-entities"],
        "better-block-entities" => &["bbe"],
        _ => &[],
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
        install_mod_slugs_with_fallback(
            app,
            client,
            mc,
            &["neoforge", "forge"],
            NEOFORGE_MODS,
            instance_dir,
            None,
        )
        .await?;
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
    let mods = fabric_mods_for_mc(mc);
    let mut forced: HashMap<String, String> = HashMap::new();

    if mc == "1.21.11" {
        // Pins conocidos compatibles entre sí.
        install_pinned_mods(app, client, PINS_1_21_11, instance_dir).await?;
        for pin in PINS_1_21_11 {
            forced.insert(pin.slug.to_string(), pin.version_id.to_string());
        }
    } else if let Ok(Some(sodium_vid)) = resolve_sodium_pin_from_iris(client, mc).await {
        forced.insert("sodium".into(), sodium_vid);
    }

    let remaining: Vec<&str> = mods
        .into_iter()
        .filter(|s| !forced.contains_key(*s))
        .collect();
    install_mod_slugs_with_fallback(
        app,
        client,
        mc,
        &["fabric"],
        &remaining,
        instance_dir,
        Some(&forced),
    )
    .await?;

    // Asegurar Sodium pineado si Iris lo exige y aún no está el pin en forced path.
    if let Some(sodium_vid) = forced.get("sodium") {
        let _ = install_modrinth_version(app, client, "sodium", sodium_vid, instance_dir).await;
    }
    Ok(())
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

async fn install_modrinth_version(
    app: &AppHandle,
    client: &reqwest::Client,
    slug: &str,
    version_id: &str,
    instance_dir: &Path,
) -> AppResult<()> {
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let (url, fname, sha1) = resolve_modrinth_version_file(client, version_id).await?;
    purge_slug_jars(&mods_dir, slug, &fname);
    let dest = mods_dir.join(&fname);
    if dest.is_file() {
        return Ok(());
    }
    net::download_all(
        client,
        vec![DownloadItem::new(url, dest).with_sha1(sha1)],
        1,
        app,
        "optimized-pin",
        &format!("Optimized {slug}"),
    )
    .await
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
            "ferrite-core" => name.contains("ferritecore"),
            "reeses-sodium-options" => name.contains("reeses-sodium-options"),
            "sodium-extra" => name.contains("sodium-extra"),
            "fabric-api" => name.contains("fabric-api"),
            other => name.contains(other) || name.contains(&other.replace('-', "")),
        };
        if matches && name.trim_end_matches(".disabled") != keep {
            let _ = std::fs::remove_file(path);
        }
    }
}

async fn resolve_sodium_pin_from_iris(
    client: &reqwest::Client,
    mc: &str,
) -> AppResult<Option<String>> {
    let url = format!(
        "{MODRINTH}/project/iris/version?loaders={}&game_versions={}",
        net::url_encode(r#"["fabric"]"#),
        net::url_encode(&format!(r#"["{mc}"]"#))
    );
    let versions: Value = net::fetch_json(client, &url).await?;
    let arr = versions.as_array().cloned().unwrap_or_default();
    let sodium_pid = "AANobbMI";
    for ver in &arr {
        if let Some(deps) = ver["dependencies"].as_array() {
            for dep in deps {
                if dep["project_id"].as_str() == Some(sodium_pid) {
                    if let Some(vid) = dep["version_id"].as_str() {
                        if !vid.is_empty() {
                            return Ok(Some(vid.to_string()));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

fn shader_slugs_for_tier(tier: &str) -> &'static [&'static str] {
    // Varias alternativas por gama. Defaults de rendimiento (Solas/MakeUp), no Unbound.
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
    // Defaults tipo Keo: packs livianos/medios. Unbound solo como alternativa.
    match tier {
        "alta" => "solas-shader",
        "media" => "solas-shader",
        _ => "makeup-ultra-fast-shaders",
    }
}

fn iris_shadow_distance(tier: &str) -> &'static str {
    match tier {
        "alta" => "16",
        "media" => "12",
        _ => "8",
    }
}

async fn install_mod_slugs_with_fallback(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loaders: &[&str],
    slugs: &[&str],
    instance_dir: &Path,
    forced: Option<&HashMap<String, String>>,
) -> AppResult<()> {
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let mut items = Vec::new();
    let mut seen_names = std::collections::HashSet::new();
    for slug in slugs {
        if let Some(map) = forced {
            if let Some(vid) = map.get(*slug) {
                match resolve_modrinth_version_file(client, vid).await {
                    Ok((url, fname, sha1)) => {
                        purge_slug_jars(&mods_dir, slug, &fname);
                        if seen_names.insert(fname.clone()) {
                            let dest = mods_dir.join(&fname);
                            if !dest.is_file() {
                                items.push(DownloadItem::new(url, dest).with_sha1(sha1));
                            }
                        }
                    }
                    Err(e) => eprintln!("[optimized] forced {slug}: {e}"),
                }
                continue;
            }
        }
        let mut candidates = vec![*slug];
        candidates.extend(mod_slug_alternates(slug).iter().copied());
        let mut got = None;
        'outer: for candidate in candidates {
            for loader in loaders {
                match resolve_modrinth_file(client, candidate, mc, loader).await {
                    Ok(v) => {
                        got = Some(v);
                        break 'outer;
                    }
                    Err(_) => continue,
                }
            }
        }
        let Some((url, fname, sha1)) = got else {
            eprintln!("[optimized] skip mod {slug}@{mc}");
            continue;
        };
        if !seen_names.insert(fname.clone()) {
            continue;
        }
        purge_slug_jars(&mods_dir, slug, &fname);
        let dest = mods_dir.join(&fname);
        if dest.is_file() {
            continue;
        }
        items.push(DownloadItem::new(url, dest).with_sha1(sha1));
    }
    if !items.is_empty() {
        net::download_all(
            client,
            items,
            8,
            app,
            "optimized-mods",
            &format!("Paraguacraft Optimized mods ({mc})"),
        )
        .await?;
    }
    Ok(())
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
