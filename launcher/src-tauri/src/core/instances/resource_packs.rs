//! Sincroniza resource packs con `options.txt` (no solo renombrar archivos).

use std::path::Path;

use crate::core::branding::{is_system_pack, parse_quoted_packs, PACK_NAME as BRAND_PACK};
use crate::error::AppResult;

/// Token en options.txt:
/// - MC 1.13+: `file/nombre.zip` o `file/carpeta`
/// - MC 1.8–1.12: nombre del zip **con** `.zip` o nombre de carpeta (sin `file/`).
fn pack_token(name: &str, is_dir: bool, mc_major: u32) -> String {
    let base = name.trim_end_matches(".disabled");
    if mc_major >= 13 {
        if base.starts_with("file/") {
            return base.to_string();
        }
        if base.ends_with(".zip") || is_dir {
            format!("file/{base}")
        } else {
            format!("file/{base}.zip")
        }
    } else if is_dir {
        base.trim_end_matches('/').to_string()
    } else if base.ends_with(".zip") {
        base.to_string()
    } else {
        base.to_string()
    }
}

fn mc_major(mc: &str) -> u32 {
    mc.split('.').nth(1).and_then(|p| p.parse().ok()).unwrap_or(21)
}

fn pack_token_matches(entry: &str, expected: &str) -> bool {
    if entry == expected {
        return true;
    }
    let e = entry.strip_prefix("file/").unwrap_or(entry);
    let t = expected.strip_prefix("file/").unwrap_or(expected);
    e.eq_ignore_ascii_case(t)
        || e.trim_end_matches(".zip")
            .eq_ignore_ascii_case(t.trim_end_matches(".zip"))
}

/// Activa/desactiva un pack en `options.txt` según la versión de MC.
pub fn set_enabled(
    game_dir: &Path,
    mc_version: &str,
    pack_name: &str,
    is_dir: bool,
    enabled: bool,
) -> AppResult<()> {
    let major = mc_major(mc_version);
    let token = pack_token(pack_name, is_dir, major);
    let options_path = game_dir.join("options.txt");
    let mut lines: Vec<String> = if options_path.is_file() {
        std::fs::read_to_string(&options_path)?
            .lines()
            .map(String::from)
            .collect()
    } else {
        Vec::new()
    };

    if major < 6 {
        if enabled {
            lines.retain(|l| !l.starts_with("texturepack:"));
            lines.push(format!("texturepack:{token}"));
        } else {
            lines.retain(|l| !l.starts_with("texturepack:") || !l.contains(&token));
        }
    } else if major < 13 {
        let mut packs: Vec<String> = lines
            .iter()
            .find(|l| l.starts_with("resourcePacks:"))
            .map(|l| parse_quoted_packs(l))
            .unwrap_or_default();
        if enabled {
            if !packs.iter().any(|p| pack_token_matches(p, &token)) {
                packs.insert(0, token);
            }
        } else {
            packs.retain(|p| !pack_token_matches(p, &token) && !is_system_pack(p));
        }
        lines.retain(|l| !l.starts_with("resourcePacks:"));
        if !packs.is_empty() {
            let quoted: Vec<String> = packs.iter().map(|p| format!("\"{p}\"")).collect();
            lines.push(format!("resourcePacks:[{}]", quoted.join(",")));
        }
    } else {
        let mut packs: Vec<String> = lines
            .iter()
            .find(|l| l.starts_with("resourcePacks:"))
            .map(|l| parse_quoted_packs(l))
            .unwrap_or_default();
        if packs.is_empty() {
            packs.push("vanilla".into());
        }
        if enabled {
            if !packs.iter().any(|p| pack_token_matches(p, &token)) {
                packs.push(token);
            }
        } else {
            packs.retain(|p| !pack_token_matches(p, &token) && p != "vanilla" && !is_system_pack(p));
        }
        if !packs.iter().any(|p| p == "vanilla") {
            packs.insert(0, "vanilla".into());
        }
        lines.retain(|l| !l.starts_with("resourcePacks:"));
        let quoted: Vec<String> = packs.iter().map(|p| format!("\"{p}\"")).collect();
        lines.push(format!("resourcePacks:[{}]", quoted.join(",")));
    }

    if let Some(parent) = options_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&options_path, format!("{}\n", lines.join("\n")))?;
    Ok(())
}

