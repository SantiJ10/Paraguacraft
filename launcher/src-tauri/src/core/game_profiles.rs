//! Perfiles de juego 1 clic (sin social): orquestan instancia + Competir + servidor.

use crate::core::instances;
use crate::core::loaders;
use crate::error::{AppError, AppResult};
use crate::models::Instance;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub instance_id: Option<String>,
    /// Filtra instancias si `instance_id` es None (ej. `paraguacraft-pvp`).
    #[serde(default)]
    pub loader_filter: Option<String>,
    #[serde(default)]
    pub compete_mode: bool,
    #[serde(default)]
    pub server_address: Option<String>,
    #[serde(default)]
    pub builtin: bool,
    #[serde(default)]
    pub available: bool,
    #[serde(default)]
    pub resolved_instance_name: Option<String>,
}

pub fn list() -> Vec<GameProfile> {
    let instances = instances::list_local();
    builtins()
        .into_iter()
        .map(|mut p| {
            enrich(&mut p, &instances);
            p
        })
        .collect()
}

pub fn resolve(id: &str) -> AppResult<GameProfile> {
    let instances = instances::list_local();
    let mut profile = builtins()
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::msg("Perfil de juego no encontrado"))?;
    enrich(&mut profile, &instances);
    if !profile.available {
        return Err(AppError::msg(format!(
            "No hay instancia compatible para «{}».",
            profile.name
        )));
    }
    Ok(profile)
}

pub fn resolve_instance_id(profile: &GameProfile) -> AppResult<String> {
    if let Some(ref id) = profile.instance_id {
        if instances::read_meta(id).is_some() || instances::resolve_meta(id).is_some() {
            return Ok(id.clone());
        }
    }
    let filter = profile.loader_filter.as_deref().unwrap_or("vanilla");
    let all = instances::list_local();
    pick_instance(filter, &all)
        .map(|i| i.id.clone())
        .ok_or_else(|| {
            AppError::msg(format!(
                "Creá una instancia con loader «{}» para usar este perfil.",
                filter.replace('-', " ")
            ))
        })
}

fn enrich(profile: &mut GameProfile, instances: &[Instance]) {
    if let Some(ref id) = profile.instance_id {
        if let Some(inst) = instances.iter().find(|i| i.id == *id) {
            profile.available = true;
            profile.resolved_instance_name = Some(inst.name.clone());
            return;
        }
    }
    if let Some(filter) = profile.loader_filter.as_deref() {
        if let Some(inst) = pick_instance(filter, instances) {
            profile.instance_id = Some(inst.id.clone());
            profile.available = true;
            profile.resolved_instance_name = Some(inst.name.clone());
            return;
        }
    }
    profile.available = false;
    profile.resolved_instance_name = None;
}

fn pick_instance<'a>(loader_filter: &str, instances: &'a [Instance]) -> Option<&'a Instance> {
    let norm = loaders::normalize(loader_filter);
    let mut matches: Vec<&Instance> = instances
        .iter()
        .filter(|i| loaders::normalize(&i.loader) == norm)
        .collect();
    if matches.is_empty() {
        return None;
    }
    matches.sort_by(|a, b| {
        let ta = a.total_play_minutes;
        let tb = b.total_play_minutes;
        tb.cmp(&ta).then_with(|| a.name.cmp(&b.name))
    });
    Some(matches[0])
}

fn builtins() -> Vec<GameProfile> {
    vec![
        GameProfile {
            id: "hypixel-pvp".into(),
            name: "Hypixel PvP".into(),
            description: "Modo Competir + cliente PvP + mc.hypixel.net".into(),
            instance_id: None,
            loader_filter: Some("paraguacraft-pvp".into()),
            compete_mode: true,
            server_address: Some("mc.hypixel.net".into()),
            builtin: true,
            available: false,
            resolved_instance_name: None,
        },
        GameProfile {
            id: "pvp-practice".into(),
            name: "PvP práctica".into(),
            description: "Modo Competir en singleplayer o servidor favorito".into(),
            instance_id: None,
            loader_filter: Some("paraguacraft-pvp".into()),
            compete_mode: true,
            server_address: None,
            builtin: true,
            available: false,
            resolved_instance_name: None,
        },
        GameProfile {
            id: "vanilla-chill".into(),
            name: "Vanilla".into(),
            description: "Minecraft vanilla sin optimizaciones de competir".into(),
            instance_id: None,
            loader_filter: Some("vanilla".into()),
            compete_mode: false,
            server_address: None,
            builtin: true,
            available: false,
            resolved_instance_name: None,
        },
        GameProfile {
            id: "modpack".into(),
            name: "Modpack / Iris".into(),
            description: "Tu instancia Fabric + Iris más jugada".into(),
            instance_id: None,
            loader_filter: Some("fabric-iris".into()),
            compete_mode: false,
            server_address: None,
            builtin: true,
            available: false,
            resolved_instance_name: None,
        },
    ]
}
