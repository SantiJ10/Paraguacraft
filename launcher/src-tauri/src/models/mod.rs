//! Modelos compartidos con la UI. `rename_all = "camelCase"` para que el JSON
//! coincida con los tipos TypeScript de `src/lib/types.ts`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub ram_gb: f64,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub cpu_name: String,
    pub gpu_name: String,
    pub os: String,
    pub arch: String,
    /// "baja" | "media" | "alta"
    pub perfil_sugerido: String,
    pub recommended_ram_mb: u32,
    /// "G1GC" | "ZGC" | ...
    pub recommended_gc: String,
}
