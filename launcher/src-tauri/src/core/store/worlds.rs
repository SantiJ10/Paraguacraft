//! Mundos de CurseForge/Modrinth: ZIP → `saves/<mundo>/level.dat`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

/// 1.21 coincide con 1.21.11 (etiquetado típico de mapas CF).
pub fn mc_flexible_match(instance_mc: &str, want: &str) -> bool {
    if instance_mc == want {
        return true;
    }
    if is_major_minor(want) && instance_mc.starts_with(&format!("{want}.")) {
        return true;
    }
    if is_major_minor(instance_mc) && want.starts_with(&format!("{instance_mc}.")) {
        return true;
    }
    false
}

fn is_major_minor(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 2 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

pub fn finalize_if_world(project_type: &str, saves_dir: &Path, filename: &str) -> AppResult<String> {
    if project_type != "world" && project_type != "worlds" {
        return Ok(filename.to_string());
    }
    let zip = saves_dir.join(filename);
    let fallback = Path::new(filename)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "world".into());
    let dest = install_world_zip(&zip, saves_dir, &fallback)?;
    Ok(dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or(fallback))
}

pub fn install_world_zip(zip_path: &Path, saves_dir: &Path, fallback_name: &str) -> AppResult<PathBuf> {
    if !zip_path.is_file() {
        return Err(AppError::msg("No se descargó el ZIP del mundo."));
    }
    fs::create_dir_all(saves_dir)?;
    let tmp = unique_dir(saves_dir, ".pg-world-tmp");
    fs::create_dir_all(&tmp)?;
    if let Err(e) = extract_zip(zip_path, &tmp) {
        let _ = fs::remove_dir_all(&tmp);
        return Err(e);
    }
    let Some(world_src) = find_world_root(&tmp) else {
        let _ = fs::remove_dir_all(&tmp);
        return Err(AppError::msg(
            "El ZIP no trae level.dat. No es un mundo de Minecraft.",
        ));
    };
    let mut name = if world_src == tmp {
        fallback_name.to_string()
    } else {
        world_src
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| fallback_name.to_string())
    };
    name = sanitize_name(&name);
    if name.is_empty() || name.starts_with('.') {
        name = fallback_name.to_string();
    }
    let dest = unique_dir(saves_dir, &name);
    let result = if world_src == tmp {
        fs::rename(&tmp, &dest)
    } else {
        let r = fs::rename(&world_src, &dest);
        let _ = fs::remove_dir_all(&tmp);
        r
    };
    if let Err(e) = result {
        let _ = fs::remove_dir_all(&tmp);
        return Err(AppError::msg(format!("No se pudo mover el mundo a saves/: {e}")));
    }
    let _ = fs::remove_file(zip_path);
    Ok(dest)
}

fn extract_zip(zip_path: &Path, dest: &Path) -> AppResult<()> {
    let file = fs::File::open(zip_path)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| AppError::msg(format!("ZIP de mundo inválido: {e}")))?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let rel_s = rel.to_string_lossy().replace('\\', "/");
        if rel_s.contains("__MACOSX") || rel_s.ends_with(".DS_Store") {
            continue;
        }
        let out_path = dest.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

fn find_world_root(root: &Path) -> Option<PathBuf> {
    let mut hits = Vec::new();
    collect_level_dat(root, &mut hits, 0);
    hits.iter()
        .find(|p| p.join("region").is_dir() || p.join("db").is_dir())
        .cloned()
        .or_else(|| hits.into_iter().next())
}

fn collect_level_dat(dir: &Path, out: &mut Vec<PathBuf>, depth: u32) {
    if depth > 8 {
        return;
    }
    if depth > 0 {
        let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name == "__MACOSX" || name.starts_with('.') {
            return;
        }
    }
    if dir.join("level.dat").is_file() {
        out.push(dir.to_path_buf());
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_level_dat(&p, out, depth + 1);
        }
    }
}

fn unique_dir(parent: &Path, name: &str) -> PathBuf {
    let dest = parent.join(name);
    if !dest.exists() {
        return dest;
    }
    parent.join(format!(
        "{}-{}",
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(1)
    ))
}

fn sanitize_name(name: &str) -> String {
    let s = name.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_");
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
use zip::write::SimpleFileOptions;
    use std::io::Write;

    fn write_zip(path: &Path, files: &[(&str, &[u8])]) {
        let f = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, data) in files {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn flexible_mc_1_21_matches_1_21_11() {
        assert!(mc_flexible_match("1.21.11", "1.21"));
        assert!(mc_flexible_match("1.21", "1.21.11"));
        assert!(!mc_flexible_match("1.20.1", "1.21"));
        assert!(mc_flexible_match("1.21.11", "1.21.11"));
    }

    #[test]
    fn extracts_nested_world() {
        let tmp = std::env::temp_dir().join(format!("pg-world-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let zip = tmp.join("oneblock.zip");
        write_zip(
            &zip,
            &[
                ("readme.txt", b"hi"),
                ("One Block/level.dat", b"nbt"),
                ("One Block/region/r.0.0.mca", b"chunk"),
            ],
        );
        let saves = tmp.join("saves");
        let dest = install_world_zip(&zip, &saves, "one-block").unwrap();
        assert_eq!(dest.file_name().unwrap(), "One Block");
        assert!(dest.join("level.dat").is_file());
        assert!(!zip.exists());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn extracts_root_level_dat() {
        let tmp = std::env::temp_dir().join(format!("pg-world-root-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let zip = tmp.join("map.zip");
        write_zip(&zip, &[("level.dat", b"nbt"), ("region/r.0.0.mca", b"c")]);
        let saves = tmp.join("saves");
        let dest = install_world_zip(&zip, &saves, "oneblock").unwrap();
        assert_eq!(dest.file_name().unwrap(), "oneblock");
        assert!(dest.join("level.dat").is_file());
        let _ = fs::remove_dir_all(&tmp);
    }
}
