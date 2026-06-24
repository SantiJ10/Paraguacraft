//! Extras de rendimiento (Game Mode, Turbo, mantenimiento, Discord RPC).

pub mod discord_rpc;
pub mod game_presence;
pub mod game_mode;
pub mod java_priority;
pub mod maintenance;
pub mod turbo;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtrasStatus {
    pub game_mode_active: bool,
    pub turbo_active: bool,
    pub java_priority: String,
}

pub fn status() -> ExtrasStatus {
    ExtrasStatus {
        game_mode_active: game_mode::is_active(),
        turbo_active: turbo::is_active(),
        java_priority: java_priority::current_level(),
    }
}