/// Activa el stack PvP: pack oficial (máx. prioridad) + brand (skins offline) si existe.
pub fn set_pvp_stack(game_dir: &Path, mc_version: &str, pack_name: &str) -> AppResult<()> {
    let major = mc_major(mc_version);
    let official_token = pack_token(pack_name, false, major);
    let brand_dir = game_dir.join("resourcepacks").join(BRAND_PACK);
    let brand_exists = brand_dir.is_dir()
        || game_dir
            .join("resourcepacks")
            .join(format!("{BRAND_PACK}.zip"))
            .is_file();
    let options_path = game_dir.join("options.txt");
    let mut lines: Vec<String> = if options_path.is_file() {
        std::fs::read_to_string(&options_path)?
            .lines()
            .map(String::from)
            .collect()
    } else {
        Vec::new()
    };

    if major < 6 {
        lines.retain(|l| !l.starts_with("texturepack:"));
        lines.push(format!("texturepack:{official_token}"));
    } else if major < 13 {
        // 1.8.9: última entrada del listado en options = mayor prioridad al cargar
        // (FallbackResourceManager itera de atrás hacia adelante). Brand primero, oficial al final.
        let mut packs: Vec<String> = Vec::new();
        if brand_exists {
            packs.push(pack_token(BRAND_PACK, brand_dir.is_dir(), major));
        }
        packs.push(official_token);
        if let Some(existing) = lines.iter().find(|l| l.starts_with("resourcePacks:")) {
            for p in parse_quoted_packs(existing) {
                if is_system_pack(&p) {
                    continue;
                }
                if packs.iter().any(|x| pack_token_matches(x, &p)) {
                    continue;
                }
                // Extra packs debajo del oficial (menor prioridad)
                packs.insert(packs.len().saturating_sub(1).max(0), p);
            }
        }
        lines.retain(|l| !l.starts_with("resourcePacks:"));
        let quoted: Vec<String> = packs.iter().map(|p| format!("\"{p}\"")).collect();
        lines.push(format!("resourcePacks:[{}]", quoted.join(",")));
    } else {
        let brand_token = pack_token(BRAND_PACK, brand_dir.is_dir(), major);
        let mut packs: Vec<String> = vec!["vanilla".into()];
        if brand_exists {
            packs.push(brand_token);
        }
        packs.push(official_token);
        lines.retain(|l| !l.starts_with("resourcePacks:"));
        let quoted: Vec<String> = packs.iter().map(|p| format!("\"{p}\"")).collect();
        lines.push(format!("resourcePacks:[{}]", quoted.join(",")));
    }

    if let Some(parent) = options_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&options_path, format!("{}\n", lines.join("\n")))?;
    Ok(())
}

/// Activa un pack como principal del stack PvP.
pub fn set_primary(game_dir: &Path, mc_version: &str, pack_name: &str, is_dir: bool) -> AppResult<()> {
    let _ = is_dir;
    set_pvp_stack(game_dir, mc_version, pack_name)
}

pub fn is_enabled_in_options(game_dir: &Path, mc_version: &str, pack_name: &str, is_dir: bool) -> bool {
    let options_path = game_dir.join("options.txt");
    let Ok(raw) = std::fs::read_to_string(&options_path) else {
        return is_dir;
    };
    let major = mc_major(mc_version);
    let token = pack_token(pack_name, is_dir, major);
    for line in raw.lines() {
        if major < 6 {
            if line.starts_with("texturepack:") && line.contains(&token) {
                return true;
            }
        } else if line.starts_with("resourcePacks:") {
            return parse_quoted_packs(line)
                .iter()
                .any(|p| pack_token_matches(p, &token));
        }
    }
    false
}
