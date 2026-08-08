//! Secret y dirección Playit **compartidos** por todos los servers del launcher.
//!
//! Un claim de la cuenta free de playit.gg alcanza para varios mundos:
//! copiamos el mismo `agent-secret` (plugin) / `playit-agent.toml` (desktop)
//! a cada carpeta y reutilizamos la IP `*.tun.ply.gg`.
//! Solo **un** servidor MC debe estar en ejecución a la vez con ese túnel.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::paths;
use crate::error::AppResult;

const SHARED_DIR: &str = "playit";
const SHARED_META: &str = "shared.json";
const SHARED_AGENT_TOML: &str = "playit-agent.toml";
const MIN_SECRET_LEN: usize = 32;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedPlayit {
    /// Secret del plugin `agent-secret` (plugins/playit-gg/config.yml).
    #[serde(default)]
    pub agent_secret: Option<String>,
    /// Última IP pública capturada (p. ej. `host.tun.ply.gg`).
    #[serde(default)]
    pub address: Option<String>,
    /// Dirección del túnel Bedrock (host o host:puerto público Playit).
    #[serde(default)]
    pub bedrock_address: Option<String>,
    /// El usuario ya hizo claim al menos una vez.
    #[serde(default)]
    pub claimed: bool,
}

fn shared_dir() -> PathBuf {
    let d = paths::data_dir().join(SHARED_DIR);
    let _ = fs::create_dir_all(&d);
    d
}

fn shared_meta_path() -> PathBuf {
    shared_dir().join(SHARED_META)
}

pub fn shared_agent_toml_path() -> PathBuf {
    shared_dir().join(SHARED_AGENT_TOML)
}

pub fn load() -> SharedPlayit {
    crate::config::read_json(&shared_meta_path()).unwrap_or_default()
}

fn save(meta: &SharedPlayit) -> AppResult<()> {
    crate::config::write_json_atomic(&shared_meta_path(), meta)
}

pub fn has_agent_secret() -> bool {
    load()
        .agent_secret
        .as_ref()
        .is_some_and(|s| s.trim().len() >= MIN_SECRET_LEN)
}

pub fn shared_address() -> Option<String> {
    load()
        .address
        .filter(|s| !s.trim().is_empty())
}

/// Guarda secret + opcionalmente IP en el store global.
pub fn set_agent_secret(secret: &str, address: Option<&str>) -> AppResult<()> {
    let secret = secret.trim();
    if secret.len() < MIN_SECRET_LEN {
        return Ok(());
    }
    let mut m = load();
    m.agent_secret = Some(secret.to_string());
    m.claimed = true;
    if let Some(a) = address.filter(|s| !s.is_empty()) {
        m.address = Some(a.to_string());
    }
    save(&m)?;
    // Mantener el TOML del daemon sincronizado (evita IPC mode).
    let body = format!(
        "# Generado por Paraguacraft Launcher — secret compartido\nsecret_key = \"{secret}\"\n"
    );
    let _ = fs::write(shared_agent_toml_path(), body);
    Ok(())
}

pub fn set_address(address: &str) -> AppResult<()> {
    let address = address.trim();
    if address.is_empty() {
        return Ok(());
    }
    let mut m = load();
    m.address = Some(address.to_string());
    save(&m)
}

pub fn set_bedrock_address(address: &str) -> AppResult<()> {
    let address = address.trim();
    if address.is_empty() {
        return Ok(());
    }
    let mut m = load();
    m.bedrock_address = Some(address.to_string());
    save(&m)
}

pub fn shared_bedrock_address() -> Option<String> {
    load()
        .bedrock_address
        .filter(|s| !s.trim().is_empty())
}

pub fn mark_claimed() -> AppResult<()> {
    let mut m = load();
    m.claimed = true;
    save(&m)
}

pub fn wipe_shared() -> AppResult<()> {
    let _ = fs::remove_file(shared_meta_path());
    let _ = fs::remove_file(shared_agent_toml_path());
    Ok(())
}

/// Extrae `agent-secret` de un YAML suelto (`config.yml` del plugin).
pub fn parse_agent_secret_yaml(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        let Some(rest) = t
            .strip_prefix("agent-secret:")
            .or_else(|| t.strip_prefix("agent_secret:"))
        else {
            continue;
        };
        let mut v = rest.trim().to_string();
        if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
            v = v[1..v.len() - 1].to_string();
        }
        v = v.trim().to_string();
        if v.len() >= MIN_SECRET_LEN {
            return Some(v);
        }
    }
    None
}

fn plugin_config_paths(server_dir: &Path) -> Vec<PathBuf> {
    [
        "plugins/playit-gg/config.yml",
        "plugins/playit-gg/config.yaml",
        "plugins/Playit/config.yml",
        "plugins/playit/config.yml",
    ]
    .iter()
    .map(|p| server_dir.join(p))
    .collect()
}

fn plugin_config_dir(server_dir: &Path) -> PathBuf {
    server_dir.join("plugins").join("playit-gg")
}

/// Lee secret del plugin en la carpeta del server (si existe).
pub fn read_plugin_secret_from_server(server_dir: &Path) -> Option<String> {
    for p in plugin_config_paths(server_dir) {
        if let Ok(c) = fs::read_to_string(&p) {
            if let Some(s) = parse_agent_secret_yaml(&c) {
                return Some(s);
            }
        }
    }
    None
}

