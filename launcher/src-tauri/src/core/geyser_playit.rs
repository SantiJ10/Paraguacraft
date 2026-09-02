//! Ajustes Geyser para túneles playit.gg (guía oficial GeyserMC).
//!
//! - `broadcast-port` = puerto público del túnel Bedrock
//! - `advanced.bedrock.use-haproxy-protocol` = true (con PROXY en el túnel playit)
//! - `auth-type: floodgate` si hay Floodgate en plugins/

use std::fs;
use std::path::Path;

use crate::error::AppResult;

/// Aplica los ajustes mínimos para que Bedrock entre por playit (host:puerto público).
pub fn apply_playit_settings(server_dir: &Path, bedrock_public_addr: &str) -> AppResult<Vec<String>> {
    let mut notes = Vec::new();
    let public_port = parse_public_port(bedrock_public_addr);
    let has_floodgate = floodgate_present(server_dir);

    for path in geyser_config_paths(server_dir) {
        if !path.is_file() {
            continue;
        }
        let original = fs::read_to_string(&path)?;
        let mut next = original.clone();

        if has_floodgate {
            next = next.replacen("auth-type: online", "auth-type: floodgate", 1);
            next = next.replacen("auth-type: offline", "auth-type: floodgate", 1);
        }

        if let Some(port) = public_port {
            next = set_yaml_key_line(&next, "broadcast-port", &port.to_string(), true);
        }

        // 1ª aparición = advanced.java (dejar false); 2ª = advanced.bedrock (true).
        next = set_nth_yaml_key(&next, "use-haproxy-protocol", "true", 2);

        if next != original {
            fs::write(&path, next)?;
            notes.push(format!(
                "Geyser: actualicé {} para playit (broadcast-port + HAProxy bedrock{}, reiniciá el server).",
                path.file_name().and_then(|s| s.to_str()).unwrap_or("config.yml"),
                if has_floodgate { ", auth floodgate" } else { "" }
            ));
        } else {
            notes.push("Geyser: config ya tenía ajustes playit.".into());
        }
    }

    if notes.is_empty() {
        notes.push("Geyser: no encontré config.yml (arrancá una vez el server para generarla).".into());
    }
    Ok(notes)
}

const SR_CONFIG_RELS: &[&str] = &[
    "plugins/SkinsRestorer/config.yml",
    "plugins/SkinsRestorer/config.yaml",
    "config/skinsrestorer/config.yml",
    "config/SkinsRestorer/config.yml",
];

/// SkinsRestorer: Ely.by para no-premium (vanilla y otros launchers) y sin
/// defaultSkins para no pisar la skin Bedrock que sube Floodgate.
pub fn apply_skinsrestorer_compat(server_dir: &Path) -> AppResult<Vec<String>> {
    let mut notes = Vec::new();
    let mut patched_sr = false;
    for rel in SR_CONFIG_RELS {
        let path = server_dir.join(rel);
        if !path.is_file() {
            continue;
        }
        let original = fs::read_to_string(&path)?;
        let mut next = disable_sr_default_skins(&original);
        next = enable_sr_elyby(&next);
        if next != original {
            fs::write(&path, next)?;
            patched_sr = true;
        }
    }
    if patched_sr {
        notes.push(
            "SkinsRestorer: Ely.by para cuentas no-premium y sin skins default (no pisa Floodgate). Reiniciá el server una vez.".into(),
        );
    }

    if floodgate_present(server_dir) {
        notes.push(
            "Skins Bedrock→Java: en el celu usá una skin clásica 64×64 (archivo PNG), no el creador de personajes. Las skins «persona» Java no las puede mostrar. Al entrar, esperá 5–10 s o reconectá en Java.".into(),
        );
    }
    Ok(notes)
}

/// `api.elyByEnabled` (SR 15.11+): busca skins en Ely.by y cae a Mojang.
/// `advanced.noConnections: false` para que esas consultas salgan.
fn enable_sr_elyby(content: &str) -> String {
    let mut next = if content.contains("elyByEnabled:") {
        set_yaml_key_line(content, "elyByEnabled", "true", true)
    } else {
        insert_under_yaml_key(content, "api", "elyByEnabled", "true")
    };
    if next.contains("noConnections:") {
        next = set_yaml_key_line(&next, "noConnections", "false", true);
    }
    next
}

