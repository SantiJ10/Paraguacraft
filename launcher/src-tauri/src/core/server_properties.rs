//! Lectura/escritura atómica de `server.properties`.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};

static PROPS_LOCK: Mutex<()> = Mutex::new(());

pub fn read(dir: &Path) -> AppResult<HashMap<String, String>> {
    let path = dir.join("server.properties");
    if !path.is_file() {
        return Err(AppError::msg(
            "server.properties no existe. Iniciá el servidor al menos una vez o usá «Preparar servidor».",
        ));
    }
    let content = fs::read_to_string(&path)?;
    let mut props = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            props.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(props)
}

pub fn write(dir: &Path, updates: &HashMap<String, String>) -> AppResult<()> {
    for (k, v) in updates {
        if k.contains('=') || k.contains('\n') || k.contains('\r') {
            return Err(AppError::msg(format!("Clave inválida: {k}")));
        }
        if v.contains('\n') || v.contains('\r') {
            return Err(AppError::msg(format!("Valor inválido en {k}")));
        }
    }

    let path = dir.join("server.properties");
    if !path.is_file() {
        return Err(AppError::msg("server.properties no existe."));
    }

    let _guard = PROPS_LOCK.lock().unwrap();
    let lines = fs::read_to_string(&path)?;
    let mut updated = std::collections::HashSet::new();
    let mut new_lines = Vec::new();

    for line in lines.lines() {
        let stripped = line.trim();
        if stripped.starts_with('#') || !stripped.contains('=') {
            new_lines.push(format!("{line}\n"));
            continue;
        }
        let k = stripped.split('=').next().unwrap_or("").trim();
        if let Some(v) = updates.get(k) {
            new_lines.push(format!("{k}={v}\n"));
            updated.insert(k.to_string());
        } else {
            new_lines.push(format!("{line}\n"));
        }
    }
    for (k, v) in updates {
        if !updated.contains(k) {
            new_lines.push(format!("{k}={v}\n"));
        }
    }

    let tmp = dir.join(format!(".server.properties.{}.tmp", uuid::Uuid::new_v4()));
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(new_lines.join("").as_bytes())?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Claves editables en la UI (subset útil).
pub const EDITABLE_KEYS: &[&str] = &[
    "motd",
    "max-players",
    "difficulty",
    "gamemode",
    "pvp",
    "enable-command-block",
    "spawn-protection",
    "white-list",
    "server-port",
    "level-name",
    "online-mode",
    "view-distance",
    "simulation-distance",
];
