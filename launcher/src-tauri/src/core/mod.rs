//! Logica de dominio del launcher, separada por modulo.
//!
//! Fase 2 implementa: hardware, paths, java, accounts, instances (con
//! profiles/backups). Fases 3-4 completan downloads/loaders/servers/ai/etc.

pub mod hardware;
pub mod paths;

// Fase 2
pub mod accounts;
pub mod bedrock;
pub mod branding;
pub mod instances;
pub mod java;

// Fase 3
pub mod launch;
pub mod loaders;
pub mod net;
pub mod store;
pub mod versions;

// Fase 4
pub mod ai;
pub mod diagnostics;
pub mod extras;
pub mod favorites;
pub mod server_admin;
pub mod server_backups;
pub mod server_console;
pub mod server_hangar;
pub mod server_properties;
pub mod server_setup;
pub mod server_manager;
pub mod server_repair;
pub mod instance_repair;
pub mod servers;
pub mod skins;
pub mod spotify;
pub mod performance;
pub mod overlay_ipc;
pub mod updater;
