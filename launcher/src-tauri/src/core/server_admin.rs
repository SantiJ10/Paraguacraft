//! Whitelist, OPs y bans (JSON del servidor + comandos en vivo).

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, AppResult};

static JSON_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedEntry {
    pub name: String,
    #[serde(default)]
    pub uuid: String,
}

fn read_json(path: &Path) -> Vec<Value> {
    if !path.is_file() {
        return Vec::new();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_json_atomic(dir: &Path, filename: &str, data: &Value) -> AppResult<()> {
    let _guard = JSON_LOCK.lock().unwrap();
    let path = dir.join(filename);
    let tmp = dir.join(format!(".{filename}.{}.tmp", uuid::Uuid::new_v4()));
    {
        let mut f = fs::File::create(&tmp)?;
        let pretty = serde_json::to_string_pretty(data)?;
        f.write_all(pretty.as_bytes())?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn whitelist_names(dir: &Path) -> Vec<String> {
    read_json(&dir.join("whitelist.json"))
        .into_iter()
        .filter_map(|e| e.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect()
}

pub fn op_names(dir: &Path) -> Vec<String> {
    read_json(&dir.join("ops.json"))
        .into_iter()
        .filter_map(|e| e.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect()
}

pub fn ban_names(dir: &Path) -> Vec<String> {
    read_json(&dir.join("banned-players.json"))
        .into_iter()
        .filter_map(|e| e.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect()
}

fn push_entry(dir: &Path, filename: &str, name: &str) -> AppResult<()> {
    let path = dir.join(filename);
    let mut entries = read_json(&path);
    if entries
        .iter()
        .any(|e| e.get("name").and_then(|n| n.as_str()).unwrap_or("").eq_ignore_ascii_case(name))
    {
        return Ok(());
    }
    entries.push(serde_json::json!({
        "uuid": uuid::Uuid::new_v4().to_string(),
        "name": name,
    }));
    write_json_atomic(dir, filename, &Value::Array(entries))
}

fn remove_entry(dir: &Path, filename: &str, name: &str) -> AppResult<()> {
    let path = dir.join(filename);
    let entries: Vec<Value> = read_json(&path)
        .into_iter()
        .filter(|e| {
            !e.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .eq_ignore_ascii_case(name)
        })
        .collect();
    write_json_atomic(dir, filename, &Value::Array(entries))
}

pub fn whitelist_add_offline(dir: &Path, name: &str) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::msg("Nombre vacío"));
    }
    push_entry(dir, "whitelist.json", name)
}

pub fn whitelist_remove_offline(dir: &Path, name: &str) -> AppResult<()> {
    remove_entry(dir, "whitelist.json", name.trim())
}

pub fn op_add_offline(dir: &Path, name: &str) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::msg("Nombre vacío"));
    }
    let path = dir.join("ops.json");
    let mut entries = read_json(&path);
    if entries.iter().any(|e| {
        e.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .eq_ignore_ascii_case(name)
    }) {
        return Ok(());
    }
    entries.push(serde_json::json!({
        "uuid": uuid::Uuid::new_v4().to_string(),
        "name": name,
        "level": 4,
        "bypassesPlayerLimit": false
    }));
    write_json_atomic(dir, "ops.json", &Value::Array(entries))
}

pub fn op_remove_offline(dir: &Path, name: &str) -> AppResult<()> {
    remove_entry(dir, "ops.json", name.trim())
}

pub fn ban_add_offline(dir: &Path, name: &str) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::msg("Nombre vacío"));
    }
    let path = dir.join("banned-players.json");
    let mut entries = read_json(&path);
    if entries.iter().any(|e| {
        e.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .eq_ignore_ascii_case(name)
    }) {
        return Ok(());
    }
    entries.push(serde_json::json!({
        "uuid": uuid::Uuid::new_v4().to_string(),
        "name": name,
        "created": chrono_now(),
        "source": "Paraguacraft Launcher",
        "expires": "forever"
    }));
    write_json_atomic(dir, "banned-players.json", &Value::Array(entries))
}

pub fn ban_remove_offline(dir: &Path, name: &str) -> AppResult<()> {
    remove_entry(dir, "banned-players.json", name.trim())
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}
