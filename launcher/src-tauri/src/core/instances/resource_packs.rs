//! Sincroniza resource packs con `options.txt` (no solo renombrar archivos).

use std::path::Path;

use crate::core::branding::{is_system_pack, parse_quoted_packs};
use crate::error::AppResult;

fn pack_token(name: &str, is_dir: bool, mc_major: u32) -> String {
    let base = name.trim_end_matches(".disabled");
    if mc_major >= 13 {
        if base.ends_with(".zip") || is_dir {
            format!("file/{base}")
        } else {
            format!("file/{base}.zip")
        }
    } else {
        base.trim_end_matches(".zip").to_string()
    }
}

fn mc_major(mc: &str) -> u32 {
    mc.split('.').nth(1).and_then(|p| p.parse().ok()).unwrap_or(21)
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
            if !packs.iter().any(|p| p == &token) {
                packs.push(token);
            }
        } else {
            packs.retain(|p| p != &token && !is_system_pack(p));
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
            if !packs.iter().any(|p| p == &token) {
                packs.push(token);
            }
        } else {
            packs.retain(|p| p != &token && p != "vanilla" && !is_system_pack(p));
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

/// Activa el stack PvP: vanilla (base) → brand Paraguacraft → pack oficial (máxima prioridad).
pub fn set_pvp_stack(game_dir: &Path, mc_version: &str, pack_name: &str) -> AppResult<()> {
    use crate::core::branding::PACK_NAME as BRAND_PACK;

    let major = mc_major(mc_version);
    let official_token = pack_token(pack_name, false, major);
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
        let packs = vec![official_token.clone()];
        lines.retain(|l| !l.starts_with("resourcePacks:"));
        let quoted: Vec<String> = packs.iter().map(|p| format!("\"{p}\"")).collect();
        lines.push(format!("resourcePacks:[{}]", quoted.join(",")));
    } else {
        let brand_dir = game_dir.join("resourcepacks").join(BRAND_PACK);
        let brand_token = pack_token(BRAND_PACK, brand_dir.is_dir(), major);
        let mut packs: Vec<String> = vec!["vanilla".into()];
        if brand_dir.is_dir() {
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

/// Activa un pack como principal: quita otros `file/` no oficiales y deja vanilla + sistema + el pack pedido.
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
            return parse_quoted_packs(line).iter().any(|p| p == &token);
        }
    }
    false
}
