//! Escaneo de instancias de otros launchers (solo lectura).
//!
//! Es un escaneo puntual y barato (lecturas de directorio + algun JSON chico),
//! sin watchers ni hilos: se ejecuta cuando la UI lo pide y vuelve a idle.

use std::collections::HashSet;
use std::path::Path;

use crate::core::paths;
use crate::models::Instance;

/// Todas las instancias: locales (Paraguacraft) + detectadas en otros launchers.
pub fn scan_all() -> Vec<Instance> {
    let mut out = super::list_local();
    out.extend(scan_external());
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
            "lunar" => scan_simple_versions(&root, "lunar", "Lunar", &mut out),
            _ => scan_vanilla_versions(&root, source, &mut out),
        }
    }
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
            mod_count: 0,
        });
    }
}

/// Carpetas-version simples (Lunar multiver): cada subdir es una version.
fn scan_simple_versions(root: &Path, source: &str, label: &str, out: &mut Vec<Instance>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for e in entries.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let ver = e.file_name().to_string_lossy().to_string();
        out.push(Instance {
            id: ext_id(source, &ver),
            name: format!("{label} {ver}"),
            icon: "\u{1F319}".into(),
            mc_version: ver.clone(),
            loader: "vanilla".into(),
            loader_version: String::new(),
            source: source.to_string(),
            last_played: None,
            total_play_minutes: 0,
            ram_mb: 0,
            mod_count: 0,
        });
    }
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