/// Inserta `key: value` como hijo de `parent:` si no está.
fn insert_under_yaml_key(content: &str, parent: &str, key: &str, value: &str) -> String {
    if content.contains(&format!("{key}:")) {
        return content.to_string();
    }
    let mut out = String::with_capacity(content.len() + 32);
    let mut inserted = false;
    let parent_hdr = format!("{parent}:");
    for line in content.lines() {
        out.push_str(line);
        out.push('\n');
        if inserted {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with(&parent_hdr) {
            let indent = line.len() - trimmed.len();
            for _ in 0..(indent + 2) {
                out.push(' ');
            }
            out.push_str(key);
            out.push_str(": ");
            out.push_str(value);
            out.push('\n');
            inserted = true;
        }
    }
    if !inserted {
        out.push_str(parent);
        out.push_str(":\n  ");
        out.push_str(key);
        out.push_str(": ");
        out.push_str(value);
        out.push('\n');
    }
    if !content.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

/// `storage.defaultSkins.enabled: true` hace que SR aplique Steve/xknat a
/// jugadores Floodgate (`.nick`) y tape la skin Bedrock.
fn disable_sr_default_skins(content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 8);
    let mut in_default_skins = false;
    let mut parent_indent = 0usize;
    for line in content.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if trimmed.starts_with("defaultSkins:") {
            in_default_skins = true;
            parent_indent = indent;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_default_skins {
            if !trimmed.is_empty() && !trimmed.starts_with('#') && indent <= parent_indent {
                in_default_skins = false;
            } else if trimmed.starts_with("enabled:") {
                out.push_str(&line[..indent]);
                out.push_str("enabled: false\n");
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    if !content.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

fn floodgate_present(server_dir: &Path) -> bool {
    let roots = [server_dir.join("plugins"), server_dir.join("mods")];
    for root in roots {
        let Ok(rd) = fs::read_dir(root) else {
            continue;
        };
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_ascii_lowercase();
            if n.ends_with(".jar") && n.contains("floodgate") {
                return true;
            }
        }
    }
    false
}

fn geyser_config_paths(server_dir: &Path) -> Vec<std::path::PathBuf> {
    [
        "plugins/Geyser-Spigot/config.yml",
        "plugins/Geyser-Fabric/config.yml",
        "plugins/Geyser-Paper/config.yml",
        "config/Geyser-Fabric/config.yml",
    ]
    .iter()
    .map(|rel| server_dir.join(rel))
    .collect()
}

fn parse_public_port(addr: &str) -> Option<u16> {
    let addr = addr.trim();
    let port = addr.rsplit_once(':')?.1;
    port.parse().ok().filter(|p| *p > 0)
}

/// Reemplaza la 1ª (o todas con `all`) línea `key: value` (permite espacios).
fn set_yaml_key_line(content: &str, key: &str, value: &str, all: bool) -> String {
    let mut out = String::with_capacity(content.len() + 16);
    let mut replaced = 0u32;
    for line in content.lines() {
        let trimmed = line.trim_start();
        let is_key = trimmed.starts_with(&format!("{key}:"))
            || trimmed.starts_with(&format!("{key} "));
        if is_key && (all || replaced == 0) {
            let indent = &line[..line.len() - line.trim_start().len()];
            out.push_str(indent);
            out.push_str(key);
            out.push_str(": ");
            out.push_str(value);
            out.push('\n');
            replaced += 1;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if content.ends_with('\n') {
        // already newlines
    } else if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Cambia la n-ésima (1-based) aparición de `key:`.
fn set_nth_yaml_key(content: &str, key: &str, value: &str, nth: u32) -> String {
    let mut out = String::with_capacity(content.len() + 16);
    let mut seen = 0u32;
    for line in content.lines() {
        let trimmed = line.trim_start();
        let is_key = trimmed.starts_with(&format!("{key}:"));
        if is_key {
            seen += 1;
            if seen == nth {
                let indent = &line[..line.len() - line.trim_start().len()];
                out.push_str(indent);
                out.push_str(key);
                out.push_str(": ");
                out.push_str(value);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    if !content.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_broadcast_and_second_haproxy() {
        let sample = r#"java:
  auth-type: online
advanced:
  java:
    use-haproxy-protocol: false
  bedrock:
    broadcast-port: 0
    use-haproxy-protocol: false
"#;
        let mut s = sample.replacen("auth-type: online", "auth-type: floodgate", 1);
        s = set_yaml_key_line(&s, "broadcast-port", "15353", true);
        s = set_nth_yaml_key(&s, "use-haproxy-protocol", "true", 2);
        assert!(s.contains("auth-type: floodgate"));
        assert!(s.contains("broadcast-port: 15353"));
        assert!(s.contains("bedrock:\n    broadcast-port: 15353\n    use-haproxy-protocol: true"));
        assert!(s.contains("java:\n    use-haproxy-protocol: false") || s.lines().filter(|l| l.contains("use-haproxy-protocol: false")).count() >= 1);
    }

    #[test]
    fn disables_only_default_skins_enabled() {
        let sample = r#"storage:
  defaultSkins:
    enabled: true
    applyForPremium: false
    list:
      - "steve"
commands:
  forceDefaultPermissions: true
"#;
        let next = disable_sr_default_skins(sample);
        assert!(next.contains("defaultSkins:\n    enabled: false"));
        assert!(next.contains("forceDefaultPermissions: true"));
        assert!(!next.contains("defaultSkins:\n    enabled: true"));
    }

    #[test]
    fn enables_elyby_and_clears_no_connections() {
        let sample = r#"api:
  mineSkinApiKey: ""
  fetchRecommendedSkins: true
advanced:
  disableOnJoinSkins: false
  noConnections: true
"#;
        let next = enable_sr_elyby(sample);
        assert!(next.contains("elyByEnabled: true"));
        assert!(next.contains("mineSkinApiKey: \"\""));
        assert!(next.contains("noConnections: false"));
        assert!(!next.contains("noConnections: true"));
    }

    #[test]
    fn flips_elyby_false_to_true() {
        let sample = "api:\n  elyByEnabled: false\n";
        let next = enable_sr_elyby(sample);
        assert!(next.contains("elyByEnabled: true"));
        assert!(!next.contains("elyByEnabled: false"));
    }
}
