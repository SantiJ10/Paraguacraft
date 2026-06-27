//! Escaneo de instancias de otros launchers (solo lectura).
//!
//! El resultado se cachea en memoria para evitar re-escaneos en cada instalación
//! desde la tienda. La UI dispara `scan_external()` (async vía `spawn_blocking`).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::core::paths;
use crate::models::Instance;

static EXTERNAL_CACHE: OnceLock<Mutex<Option<Vec<Instance>>>> = OnceLock::new();

fn cache() -> &'static Mutex<Option<Vec<Instance>>> {
    EXTERNAL_CACHE.get_or_init(|| Mutex::new(None))
}

pub fn set_external_cache(instances: Vec<Instance>) {
    *cache().lock().unwrap() = Some(instances);
}

pub fn external_from_cache() -> Option<Vec<Instance>> {
    cache().lock().unwrap().clone()
}

/// Busca una instancia externa por id; refresca el cache si no está.
pub fn find_external(id: &str) -> Option<Instance> {
    if let Some(cached) = external_from_cache() {
        if let Some(i) = cached.iter().find(|i| i.id == id) {
            return Some(i.clone());
        }
    }
    let fresh = scan_external();
    fresh.into_iter().find(|i| i.id == id)
}

/// Todas las instancias: locales (Paraguacraft) + detectadas en otros launchers.
pub fn scan_all() -> Vec<Instance> {
    let mut out = super::list_local();
    out.extend(external_from_cache().unwrap_or_else(scan_external));
    out
}

/// Detecta instancias de Vanilla/Lunar/Prism/TLauncher/SK en modo lectura.
pub fn scan_external() -> Vec<Instance> {
    let mut out = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    for (source, root) in paths::external_launcher_roots() {
        let canon = std::fs::canonicalize(&root)
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_else(|_| root.to_string_lossy().to_lowercase());
        // Evita escanear `.minecraft/versions` varias veces (vanilla/tlauncher/sk).
        if !seen_paths.insert(canon) {
            continue;
        }
        match source {
            "prism" => scan_prism(&root, &mut out),
            "lunar" => scan_lunar(&root, &mut out),
            _ => scan_vanilla_versions(&root, source, &mut out),
        }
    }
    set_external_cache(out.clone());
    out
}

fn ext_id(source: &str, key: &str) -> String {
    format!("ext::{source}::{key}")
}

/// `.minecraft/versions/<ver>` con su `<ver>.json`. Loader inferido del nombre.
fn scan_vanilla_versions(versions_dir: &Path, source: &str, out: &mut Vec<Instance>) {
    let Ok(entries) = std::fs::read_dir(versions_dir) else {
        return;
    };
    for e in entries.flatten() {
        let dir = e.path();
        if !dir.is_dir() {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if !dir.join(format!("{name}.json")).exists() {
            continue;
        }
        let loader = infer_loader(&name);
        let game_dir = paths::default_minecraft_dir();
        out.push(Instance {
            id: ext_id(source, &name),
            name: name.clone(),
            icon: "\u{1F4E6}".into(),
            mc_version: extract_mc_version(&name),
            loader,
            loader_version: String::new(),
            source: source.to_string(),
            last_played: None,
            total_play_minutes: 0,
            ram_mb: 0,
            mod_count: super::count_mods(&game_dir),
        });
    }
}

/// Lunar Client: cada subcarpeta de multiver es una versión MC.
fn scan_lunar(root: &Path, out: &mut Vec<Instance>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for e in entries.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let ver = e.file_name().to_string_lossy().to_string();
        let game_dir = lunar_version_dir(&ver);
        let loader = infer_loader(&ver);
        out.push(Instance {
            id: ext_id("lunar", &ver),
            name: format!("Lunar {ver}"),
            icon: "\u{1F319}".into(),
            mc_version: extract_mc_version(&ver),
            loader,
            loader_version: String::new(),
            source: "lunar".into(),
            last_played: None,
            total_play_minutes: 0,
            ram_mb: 0,
            mod_count: super::count_mods(&game_dir),
        });
    }
}

/// Carpeta de juego efectiva para una versión Lunar (multiver o `.minecraft` compartido).
pub fn lunar_version_dir(version: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let ver_dir = home
        .join(".lunarclient")
        .join("offline")
        .join("multiver")
        .join(version);
    if ver_dir.is_dir() {
        return ver_dir;
    }
    paths::default_minecraft_dir()
}

/// Instancias Prism/PolyMC/MultiMC: `instance.cfg` + `mmc-pack.json`.
fn scan_prism(instances_dir: &Path, out: &mut Vec<Instance>) {
    let Ok(entries) = std::fs::read_dir(instances_dir) else {
        return;
    };
    for e in entries.flatten() {
        let dir = e.path();
        if !dir.is_dir() || !dir.join("instance.cfg").exists() {
            continue;
        }
        let folder = e.file_name().to_string_lossy().to_string();
        let name = read_cfg_value(&dir.join("instance.cfg"), "name").unwrap_or_else(|| folder.clone());
        let (mc_version, loader) = read_mmc_pack(&dir.join("mmc-pack.json"));
        out.push(Instance {
            id: ext_id("prism", &folder),
            name,
            icon: "\u{1F9F1}".into(),
            mc_version,
            loader,
            loader_version: String::new(),
            source: "prism".into(),
            last_played: None,
            total_play_minutes: 0,
            ram_mb: 0,
            mod_count: super::count_mods(&dir.join(".minecraft")),
        });
    }
}

fn read_cfg_value(path: &Path, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(v) = line.strip_prefix(&format!("{key}=")) {
            return Some(v.trim().to_string());
        }
    }
    None
}

fn read_mmc_pack(path: &Path) -> (String, String) {
    let mut mc = String::new();
    let mut loader = "vanilla".to_string();
    if let Some(v) = crate::config::read_json::<serde_json::Value>(path) {
        if let Some(components) = v["components"].as_array() {
            for c in components {
                let uid = c["uid"].as_str().unwrap_or("");
                let version = c["version"].as_str().unwrap_or("").to_string();
                match uid {
                    "net.minecraft" => mc = version,
                    "net.fabricmc.fabric-loader" => loader = "fabric".into(),
                    "org.quiltmc.quilt-loader" => loader = "quilt".into(),
                    "net.minecraftforge" => loader = "forge".into(),
                    "net.neoforged" => loader = "neoforge".into(),
                    _ => {}
                }
            }
        }
    }
    (mc, loader)
}

fn infer_loader(version_name: &str) -> String {
    let low = version_name.to_lowercase();
    if low.contains("neoforge") {
        "neoforge".into()
    } else if low.contains("fabric") {
        "fabric".into()
    } else if low.contains("quilt") {
        "quilt".into()
    } else if low.contains("forge") {
        "forge".into()
    } else if low.contains("optifine") {
        "optifine".into()
    } else {
        "vanilla".into()
    }
}

/// Extrae el primer token tipo `1.20.1` del nombre de la version.
fn extract_mc_version(name: &str) -> String {
    for token in name.split(['-', '_', ' ']) {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() >= 2 && parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok() {
            return token.to_string();
        }
    }
    name.to_string()
}
