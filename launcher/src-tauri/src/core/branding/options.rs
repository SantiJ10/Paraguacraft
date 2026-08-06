//! Sincronización de `options.txt` — fuerza Brand Pack activo en cada lanzamiento.

use std::fs;
use std::path::Path;

use super::version::{McVersion, PackProfile};
use crate::error::AppResult;

const PACK_NAME: &str = "ParaguacraftBrandPack";
const OFFICIAL_189: &str = "paraguacraft-pvp-189.zip";
const OFFICIAL_MODERN: &str = "paraguacraft-pvp-modern.zip";

pub fn is_system_pack(inner: &str) -> bool {
    inner == "vanilla"
        || inner.starts_with("file/ParaguacraftBrandPack")
        || inner.starts_with("file/Pack_Graficos_Minimos")
        || inner.starts_with("file/paraguacraft-pvp")
        || inner == "ParaguacraftBrandPack"
        || inner.starts_with("Pack_Graficos_Minimos")
        || inner.contains("paraguacraft-pvp-189")
        || inner.contains("paraguacraft-pvp-modern")
}

pub fn is_official_pvp_pack(inner: &str) -> bool {
    inner.contains("paraguacraft-pvp-189") || inner.contains("paraguacraft-pvp-modern")
}

pub fn parse_quoted_packs(line: &str) -> Vec<String> {
    // Acepta "resourcePacks:[\"a\",\"b\"]" o sólo el cuerpo entre corchetes.
    let body = line
        .split_once(':')
        .map(|(_, rest)| rest)
        .unwrap_or(line);
    let body = body.trim().trim_start_matches('[').trim_end_matches(']');
    let mut packs = Vec::new();
    let mut chars = body.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c == '"' {
            chars.next();
            let mut inner = String::new();
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    if let Some(next) = chars.next() {
                        inner.push(next);
                    }
                } else if ch == '"' {
                    break;
                } else {
                    inner.push(ch);
                }
            }
            packs.push(inner);
        } else {
            chars.next();
        }
    }
    packs
}

fn official_token_for_dir(game_dir: &Path, major: u32) -> Option<String> {
    let packs = game_dir.join("resourcepacks");
    if packs.join(OFFICIAL_189).is_file() {
        if major >= 13 {
            return Some(format!("file/{OFFICIAL_189}"));
        }
        return Some(OFFICIAL_189.into());
    }
    if packs.join(OFFICIAL_MODERN).is_file() {
        if major >= 13 {
            return Some(format!("file/{OFFICIAL_MODERN}"));
        }
        return Some(OFFICIAL_MODERN.into());
    }
    None
}

pub fn ensure_enabled(game_dir: &Path, ver: McVersion, profile: PackProfile, min_graphics: bool) -> AppResult<()> {
    let options_path = game_dir.join("options.txt");
    let lines: Vec<String> = if options_path.is_file() {
        fs::read_to_string(&options_path)?
            .lines()
            .map(String::from)
            .collect()
    } else {
        Vec::new()
    };

    let mut new_lines = lines;

    if profile == PackProfile::Classic {
        new_lines.retain(|l| !l.starts_with("texturepack:"));
        new_lines.push(format!("texturepack:{PACK_NAME}.zip"));
    } else if ver.major < 13 {
        let existing: Vec<String> = new_lines
            .iter()
            .find(|l| l.starts_with("resourcePacks:"))
            .map(|l| parse_quoted_packs(l))
            .unwrap_or_default();
        let user_packs: Vec<String> = existing
            .iter()
            .filter(|p| !is_system_pack(p))
            .cloned()
            .collect();

        // 1.8.9: última entrada = mayor prioridad al cargar texturas.
        // Brand primero, oficial PvP al final (mismas rules que el cliente).
        let mut packs: Vec<String> = Vec::new();
        packs.push(format!("\"{PACK_NAME}\""));
        if min_graphics {
            packs.push("\"Pack_Graficos_Minimos.zip\"".to_string());
        }
        for p in user_packs {
            packs.push(format!("\"{p}\""));
        }
        if let Some(official) = official_token_for_dir(game_dir, ver.major) {
            packs.push(format!("\"{official}\""));
        } else {
            for p in existing.iter().filter(|p| is_official_pvp_pack(p)) {
                packs.push(format!("\"{p}\""));
            }
        }
        new_lines.retain(|l| !l.starts_with("resourcePacks:"));
        new_lines.push(format!("resourcePacks:[{}]", packs.join(",")));
    } else {
        let existing: Vec<String> = new_lines
            .iter()
            .find(|l| l.starts_with("resourcePacks:"))
            .map(|l| parse_quoted_packs(l))
            .unwrap_or_default();
        let user_packs: Vec<String> = existing
            .iter()
            .filter(|p| !is_system_pack(p))
            .map(|p| format!("\"{p}\""))
            .collect();

        let brand = if profile.uses_zip_file() {
            format!("\"file/{PACK_NAME}.zip\"")
        } else {
            format!("\"file/{PACK_NAME}\"")
        };
        let mut packs = vec!["\"vanilla\"".to_string(), brand];
        if min_graphics {
            packs.push("\"file/Pack_Graficos_Minimos.zip\"".to_string());
        }
        if let Some(official) = official_token_for_dir(game_dir, ver.major) {
            packs.push(format!("\"{official}\""));
        } else {
            for p in existing.iter().filter(|p| is_official_pvp_pack(p)) {
                packs.push(format!("\"{p}\""));
            }
        }
        packs.extend(user_packs);
        new_lines.retain(|l| !l.starts_with("resourcePacks:"));
        new_lines.push(format!("resourcePacks:[{}]", packs.join(",")));
    }

    if let Some(parent) = options_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(options_path, format!("{}\n", new_lines.join("\n")))?;
    Ok(())
}
