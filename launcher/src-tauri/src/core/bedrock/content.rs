//! Mundos y packs de `%LOCALAPPDATA%\Packages\...\LocalState\games\com.mojang`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

use super::{all_com_mojang_dirs, com_mojang_dir};

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

fn mojang_roots() -> Vec<PathBuf> {
    let dirs = all_com_mojang_dirs();
    if !dirs.is_empty() {
        return dirs;
    }
    com_mojang_dir().into_iter().collect()
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
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for root in mojang_roots() {
        let worlds = root.join("minecraftWorlds");
        let Ok(entries) = std::fs::read_dir(&worlds) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let folder = entry.file_name().to_string_lossy().to_string();
            if folder.starts_with('.') {
                continue;
            }
            if !path.join("level.dat").is_file() && !path.join("levelname.txt").is_file() {
                continue;
            }
            let key = path.to_string_lossy().to_string();
            if !seen.insert(key) {
                continue;
            }
            let name = read_trimmed(&path.join("levelname.txt")).unwrap_or_else(|| folder.clone());
            out.push(BedrockWorld {
                id: folder,
                name,
                path: path.to_string_lossy().to_string(),
                icon_path: folder_icon(&path),
            });
        }
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
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for root in mojang_roots() {
        let dir = root.join(folder);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
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
            let key = path.to_string_lossy().to_string();
            if !seen.insert(key) {
                continue;
            }
            let name = pack_name(&path, &id);
            out.push(BedrockPack {
                id,
                name,
                kind: kind.to_string(),
                path: path.to_string_lossy().to_string(),
                icon_path: folder_icon(&path),
            });
        }
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
    remove_named_dir("minecraftWorlds", id)
}

pub fn delete_pack(kind: &str, id: &str) -> AppResult<()> {
    let folder = match kind {
        "behavior" => "behavior_packs",
        _ => "resource_packs",
    };
    remove_named_dir(folder, id)
}

/// Borra en todas las raíces `com.mojang`, no solo la primera: listamos desde
/// todas, así que borrar de una sola dejaba entradas imposibles de eliminar.
fn remove_named_dir(folder: &str, id: &str) -> AppResult<()> {
    if id.is_empty() || id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err(AppError::msg("Identificador inválido"));
    }
    let mut removed = false;
    for root in mojang_roots() {
        let path = root.join(folder).join(id);
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
            removed = true;
        }
    }
    if !removed {
        return Err(AppError::msg("No se encontró esa carpeta"));
    }
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

struct PackOutcome {
    name: String,
    kind: &'static str,
    activated: bool,
    downgraded: bool,
}

fn import_pack_from_dir(
    src: &Path,
    fallback: &str,
    engine: Option<[u32; 3]>,
) -> AppResult<PackOutcome> {
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

    let downgraded = engine.map(|e| relax_min_engine_version(&dest, e)).unwrap_or(false);
    let activated = kind == "resource" && activate_resource_pack(&dest);

    Ok(PackOutcome {
        name: dest
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or(name),
        kind,
        activated,
        downgraded,
    })
}

/// Packs que cuelgan directo de `root` (o `root` mismo si ya es un pack).
///
/// No sirve recorrer el árbol entero: un resource pack puede traer `subpacks/`
/// con sus propios `manifest.json` y contarían como packs sueltos.
fn top_level_packs(root: &Path) -> Vec<PathBuf> {
    if root.join("manifest.json").is_file() {
        return vec![root.to_path_buf()];
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("manifest.json").is_file())
        .collect();
    out.sort();
    if out.is_empty() {
        // `.mcaddon` suele envolver todo en una carpeta intermedia.
        if let Some(only) = single_subdir(root) {
            return top_level_packs(&only);
        }
    }
    out
}

fn single_subdir(root: &Path) -> Option<PathBuf> {
    let mut dirs = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir());
    let first = dirs.next()?;
    dirs.next().is_none().then_some(first)
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

