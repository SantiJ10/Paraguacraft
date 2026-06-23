//! Historial de skins aplicadas (espejo de `paragua.py`).

use std::path::PathBuf;

use crate::core::paths;
use crate::error::AppResult;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinHistoryEntry {
    pub nombre: String,
    pub url: String,
    pub tipo: String,
}

fn historial_path() -> PathBuf {
    paths::skins_library_dir().join("historial.json")
}

pub fn list() -> Vec<SkinHistoryEntry> {
    let path = historial_path();
    if !path.is_file() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn push(nombre: &str, url: &str, tipo: &str) {
    let mut hist = list();
    hist.retain(|h| h.url != url);
    hist.insert(
        0,
        SkinHistoryEntry {
            nombre: nombre.to_string(),
            url: url.to_string(),
            tipo: tipo.to_string(),
        },
    );
    hist.truncate(50);
    let path = historial_path();
    if let Ok(json) = serde_json::to_string_pretty(&hist) {
        let _ = std::fs::write(path, json);
    }
}

pub fn clear() -> AppResult<()> {
    let path = historial_path();
    if path.is_file() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
