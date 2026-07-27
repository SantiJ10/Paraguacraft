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

/// Mods Fabric alineados a Keo Optimized 1.21.11 (los que no existan en otra MC se saltan).
const FABRIC_MODS: &[&str] = &[
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
    "krypton",
    "modernfix-mvus",
    "clumps",
    "lmd",
    "noisiumforked",
    "particle-core",
    "smooth-boot",
    "cloth-config",
    "fastquit",
    "scalablelux",
    "better-block-entities",
    "c2me-fabric",
    "fpsdisplay",
    "fzzy-config",
    "renderscale",
    "better-render-distance",
    "voxy",
    "almanac",
    "placeholder-api",
];

/// Si el slug primario no tiene build para esa MC, probar alternativas.
fn mod_slug_alternates(slug: &str) -> &'static [&'static str] {
    match slug {
        "modernfix-mvus" => &["modernfix"],
        "noisiumforked" => &["noisium"],
        _ => &[],
    }
}

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
    if is_optifine_mc(mc) {
        return optifine::versions(client, mc).await;
    }
    if is_fabric_mc(mc) {
        return fabric::versions(client, mc).await;
    }
    Ok(vec![])
}

pub async fn versions_neoforge(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    if is_neoforge_mc(mc) {
        return neoforge::versions(client, mc).await;
    }
    Ok(vec![])
}

pub async fn install(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader_version: &str,
) -> AppResult<String> {
    if is_optifine_mc(mc) {
        return optifine::install(app, client, mc, loader_version).await;
    }
    if is_fabric_mc(mc) {
        return fabric::install(app, client, mc, loader_version).await;
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
    neoforge::install(app, client, mc, loader_version).await
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
        install_mod_slugs_with_fallback(
            app,
            client,
            mc,
            &["neoforge", "forge"],
            NEOFORGE_MODS,
            instance_dir,
        )
        .await?;
        install_shaders_for_tier(app, client, mc, &["iris", "optifine"], &tier, instance_dir)
            .await?;
        apply_preconfig_once(instance_dir, &tier, "neoforge")?;
    } else if is_fabric_mc(mc) {
        install_mod_slugs(app, client, mc, "fabric", FABRIC_MODS, instance_dir).await?;
        install_shaders_for_tier(app, client, mc, &["iris", "optifine"], &tier, instance_dir)
            .await?;
        apply_preconfig_once(instance_dir, &tier, "fabric")?;
    } else if is_optifine_mc(mc) {
        // 1.8.9 / 1.12.2: solo shaders con loader OptiFine (varios por gama).
        install_shaders_for_tier(app, client, mc, &["optifine"], &tier, instance_dir).await?;
        apply_preconfig_once(instance_dir, &tier, "optifine")?;
    }

    Ok(())
}

fn shader_slugs_for_tier(tier: &str) -> &'static [&'static str] {
    match tier {
        "alta" => &[
            "complementary-unbound",
            "complementary-reimagined",
            "rethinking-voxels",
            "bsl-shaders",
            "solas-shader",
            "makeup-ultra-fast-shaders",
        ],
        "media" => &[
            "complementary-reimagined",
            "bsl-shaders",
            "solas-shader",
            "makeup-ultra-fast-shaders",
            "super-duper-vanilla",
        ],
        _ => &[
            "makeup-ultra-fast-shaders",
            "super-duper-vanilla",
            "lite-shaders",
            "bsl-shaders",
        ],
    }
}

fn default_shader_for_tier(tier: &str) -> &'static str {
    match tier {
        "alta" => "complementary-unbound",
        "media" => "complementary-reimagined",
        _ => "makeup-ultra-fast-shaders",
    }
}

async fn install_mod_slugs(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader: &str,
    slugs: &[&str],
    instance_dir: &Path,
) -> AppResult<()> {
    install_mod_slugs_with_fallback(app, client, mc, &[loader], slugs, instance_dir).await
}

async fn install_mod_slugs_with_fallback(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loaders: &[&str],
    slugs: &[&str],
    instance_dir: &Path,
) -> AppResult<()> {
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let mut items = Vec::new();
    let mut seen_names = std::collections::HashSet::new();
    for slug in slugs {
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
        return Ok(());
    }

    // options.txt genérico por gama (no el preset PvP).
    let _ = performance::optimize_instance_options(instance_dir);
    let _ = performance::apply_optimized_mod_configs(instance_dir, tier);
    if backend == "optifine" {
        write_optifine_optionsof(instance_dir, tier)?;
    }
    write_shader_default(instance_dir, tier, backend)?;

    let _ = std::fs::write(&marker, format!("tier={tier}\nbackend={backend}\n"));
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
    let want_slug = default_shader_for_tier(tier);
    let pack_name = find_downloaded_shader_name(instance_dir, want_slug)
        .or_else(|| first_shaderpack_name(instance_dir));

    let Some(pack) = pack_name else {
        return Ok(());
    };

    if backend == "optifine" {
        let path = instance_dir.join("optionsof.txt");
        patch_kv_file(&path, &[("ofShaderPack", &pack)])?;
        let options = instance_dir.join("options.txt");
        let mut map = HashMap::new();
        map.insert("shaderPack".into(), pack);
        let _ = performance::merge_options_keys(&options, map);
    } else {
        let config = instance_dir.join("config");
        std::fs::create_dir_all(&config)?;
        let iris = config.join("iris.properties");
        patch_kv_file(
            &iris,
            &[("shaderPack", &pack), ("enableShaders", "true")],
        )?;
        let oculus = config.join("oculus.properties");
        let _ = patch_kv_file(
            &oculus,
            &[("shaderPack", &pack), ("enableShaders", "true")],
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
