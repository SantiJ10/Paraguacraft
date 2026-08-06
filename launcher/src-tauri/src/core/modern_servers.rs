//! Servidores predefinidos para el menú multijugador PvP 1.21.11.

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
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerListFile {
    version: u32,
    servers: Vec<ServerEntry>,
}

fn default_servers() -> Vec<ServerEntry> {
    vec![
        ServerEntry {
            name: "Hypixel".into(),
            address: "mc.hypixel.net".into(),
            description: "BedWars · SkyWars · Duels".into(),
        },
        ServerEntry {
            name: "CubeCraft".into(),
            address: "play.cubecraft.net".into(),
            description: "EggWars · SkyWars".into(),
        },
        ServerEntry {
            name: "Regorland".into(),
            address: "regorland.net".into(),
            description: "Survival · PvP latino".into(),
        },
        ServerEntry {
            name: "UniversoCraft".into(),
            address: "mc.universocraft.net".into(),
            description: "SkyWars · BedWars LATAM".into(),
        },
        ServerEntry {
            name: "Mush".into(),
            address: "mush.com.br".into(),
            description: "PvP · BedWars Brasil".into(),
        },
    ]
}

/// Escribe `paraguacraft_servers.json` para que el mod muestre servidores en multijugador.
/// Solo se invoca en instancias PvP modern (1.21.11).
pub fn write_default_servers(instance_dir: &Path) -> AppResult<()> {
    let file = ServerListFile {
        version: 3,
        servers: default_servers(),
    };
    let path = instance_dir.join("paraguacraft_servers.json");
    let body = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, format!("{body}\n"))?;
    Ok(())
}
