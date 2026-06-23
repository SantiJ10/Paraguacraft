//! Sincronización mínima de `options.txt` — solo añade el pack si falta.

use std::fs;
use std::path::Path;

use super::version::{McVersion, PackProfile};
use crate::error::AppResult;

const PACK_NAME: &str = "ParaguacraftBrandPack";

fn is_system_pack(inner: &str) -> bool {
    inner == "vanilla"
        || inner.starts_with("file/ParaguacraftBrandPack")
        || inner.starts_with("file/Pack_Graficos_Minimos")
        || inner == "ParaguacraftBrandPack"
        || inner.starts_with("Pack_Graficos_Minimos")
}

fn brand_tokens(profile: PackProfile) -> Vec<String> {
    match profile {
        PackProfile::Classic => vec![format!("{PACK_NAME}.zip")],
        PackProfile::Modern => vec![format!("file/{PACK_NAME}.zip")],
        PackProfile::Legacy | PackProfile::Standard | PackProfile::StandardRange
        | PackProfile::Wide => vec![format!("file/{PACK_NAME}")],
    }
}

fn parse_quoted_packs(line: &str) -> Vec<String> {
    let mut packs = Vec::new();
    let mut chars = line.chars().peekable();
    while chars.peek() == Some(&'"') {
        chars.next();
        let mut inner = String::new();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    inner.push(next);
                }
            } else if c == '"' {
                break;
            } else {
                inner.push(c);
            }
        }
        packs.push(inner);
    }
    packs
}

fn line_has_brand(line: &str, profile: PackProfile) -> bool {
    if profile == PackProfile::Classic {
        return line.starts_with("texturepack:") && line.contains(PACK_NAME);
    }
    if !line.starts_with("resourcePacks:") {
        return false;
    }
    brand_tokens(profile)
        .iter()
        .any(|t| line.contains(&t.trim_start_matches("file/")))
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

    if lines.iter().any(|l| line_has_brand(l, profile)) {
        return Ok(());
    }

    let mut new_lines = lines;

    if profile == PackProfile::Classic {
        new_lines.retain(|l| !l.starts_with("texturepack:"));
        new_lines.push(format!("texturepack:{PACK_NAME}.zip"));
    } else if ver.major < 13 {
        let mut packs: Vec<String> = new_lines
            .iter()
            .find(|l| l.starts_with("resourcePacks:"))
            .map(|l| {
                parse_quoted_packs(l)
                    .into_iter()
                    .filter(|p| !is_system_pack(p))
                    .map(|p| format!("\"{p}\""))
                    .collect()
            })
            .unwrap_or_default();
        if min_graphics {
            let g = "\"Pack_Graficos_Minimos.zip\"".to_string();
            if !packs.contains(&g) {
                packs.push(g);
            }
        }
        let brand = format!("\"{PACK_NAME}\"");
        if !packs.contains(&brand) {
            packs.push(brand);
        }
        new_lines.retain(|l| !l.starts_with("resourcePacks:"));
        if !packs.is_empty() {
            new_lines.push(format!("resourcePacks:[{}]", packs.join(",")));
        }
    } else {
        let user_packs: Vec<String> = new_lines
            .iter()
            .find(|l| l.starts_with("resourcePacks:"))
            .map(|l| {
                parse_quoted_packs(l)
                    .into_iter()
                    .filter(|p| !is_system_pack(p))
                    .map(|p| format!("\"{p}\""))
                    .collect()
            })
            .unwrap_or_default();

        let mut packs = vec!["\"vanilla\"".to_string()];
        packs.extend(user_packs);
        if min_graphics {
            let g = "\"file/Pack_Graficos_Minimos.zip\"".to_string();
            if !packs.contains(&g) {
                packs.push(g);
            }
        }
        let brand = if profile.uses_zip_file() {
            format!("\"file/{PACK_NAME}.zip\"")
        } else {
            format!("\"file/{PACK_NAME}\"")
        };
        if !packs.contains(&brand) {
            packs.push(brand);
        }
        new_lines.retain(|l| !l.starts_with("resourcePacks:"));
        new_lines.push(format!("resourcePacks:[{}]", packs.join(",")));
    }

    if let Some(parent) = options_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(options_path, format!("{}\n", new_lines.join("\n")))?;
    Ok(())
}
