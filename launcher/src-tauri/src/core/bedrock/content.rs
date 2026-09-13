//! Mundos y packs de `%LOCALAPPDATA%\Packages\...\LocalState\games\com.mojang`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

use super::com_mojang_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockWorld {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockPack {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub path: String,
    pub icon_path: Option<String>,
}

fn mojang() -> AppResult<PathBuf> {
    com_mojang_dir().ok_or_else(|| {
        AppError::msg("No hay carpeta com.mojang. Abrí Bedrock una vez para crearla.")
    })
}

fn read_trimmed(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn folder_icon(dir: &Path) -> Option<String> {
    for name in ["world_icon.jpeg", "world_icon.jpg", "world_icon.png", "pack_icon.png"] {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}

pub fn list_worlds() -> Vec<BedrockWorld> {
    let Ok(root) = mojang() else {
        return Vec::new();
    };
    let worlds = root.join("minecraftWorlds");
    let Ok(entries) = std::fs::read_dir(&worlds) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if id.starts_with('.') {
            continue;
        }
        if !path.join("level.dat").is_file() && !path.join("levelname.txt").is_file() {
            continue;
        }
        let name = read_trimmed(&path.join("levelname.txt")).unwrap_or_else(|| id.clone());
        out.push(BedrockWorld {
            id,
            name,
            path: path.to_string_lossy().to_string(),
            icon_path: folder_icon(&path),
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

fn pack_name(dir: &Path, fallback: &str) -> String {
    let manifest = dir.join("manifest.json");
    if let Ok(raw) = std::fs::read_to_string(&manifest) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(name) = v
                .pointer("/header/name")
                .and_then(|n| n.as_str())
                .filter(|s| !s.is_empty())
            {
                return name.to_string();
            }
        }
    }
    fallback.to_string()
}

fn list_pack_folder(folder: &str, kind: &str) -> Vec<BedrockPack> {
    let Ok(root) = mojang() else {
        return Vec::new();
    };
    let dir = root.join(folder);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if id.starts_with('.') {
            continue;
        }
        if !path.join("manifest.json").is_file() {
            continue;
        }
        out.push(BedrockPack {
            name: pack_name(&path, &id),
            id,
            kind: kind.to_string(),
            path: path.to_string_lossy().to_string(),
            icon_path: folder_icon(&path),
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

pub fn list_packs() -> Vec<BedrockPack> {
    let mut out = list_pack_folder("resource_packs", "resource");
    out.extend(list_pack_folder("behavior_packs", "behavior"));
    out
}

pub fn open_folder(kind: &str) -> AppResult<()> {
    let root = mojang()?;
    let dir = match kind {
        "worlds" | "minecraftWorlds" => root.join("minecraftWorlds"),
        "resource" | "resource_packs" => root.join("resource_packs"),
        "behavior" | "behavior_packs" => root.join("behavior_packs"),
        "root" | "" => root,
        _ => root,
    };
    std::fs::create_dir_all(&dir)?;
    open_in_file_manager(&dir)
}

fn open_in_file_manager(path: &Path) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir la carpeta: {e}")))?;
    }
    Ok(())
}

pub fn delete_world(id: &str) -> AppResult<()> {
    let root = mojang()?.join("minecraftWorlds");
    remove_named_dir(&root, id)
}

pub fn delete_pack(kind: &str, id: &str) -> AppResult<()> {
    let folder = match kind {
        "behavior" => "behavior_packs",
        _ => "resource_packs",
    };
    let root = mojang()?.join(folder);
    remove_named_dir(&root, id)
}

fn remove_named_dir(root: &Path, id: &str) -> AppResult<()> {
    if id.is_empty() || id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err(AppError::msg("Identificador inválido"));
    }
    let path = root.join(id);
    if !path.is_dir() {
        return Err(AppError::msg("No se encontró esa carpeta"));
    }
    std::fs::remove_dir_all(&path)?;
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> AppResult<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let name = rel.to_string_lossy();
        if name.contains("__MACOSX") {
            continue;
        }
        let out_path = dest.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

fn find_dir_with(root: &Path, filename: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.join(filename).is_file() {
            return Some(dir);
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name == "__MACOSX" {
                    continue;
                }
                stack.push(p);
            }
        }
    }
    None
}

fn unique_dir(parent: &Path, name: &str) -> PathBuf {
    let mut dest = parent.join(name);
    let mut n = 2;
    while dest.exists() {
        dest = parent.join(format!("{name}_{n}"));
        n += 1;
    }
    dest
}

fn sanitize_name(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_control() || c == '/' || c == '\\' || c == ':' { '_' } else { c })
        .collect();
    s.trim().trim_matches('.').to_string()
}

fn import_world_from_dir(src: &Path, fallback: &str) -> AppResult<String> {
    let worlds = mojang()?.join("minecraftWorlds");
    std::fs::create_dir_all(&worlds)?;
    let mut name = read_trimmed(&src.join("levelname.txt"))
        .or_else(|| {
            src.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .filter(|s| !s.is_empty() && !s.starts_with('.'))
        })
        .unwrap_or_else(|| fallback.to_string());
    name = sanitize_name(&name);
    if name.is_empty() {
        name = fallback.to_string();
    }
    let dest = unique_dir(&worlds, &name);
    copy_dir(src, &dest)?;
    Ok(dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or(name))
}

fn import_pack_from_dir(src: &Path, fallback: &str) -> AppResult<String> {
    let kind = pack_kind(src);
    let folder = if kind == "behavior" {
        "behavior_packs"
    } else {
        "resource_packs"
    };
    let root = mojang()?.join(folder);
    std::fs::create_dir_all(&root)?;
    let mut name = sanitize_name(&pack_name(src, fallback));
    if name.is_empty() {
        name = fallback.to_string();
    }
    let dest = unique_dir(&root, &name);
    copy_dir(src, &dest)?;
    Ok(dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or(name))
}

fn pack_kind(dir: &Path) -> &'static str {
    let Ok(raw) = std::fs::read_to_string(dir.join("manifest.json")) else {
        return "resource";
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return "resource";
    };
    if let Some(modules) = v.get("modules").and_then(|m| m.as_array()) {
        for m in modules {
            let t = m.get("type").and_then(|x| x.as_str()).unwrap_or("");
            if t.eq_ignore_ascii_case("data") || t.eq_ignore_ascii_case("script") {
                return "behavior";
            }
        }
    }
    "resource"
}

fn copy_dir(src: &Path, dest: &Path) -> AppResult<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let to = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

/// Importa `.mcworld`, `.mcpack`, `.mcaddon` o zip genérico.
pub fn import_archive(archive: &Path) -> AppResult<String> {
    if !archive.is_file() {
        return Err(AppError::msg("No se encontró el archivo a importar"));
    }
    let fallback = archive
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "import".into());
    let ext = archive
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let tmp_parent = mojang()?.join(".pg-import-tmp");
    let tmp = unique_dir(&tmp_parent, "extract");
    std::fs::create_dir_all(&tmp)?;
    if let Err(e) = extract_zip(archive, &tmp) {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(e);
    }

    let result = (|| -> AppResult<String> {
        if ext == "mcworld" || find_dir_with(&tmp, "levelname.txt").is_some() || find_dir_with(&tmp, "level.dat").is_some() {
            let src = find_dir_with(&tmp, "levelname.txt")
                .or_else(|| find_dir_with(&tmp, "level.dat"))
                .unwrap_or_else(|| tmp.clone());
            return import_world_from_dir(&src, &fallback);
        }
        if ext == "mcaddon" {
            let mut n = 0;
            let mut last = String::new();
            let mut stack = vec![tmp.clone()];
            while let Some(dir) = stack.pop() {
                if dir.join("manifest.json").is_file() {
                    last = import_pack_from_dir(&dir, &fallback)?;
                    n += 1;
                    continue;
                }
                let Ok(entries) = std::fs::read_dir(&dir) else { continue };
                for e in entries.flatten() {
                    if e.path().is_dir() {
                        stack.push(e.path());
                    }
                }
            }
            if n == 0 {
                return Err(AppError::msg("El .mcaddon no trae packs con manifest.json"));
            }
            return Ok(last);
        }
        let src = find_dir_with(&tmp, "manifest.json").unwrap_or_else(|| tmp.clone());
        if src.join("manifest.json").is_file() {
            import_pack_from_dir(&src, &fallback)
        } else {
            Err(AppError::msg(
                "El archivo no parece un mundo (.mcworld) ni un pack (.mcpack).",
            ))
        }
    })();

    let _ = std::fs::remove_dir_all(&tmp);
    result
}