/// Escribe el secret compartido en `plugins/playit-gg/config.yml` del server.
/// Si el archivo no existe, crea uno mínimo (el plugin lo completa al arrancar).
pub fn write_plugin_secret_to_server(server_dir: &Path, secret: &str) -> AppResult<()> {
    let secret = secret.trim();
    if secret.len() < MIN_SECRET_LEN {
        return Ok(());
    }
    let dir = plugin_config_dir(server_dir);
    fs::create_dir_all(&dir)?;
    let path = dir.join("config.yml");
    if path.is_file() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        if let Some(existing) = parse_agent_secret_yaml(&content) {
            if existing == secret {
                return Ok(());
            }
        }
        // Reemplaza o inserta agent-secret
        let mut out = String::new();
        let mut replaced = false;
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with("agent-secret:") || t.starts_with("agent_secret:") {
                out.push_str(&format!("agent-secret: \"{secret}\"\n"));
                replaced = true;
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        if !replaced {
            out.push_str(&format!("agent-secret: \"{secret}\"\n"));
        }
        fs::write(path, out)?;
    } else {
        fs::write(
            path,
            format!(
                "# Generado por Paraguacraft Launcher — secret compartido entre servers\nagent-secret: \"{secret}\"\nmc-timeout-sec: 30\n"
            ),
        )?;
    }
    Ok(())
}

/// Copia el toml de agente desktop compartido ↔ carpeta del server.
/// Si el TOML local no tiene `secret_key`, lo reescribe desde el store.
pub fn sync_agent_toml_to_server(server_dir: &Path) -> bool {
    // Preferimos regenerar desde el secret del store: un .toml vacío/viejo
    // mete a playitd en "Waiting for frontend secret provisioning" (IPC).
    if let Some(ref s) = load().agent_secret {
        if s.trim().len() >= MIN_SECRET_LEN {
            return write_agent_toml(server_dir, s).is_ok();
        }
    }
    let shared = shared_agent_toml_path();
    if !shared.is_file() {
        return false;
    }
    if let Ok(c) = fs::read_to_string(&shared) {
        if parse_agent_toml_secret(&c).is_none() {
            return false;
        }
    }
    let dest = server_dir.join("playit-agent.toml");
    fs::copy(&shared, &dest).is_ok()
}

/// Extrae `secret_key` de un playit-agent.toml (formato daemon 1.x).
pub fn parse_agent_toml_secret(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        let Some(rest) = t
            .strip_prefix("secret_key")
            .or_else(|| t.strip_prefix("secret-key"))
            .or_else(|| t.strip_prefix("SECRET_KEY"))
        else {
            continue;
        };
        let rest = rest.trim().trim_start_matches('=').trim();
        let mut v = rest.to_string();
        if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
            v = v[1..v.len() - 1].to_string();
        }
        v = v.trim().to_string();
        if v.len() >= MIN_SECRET_LEN {
            return Some(v);
        }
    }
    None
}

/// Escribe el secret del agente desktop (evita modo IPC sin secret).
pub fn write_agent_toml(server_dir: &Path, secret: &str) -> AppResult<()> {
    let secret = secret.trim();
    if secret.len() < MIN_SECRET_LEN {
        return Ok(());
    }
    let body = format!(
        "# Generado por Paraguacraft Launcher — NO editar a mano salvo reseteo\nsecret_key = \"{secret}\"\n"
    );
    fs::write(server_dir.join("playit-agent.toml"), &body)?;
    let _ = fs::write(shared_agent_toml_path(), &body);
    Ok(())
}

/// Secret efectivo para el daemon: store → toml del server → plugin config.
pub fn resolve_secret_for_server(server_dir: &Path) -> Option<String> {
    if let Some(s) = load()
        .agent_secret
        .filter(|s| s.trim().len() >= MIN_SECRET_LEN)
    {
        return Some(s.trim().to_string());
    }
    if let Ok(c) = fs::read_to_string(server_dir.join("playit-agent.toml")) {
        if let Some(s) = parse_agent_toml_secret(&c) {
            let _ = set_agent_secret(&s, None);
            return Some(s);
        }
    }
    if let Ok(c) = fs::read_to_string(shared_agent_toml_path()) {
        if let Some(s) = parse_agent_toml_secret(&c) {
            let _ = set_agent_secret(&s, None);
            return Some(s);
        }
    }
    if let Some(s) = read_plugin_secret_from_server(server_dir) {
        let _ = set_agent_secret(&s, None);
        return Some(s);
    }
    None
}

pub fn harvest_agent_toml_from_server(server_dir: &Path) -> bool {
    let src = server_dir.join("playit-agent.toml");
    if !src.is_file() {
        return false;
    }
    let Ok(c) = fs::read_to_string(&src) else {
        return false;
    };
    let Some(secret) = parse_agent_toml_secret(&c) else {
        return false;
    };
    let _ = set_agent_secret(&secret, None);
    fs::copy(&src, shared_agent_toml_path()).is_ok()
}

/// Cosecha secret del plugin de este server y lo guarda en el store compartido.
pub fn harvest_from_server(server_dir: &Path) -> Option<String> {
    let secret = read_plugin_secret_from_server(server_dir)?;
    let _ = set_agent_secret(&secret, None);
    let _ = write_agent_toml(server_dir, &secret);
    let _ = harvest_agent_toml_from_server(server_dir);
    Some(secret)
}

/// Aplica el secret compartido a la carpeta del server (plugin + agent.toml).
/// Devuelve `true` si escribió un secret usable.
pub fn apply_to_server(server_dir: &Path) -> AppResult<bool> {
    let secret = resolve_secret_for_server(server_dir);
    let Some(secret) = secret else {
        let _ = sync_agent_toml_to_server(server_dir);
        return Ok(false);
    };
    write_plugin_secret_to_server(server_dir, &secret)?;
    write_agent_toml(server_dir, &secret)?;
    Ok(true)
}
