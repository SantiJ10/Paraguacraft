//! Mapa IP → asset Discord (estilo Lunar). Override opcional en data_dir.

use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;

use crate::core::paths;

pub const BASE_ASSET: &str = "paraguacraft_base";
pub const BASE_HOVER: &str = "Paraguacraft Launcher";

#[derive(Debug, Clone, Deserialize)]
struct AssetRuleJson {
    pattern: String,
    asset: String,
    name: String,
}

struct AssetRule {
    re: Regex,
    asset: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct RpcArt {
    pub large_image: String,
    pub large_text: String,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}

static RULES: LazyLock<Vec<AssetRule>> = LazyLock::new(load_rules);

fn load_rules() -> Vec<AssetRule> {
    let override_path = paths::data_dir().join("servers_assets.json");
    let raw = std::fs::read_to_string(&override_path)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| include_str!("servers_assets.json").to_string());
    let parsed: Vec<AssetRuleJson> = serde_json::from_str(&raw).unwrap_or_default();
    parsed
        .into_iter()
        .filter_map(|r| {
            let re = Regex::new(&r.pattern).ok()?;
            Some(AssetRule {
                re,
                asset: r.asset,
                name: r.name,
            })
        })
        .collect()
}

pub fn lookup(host: &str) -> Option<(String, String)> {
    let h = host.trim();
    for rule in RULES.iter() {
        if rule.re.is_match(h) {
            return Some((rule.asset.clone(), rule.name.clone()));
        }
    }
    None
}

/// Arte in-game estilo Badlion:
/// servidor conocido → logo del server grande + `paraguacraft_base` chico.
/// menú / un jugador / IP desconocida / Playit → logo del launcher grande.
pub fn art_for_host(host: Option<&str>) -> RpcArt {
    if let Some(host) = host.filter(|h| !h.is_empty()) {
        if let Some((asset, name)) = lookup(host) {
            return RpcArt {
                large_image: asset,
                large_text: name,
                small_image: Some(BASE_ASSET.into()),
                small_text: Some(BASE_HOVER.into()),
            };
        }
    }
    RpcArt {
        large_image: BASE_ASSET.into(),
        large_text: BASE_HOVER.into(),
        small_image: None,
        small_text: None,
    }
}

pub fn pretty_name(host: &str) -> String {
    if let Some((_, name)) = lookup(host) {
        return name;
    }
    host.trim().trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hypixel_maps_to_logo() {
        let art = art_for_host(Some("mc.hypixel.net"));
        assert_eq!(art.large_image, "logo_hypixel");
        assert_eq!(art.large_text, "Hypixel");
        assert_eq!(art.small_image.as_deref(), Some(BASE_ASSET));
        assert_eq!(art.small_text.as_deref(), Some(BASE_HOVER));
        assert_eq!(pretty_name("mc.hypixel.net"), "Hypixel");
    }

    #[test]
    fn unknown_keeps_base() {
        let art = art_for_host(Some("play.randomsmp.net"));
        assert_eq!(art.large_image, BASE_ASSET);
        assert_eq!(art.large_text, BASE_HOVER);
        assert!(art.small_image.is_none());
    }

    #[test]
    fn menu_uses_launcher_logo() {
        let art = art_for_host(None);
        assert_eq!(art.large_image, BASE_ASSET);
        assert_eq!(art.large_text, BASE_HOVER);
        assert!(art.small_image.is_none());
    }
}