/// Resultado de importar un `.mcpack` / `.mcaddon` / `.mcworld`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockImport {
    /// Carpeta final; la última si el `.mcaddon` traía varios packs.
    pub name: String,
    /// `world` | `resource` | `behavior`
    pub kind: String,
    /// Cuántos packs entraron (un `.mcaddon` puede traer varios).
    pub packs: usize,
    /// Quedó activo en `global_resource_packs.json` sin tocar el juego.
    pub activated: bool,
    /// Se bajó `min_engine_version` para que cargue en la Bedrock activa.
    pub downgraded: bool,
}

/// Versión de Bedrock o de un pack como triple comparable.
/// Acepta `[1,16,0]`, `"1.16.0"` y también `"1.21.30.3"` (sobra el build).
fn version_triple(value: &serde_json::Value) -> Option<[u32; 3]> {
    if let Some(arr) = value.as_array() {
        let mut out = [0u32; 3];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = arr.get(i).and_then(|x| x.as_u64()).unwrap_or(0) as u32;
        }
        return Some(out);
    }
    value.as_str().and_then(engine_triple)
}

fn engine_triple(version: &str) -> Option<[u32; 3]> {
    let mut out = [0u32; 3];
    let mut seen = 0;
    for (i, part) in version.trim().split('.').take(3).enumerate() {
        out[i] = part.trim().parse().ok()?;
        seen += 1;
    }
    (seen > 0).then_some(out)
}

/// Versión de Bedrock actualmente registrada, si la hay.
fn active_engine() -> Option<[u32; 3]> {
    super::status().active_version.as_deref().and_then(engine_triple)
}

