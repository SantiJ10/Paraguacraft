//! Perfiles de juego 1 clic (sin social): orquestan instancia + Competir + servidor.

use crate::core::favorites;
use crate::core::instances;
use crate::core::loaders;
use crate::core::modern_pvp;
use crate::error::{AppError, AppResult};
use crate::models::Instance;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameProfileDestination {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_address: Option<String>,
    #[serde(default)]
    pub needs_favorite: bool,
}

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub destinations: Vec<GameProfileDestination>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mod_pack_summary: Option<String>,
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
    if profile.id == "modern-pvp" {
        return Err(AppError::msg(
            "Usá ensure_modern_pvp antes de lanzar el perfil 1.21.11.",
        ));
    }
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

/// Resuelve la dirección de servidor según destino elegido en la UI.
pub fn resolve_server(
    profile_id: &str,
    destination_id: &str,
    favorite_id: Option<&str>,
) -> AppResult<Option<String>> {
    match destination_id {
        "menu" => Ok(None),
        "training" => Ok(None),
        "hypixel" => Ok(Some("mc.hypixel.net".into())),
        "cubecraft" => Ok(Some("m.cubecraft.net".into())),
        "minelatino" => Ok(Some("play.minelatino.net".into())),
        "librecraft" => Ok(Some("librecraft.gg".into())),
        "hylex" => Ok(Some("hylex.net".into())),
        "favorite" => {
            let favs = favorites::list();
            if favs.is_empty() {
                return Err(AppError::msg(
                    "Agregá un servidor favorito en Inicio para usar este destino.",
                ));
            }
            let fav = favorite_id
                .and_then(|id| favs.iter().find(|f| f.id == id))
                .or(favs.first())
                .ok_or_else(|| AppError::msg("Servidor favorito no encontrado"))?;
            Ok(Some(fav.address.clone()))
        }
        other => {
            if profile_id == "modern-pvp" {
                return Err(AppError::msg(format!("Destino desconocido: {other}")));
            }
            Err(AppError::msg(format!("Destino desconocido: {other}")))
        }
    }
}

fn enrich(profile: &mut GameProfile, instances: &[Instance]) {
    if profile.id == "modern-pvp" {
        let tier = modern_pvp::tier_from_hardware();
        profile.hardware_tier = Some(tier.clone());
        profile.mod_pack_summary = Some(modern_pvp::pack_summary(&tier));
        profile.available = true;
        if let Some(inst) = modern_pvp::find_instance(instances) {
            profile.instance_id = Some(inst.id.clone());
            profile.resolved_instance_name = Some(inst.name.clone());
        } else {
            profile.resolved_instance_name = Some("Se creará al iniciar".into());
        }
        return;
    }

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

fn pvp_destinations() -> Vec<GameProfileDestination> {
    vec![
        GameProfileDestination {
            id: "hypixel".into(),
            label: "Hypixel".into(),
            description: Some("mc.hypixel.net · BedWars / SkyWars 1.8".into()),
            server_address: Some("mc.hypixel.net".into()),
            needs_favorite: false,
        },
        GameProfileDestination {
            id: "favorite".into(),
            label: "Servidor favorito".into(),
            description: Some("Tu IP guardada en Inicio".into()),
            server_address: None,
            needs_favorite: true,
        },
        GameProfileDestination {
            id: "menu".into(),
            label: "Solo menú".into(),
            description: Some("Entrenamiento liviano sin auto-conectar".into()),
            server_address: None,
            needs_favorite: false,
        },
        GameProfileDestination {
            id: "training".into(),
            label: "Práctica PvP".into(),
            description: Some("Mundo flat local · reach/combo HUD · reglas PvP".into()),
            server_address: None,
            needs_favorite: false,
        },
    ]
}

fn modern_destinations() -> Vec<GameProfileDestination> {
    vec![
        GameProfileDestination {
            id: "hypixel".into(),
            label: "Hypixel".into(),
            description: Some("mc.hypixel.net".into()),
            server_address: Some("mc.hypixel.net".into()),
            needs_favorite: false,
        },
        GameProfileDestination {
            id: "cubecraft".into(),
            label: "CubeCraft".into(),
            description: Some("m.cubecraft.net".into()),
            server_address: Some("m.cubecraft.net".into()),
            needs_favorite: false,
        },
        GameProfileDestination {
            id: "minelatino".into(),
            label: "MineLatino".into(),
            description: Some("play.minelatino.net".into()),
            server_address: Some("play.minelatino.net".into()),
            needs_favorite: false,
        },
        GameProfileDestination {
            id: "librecraft".into(),
            label: "LibreCraft".into(),
            description: Some("librecraft.gg".into()),
            server_address: Some("librecraft.gg".into()),
            needs_favorite: false,
        },
        GameProfileDestination {
            id: "hylex".into(),
            label: "Hylex".into(),
            description: Some("hylex.net".into()),
            server_address: Some("hylex.net".into()),
            needs_favorite: false,
        },
        GameProfileDestination {
            id: "favorite".into(),
            label: "Servidor favorito".into(),
            description: None,
            server_address: None,
            needs_favorite: true,
        },
        GameProfileDestination {
            id: "training".into(),
            label: "Práctica PvP".into(),
            description: Some("Mundo flat local · HUD de entrenamiento".into()),
            server_address: None,
            needs_favorite: false,
        },
        GameProfileDestination {
            id: "menu".into(),
            label: "Solo menú".into(),
            description: None,
            server_address: None,
            needs_favorite: false,
        },
    ]
}

fn builtins() -> Vec<GameProfile> {
    vec![
        GameProfile {
            id: "pvp-compete".into(),
            name: "PvP 1.8.9 Competir".into(),
            description: "Cliente Paraguacraft PvP + Modo Competir. Elegí destino abajo.".into(),
            instance_id: None,
            loader_filter: Some("paraguacraft-pvp".into()),
            compete_mode: true,
            server_address: None,
            builtin: true,
            available: false,
            resolved_instance_name: None,
            destinations: pvp_destinations(),
            hardware_tier: None,
            mod_pack_summary: None,
        },
        GameProfile {
            id: "modern-pvp".into(),
            name: "Paraguacraft PvP 1.21.11".into(),
            description: "Cliente PvP dedicado (loader propio) + mod Paraguacraft. Distinto de Fabric+Iris genérico.".into(),
            instance_id: None,
            loader_filter: Some("paraguacraft-pvp-modern".into()),
            compete_mode: false,
            server_address: None,
            builtin: true,
            available: true,
            resolved_instance_name: None,
            destinations: modern_destinations(),
            hardware_tier: None,
            mod_pack_summary: None,
        },
        GameProfile {
            id: "fabric-iris-chill".into(),
            name: "Fabric + Iris".into(),
            description: "Optimización shaders/rendimiento sin cliente PvP Paraguacraft.".into(),
            instance_id: None,
            loader_filter: Some("fabric-iris".into()),
            compete_mode: false,
            server_address: None,
            builtin: true,
            available: false,
            resolved_instance_name: None,
            destinations: vec![GameProfileDestination {
                id: "menu".into(),
                label: "Solo menú".into(),
                description: None,
                server_address: None,
                needs_favorite: false,
            }],
            hardware_tier: None,
            mod_pack_summary: None,
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
            destinations: Vec::new(),
            hardware_tier: None,
            mod_pack_summary: None,
        },
    ]
}
