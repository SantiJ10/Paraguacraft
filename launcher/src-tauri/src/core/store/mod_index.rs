//! Índice persistente por archivo instalado (estilo Packwiz).
//!
//! Vive en `<carpeta>/.index/<stem>.json` (p.ej. `mods/.index/fabric-api.json`).
//! Permite actualizar y resolver deps sin re-hashear todos los jars (Regla 3).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModIndexEntry {
    pub provider: String,
    pub project_id: String,
    #[serde(default)]
    pub file_id: String,
    #[serde(default)]
    pub sha1: Option<String>,
    pub filename: String,
    #[serde(default)]
    pub mc: Option<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    /// Project ids REQUIRED declarados al instalar (para pre-launch).
    #[serde(default)]
    pub dependencies: Vec<String>,
}

fn index_dir(content_dir: &Path) -> PathBuf {
    content_dir.join(".index")
}

fn stem_of(filename: &str) -> String {
    let mut s = filename.to_string();
    for suf in [".jar.disabled", ".jar", ".zip", ".disabled"] {
        if let Some(stripped) = s.strip_suffix(suf) {
            s = stripped.to_string();
            break;
        }
    }
    let s = s.trim();
    if s.is_empty() {
        "file".into()
    } else {
        s.replace(['/', '\\', ':'], "_")
    }
}

fn entry_path(content_dir: &Path, filename: &str) -> PathBuf {
    index_dir(content_dir).join(format!("{}.json", stem_of(filename)))
}

pub fn write_entry(content_dir: &Path, entry: &ModIndexEntry) -> AppResult<()> {
    let dir = index_dir(content_dir);
    fs::create_dir_all(&dir)?;
    let path = entry_path(content_dir, &entry.filename);
    let json = serde_json::to_string_pretty(entry)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn record(
    content_dir: &Path,
    provider: &str,
    project_id: &str,
    file_id: &str,
    filename: &str,
    sha1: Option<String>,
    mc: Option<&str>,
    loaders: &[String],
    dependencies: &[String],
) -> AppResult<()> {
    write_entry(
        content_dir,
        &ModIndexEntry {
            provider: provider.to_string(),
            project_id: project_id.to_string(),
            file_id: file_id.to_string(),
            sha1,
            filename: filename.to_string(),
            mc: mc.map(String::from),
            loaders: loaders.to_vec(),
            dependencies: dependencies.to_vec(),
        },
    )
}

pub fn remove_for_filename(content_dir: &Path, filename: &str) {
    let _ = fs::remove_file(entry_path(content_dir, filename));
}

pub fn read_all(content_dir: &Path) -> Vec<ModIndexEntry> {
    let dir = index_dir(content_dir);
    let Ok(rd) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&p) {
            if let Ok(entry) = serde_json::from_str::<ModIndexEntry>(&raw) {
                out.push(entry);
            }
        }
    }
    out
}

pub fn project_installed(content_dir: &Path, project_id: &str) -> bool {
    let id = project_id.to_lowercase();
    read_all(content_dir)
        .iter()
        .any(|e| e.project_id.to_lowercase() == id)
}

/// Quita entradas cuyo archivo ya no está (ni `.disabled`).
pub fn prune_missing(content_dir: &Path) {
    for entry in read_all(content_dir) {
        let p = content_dir.join(&entry.filename);
        let disabled = PathBuf::from(format!("{}.disabled", p.display()));
        let alt = content_dir.join(format!("{}.disabled", entry.filename));
        if !p.is_file() && !disabled.is_file() && !alt.is_file() {
            remove_for_filename(content_dir, &entry.filename);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn roundtrip_index_entry() {
        let dir = env::temp_dir().join(format!("pg-idx-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        record(
            &dir,
            "modrinth",
            "P7dR8mSH",
            "abc",
            "fabric-api-1.0.jar",
            Some("deadbeef".into()),
            Some("1.21.11"),
            &["fabric".into()],
            &["sodium".into()],
        )
        .unwrap();
        let all = read_all(&dir);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].project_id, "P7dR8mSH");
        assert!(project_installed(&dir, "P7dR8mSH"));
        let _ = fs::remove_dir_all(&dir);
    }
}