/// Bedrock ignora un pack cuyo `min_engine_version` supera al juego. Muchos
/// packs declaran un mínimo más alto del que de verdad necesitan, así que al
/// instalar sobre una versión vieja lo bajamos en vez de dejarlo invisible.
fn relax_min_engine_version(dir: &Path, engine: [u32; 3]) -> bool {
    let path = dir.join("manifest.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(mut manifest) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    let Some(declared) = manifest
        .pointer("/header/min_engine_version")
        .and_then(version_triple)
    else {
        return false;
    };
    if declared <= engine {
        return false;
    }
    let Some(header) = manifest.get_mut("header").and_then(|h| h.as_object_mut()) else {
        return false;
    };
    header.insert(
        "min_engine_version".into(),
        serde_json::json!([engine[0], engine[1], engine[2]]),
    );
    serde_json::to_string_pretty(&manifest)
        .ok()
        .and_then(|json| std::fs::write(&path, json).ok())
        .is_some()
}

/// Activa un resource pack en `global_resource_packs.json`, que es lo que
/// escribe el juego al moverlo a «Activos». Los behavior packs no tienen
/// equivalente global: se activan por mundo desde el propio Minecraft.
fn activate_resource_pack(dir: &Path) -> bool {
    let path = dir.join("manifest.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    let Some(uuid) = manifest
        .pointer("/header/uuid")
        .and_then(|u| u.as_str())
        .filter(|s| !s.is_empty())
    else {
        return false;
    };
    let version = manifest
        .pointer("/header/version")
        .and_then(version_triple)
        .unwrap_or([1, 0, 0]);

    let mut done = false;
    for root in mojang_roots() {
        let mcpe = root.join("minecraftpe");
        if std::fs::create_dir_all(&mcpe).is_err() {
            continue;
        }
        let list_path = mcpe.join("global_resource_packs.json");
        let mut list: Vec<serde_json::Value> = std::fs::read_to_string(&list_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        if list
            .iter()
            .any(|p| p.get("pack_id").and_then(|v| v.as_str()) == Some(uuid))
        {
            done = true;
            continue;
        }
        list.insert(0, serde_json::json!({ "pack_id": uuid, "version": version }));
        if let Ok(json) = serde_json::to_string_pretty(&list) {
            if std::fs::write(&list_path, json).is_ok() {
                done = true;
            }
        }
    }
    done
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
///
/// Los packs quedan activados y con `min_engine_version` ajustado a la Bedrock
/// registrada, para que también carguen en versiones viejas.
pub fn import_archive(archive: &Path) -> AppResult<BedrockImport> {
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

    let result = (|| -> AppResult<BedrockImport> {
        let world_src = find_dir_with(&tmp, "levelname.txt").or_else(|| find_dir_with(&tmp, "level.dat"));
        if ext == "mcworld" || world_src.is_some() {
            let src = world_src.unwrap_or_else(|| tmp.clone());
            let name = import_world_from_dir(&src, &fallback)?;
            return Ok(BedrockImport {
                name,
                kind: "world".into(),
                packs: 0,
                activated: false,
                downgraded: false,
            });
        }

        let packs = top_level_packs(&tmp);
        if packs.is_empty() {
            return Err(AppError::msg(
                "El archivo no parece un mundo (.mcworld) ni un pack (.mcpack).",
            ));
        }

        let engine = active_engine();
        let mut last: Option<PackOutcome> = None;
        let mut activated = false;
        let mut downgraded = false;
        for dir in &packs {
            let outcome = import_pack_from_dir(dir, &fallback, engine)?;
            activated |= outcome.activated;
            downgraded |= outcome.downgraded;
            last = Some(outcome);
        }
        let last = last.expect("packs no está vacío");
        Ok(BedrockImport {
            name: last.name,
            kind: last.kind.into(),
            packs: packs.len(),
            activated,
            downgraded,
        })
    })();

    let _ = std::fs::remove_dir_all(&tmp);
    result
}

/// Extensiones que Bedrock entiende; todas son ZIP por dentro.
const PACK_EXTS: [&str; 4] = ["mcpack", "mcaddon", "mcworld", "zip"];

/// Un pack de 512 MB ya es absurdo; corta descargas equivocadas.
const MAX_PACK_BYTES: usize = 512 * 1024 * 1024;

fn pack_ext_from_url(url: &str) -> &'static str {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let name = path.rsplit('/').next().unwrap_or("").to_ascii_lowercase();
    PACK_EXTS
        .into_iter()
        .find(|ext| name.ends_with(&format!(".{ext}")))
        // Sin pista en la URL: el contenido decide (mundo vs pack vs addon).
        .unwrap_or("zip")
}

/// Descarga un `.mcpack` / `.mcaddon` / `.mcworld` y lo importa.
pub async fn install_from_url(client: &reqwest::Client, url: &str) -> AppResult<BedrockImport> {
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(AppError::msg("La URL tiene que empezar con http:// o https://"));
    }
    let ext = pack_ext_from_url(url);
    let bytes = crate::core::net::fetch_bytes(client, url).await?;
    if bytes.is_empty() {
        return Err(AppError::msg("La descarga vino vacía"));
    }
    if bytes.len() > MAX_PACK_BYTES {
        return Err(AppError::msg(
            "El archivo pesa más de 512 MB; no parece un pack de Bedrock.",
        ));
    }

    tokio::task::spawn_blocking(move || {
        let tmp_dir = crate::core::paths::data_dir().join("bedrock-downloads");
        std::fs::create_dir_all(&tmp_dir)?;
        let tmp = tmp_dir.join(format!("pack-{}.{ext}", std::process::id()));
        std::fs::write(&tmp, &bytes)?;
        let result = import_archive(&tmp);
        let _ = std::fs::remove_file(&tmp);
        result
    })
    .await
    .unwrap_or_else(|e| Err(AppError::msg(format!("Importación abortada: {e}"))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pg-bedrock-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_manifest(dir: &Path, json: serde_json::Value) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("manifest.json"), json.to_string()).unwrap();
    }

    #[test]
    fn engine_triple_accepts_build_suffix() {
        assert_eq!(engine_triple("1.16.201"), Some([1, 16, 201]));
        // Las versiones de Bedrock traen un cuarto número de build.
        assert_eq!(engine_triple("1.21.30.3"), Some([1, 21, 30]));
        assert_eq!(engine_triple(""), None);
        assert_eq!(engine_triple("no-es-version"), None);
    }

    #[test]
    fn version_triple_reads_array_and_string() {
        assert_eq!(
            version_triple(&serde_json::json!([1, 16, 0])),
            Some([1, 16, 0])
        );
        assert_eq!(version_triple(&serde_json::json!("2.3.4")), Some([2, 3, 4]));
        assert_eq!(version_triple(&serde_json::json!(7)), None);
    }

    #[test]
    fn pack_ext_survives_query_strings() {
        assert_eq!(pack_ext_from_url("https://x.com/a/cool.mcpack"), "mcpack");
        assert_eq!(pack_ext_from_url("https://x.com/a/b.mcaddon?dl=1"), "mcaddon");
        assert_eq!(pack_ext_from_url("https://x.com/w.MCWORLD#frag"), "mcworld");
        // Sin extensión reconocible decide el contenido, no la URL.
        assert_eq!(pack_ext_from_url("https://x.com/download/9281"), "zip");
    }

    #[test]
    fn min_engine_version_drops_to_the_installed_game() {
        let dir = tmp_dir("relax");
        write_manifest(
            &dir,
            serde_json::json!({
                "format_version": 2,
                "header": { "name": "P", "uuid": "u", "min_engine_version": [1, 21, 0] }
            }),
        );

        assert!(relax_min_engine_version(&dir, [1, 16, 201]));
        let raw = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            version_triple(v.pointer("/header/min_engine_version").unwrap()),
            Some([1, 16, 201])
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn min_engine_version_left_alone_when_already_compatible() {
        let dir = tmp_dir("keep");
        write_manifest(
            &dir,
            serde_json::json!({
                "header": { "uuid": "u", "min_engine_version": [1, 12, 0] }
            }),
        );
        assert!(!relax_min_engine_version(&dir, [1, 21, 30]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn manifest_without_min_engine_version_is_untouched() {
        let dir = tmp_dir("nomin");
        write_manifest(&dir, serde_json::json!({ "header": { "uuid": "u" } }));
        assert!(!relax_min_engine_version(&dir, [1, 16, 0]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn subpacks_do_not_count_as_separate_packs() {
        let dir = tmp_dir("subpacks");
        write_manifest(&dir, serde_json::json!({ "header": { "uuid": "raiz" } }));
        write_manifest(
            &dir.join("subpacks").join("low"),
            serde_json::json!({ "header": { "uuid": "low" } }),
        );

        let found = top_level_packs(&dir);
        assert_eq!(found, vec![dir.clone()], "el pack raíz gana sobre subpacks");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn addon_with_two_packs_is_detected() {
        let dir = tmp_dir("addon");
        write_manifest(&dir.join("RP"), serde_json::json!({ "header": { "uuid": "rp" } }));
        write_manifest(&dir.join("BP"), serde_json::json!({ "header": { "uuid": "bp" } }));

        assert_eq!(top_level_packs(&dir).len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn addon_wrapped_in_one_folder_is_unwrapped() {
        let dir = tmp_dir("wrapped");
        let inner = dir.join("MiAddon");
        write_manifest(&inner.join("RP"), serde_json::json!({ "header": { "uuid": "rp" } }));
        write_manifest(&inner.join("BP"), serde_json::json!({ "header": { "uuid": "bp" } }));

        assert_eq!(top_level_packs(&dir).len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn behavior_pack_detected_by_module_type() {
        let dir = tmp_dir("kind");
        write_manifest(
            &dir,
            serde_json::json!({
                "header": { "uuid": "u" },
                "modules": [{ "type": "data", "uuid": "m" }]
            }),
        );
        assert_eq!(pack_kind(&dir), "behavior");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
