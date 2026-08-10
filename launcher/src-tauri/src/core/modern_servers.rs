//! Servidores predefinidos para el menú multijugador PvP (1.21.11 / launcher).

use std::path::Path;

use serde::Serialize;

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerEntry {
    name: String,
    address: String,
    #[serde(default)]
    description: String,
    /// Nota corta (ej. premium / offline).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerListFile {
    version: u32,
    /// Mensaje global para la UI del mod / launcher.
    #[serde(default)]
    offline_note: String,
    servers: Vec<ServerEntry>,
}

fn default_servers() -> Vec<ServerEntry> {
    vec![
        ServerEntry {
            name: "Hypixel".into(),
            address: "mc.hypixel.net".into(),
            description: "BedWars · SkyWars · Duels".into(),
            note: "Premium (cuenta Microsoft)".into(),
        },
        ServerEntry {
            name: "Minemen Club".into(),
            address: "na.minemen.club".into(),
            description: "Practice · Duels · Pots".into(),
            note: "Premium · anticheat estricto".into(),
        },
        ServerEntry {
            name: "CubeCraft".into(),
            address: "play.cubecraft.net".into(),
            description: "EggWars · SkyWars · Lucky".into(),
            note: "Premium".into(),
        },
        ServerEntry {
            name: "UniversoCraft".into(),
            address: "mc.universocraft.net".into(),
            description: "SkyWars · BedWars LATAM".into(),
            note: "Ver reglas del server (online-mode)".into(),
        },
        ServerEntry {
            name: "Mush".into(),
            address: "mush.com.br".into(),
            description: "PvP · BedWars BR".into(),
            note: "Ver reglas del server".into(),
        },
        ServerEntry {
            name: "Regorland".into(),
            address: "regorland.net".into(),
            description: "Survival · PvP latino".into(),
            note: "A menudo amicable a no-premium".into(),
        },
    ]
}

/// Escribe `paraguacraft_servers.json` (PvP modern multijugador + catálogo launcher).
pub fn write_default_servers(instance_dir: &Path) -> AppResult<()> {
    let file = ServerListFile {
        version: 4,
        offline_note: "Algunos servidores (Hypixel, Minemen, CubeCraft) exigen cuenta premium. Offline solo funciona donde el server lo permite.".into(),
        servers: default_servers(),
    };
    let path = instance_dir.join("paraguacraft_servers.json");
    let body = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, format!("{body}\n"))?;
    Ok(())
}

/// Lista plana para destinos de perfiles / UI.
#[allow(dead_code)]
pub fn catalog_addresses() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("hypixel", "mc.hypixel.net", "Hypixel"),
        ("minemen", "na.minemen.club", "Minemen Club"),
        ("cubecraft", "play.cubecraft.net", "CubeCraft"),
        ("universocraft", "mc.universocraft.net", "UniversoCraft"),
        ("mush", "mush.com.br", "Mush"),
        ("regorland", "regorland.net", "Regorland"),
    ]
}
