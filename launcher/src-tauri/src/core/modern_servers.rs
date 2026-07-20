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
            address: "m.cubecraft.net".into(),
            description: "EggWars · SkyWars".into(),
        },
        ServerEntry {
            name: "Regorland".into(),
            address: "regorland.net".into(),
            description: "Survival · PvP latino".into(),
        },
        ServerEntry {
            name: "Hylex".into(),
            address: "original.hylex.net".into(),
            description: "Minijuegos competitivos".into(),
        },
        ServerEntry {
            name: "MineLatino".into(),
            address: "play.minelatino.net".into(),
            description: "Comunidad hispanohablante".into(),
        },
    ]
}

/// Escribe `paraguacraft_servers.json` para que el mod muestre servidores en multijugador.
pub fn write_default_servers(instance_dir: &Path) -> AppResult<()> {
    let file = ServerListFile {
        version: 1,
        servers: default_servers(),
    };
    let path = instance_dir.join("paraguacraft_servers.json");
    let body = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, format!("{body}\n"))?;
    Ok(())
}
