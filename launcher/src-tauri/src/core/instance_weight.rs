//! Peso estimado de una instancia (liviano / medio / pesado).

use crate::config;
use crate::core::instances::{self, content};
use crate::core::loaders;
use crate::core::paths;
use crate::error::AppResult;
use crate::models::AppSettings;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceWeight {
    /// "liviano" | "medio" | "pesado"
    pub tier: String,
    pub label: String,
    pub score: u32,
    pub mod_count: u32,
    pub shader_count: u32,
    pub ram_mb: u32,
    pub mods_mb: u64,
    pub reasons: Vec<String>,
}

pub fn compute(instance_id: &str) -> AppResult<InstanceWeight> {
    let meta = instances::ensure_meta(instance_id)?;
    let settings: AppSettings = config::read_json(&paths::config_file()).unwrap_or_default();
    let ram = if meta.ram_mb > 0 {
        meta.ram_mb
    } else {
        settings.ram_mb
    };

    let entries = content::list(instance_id).unwrap_or_default();
    let mod_count = entries
        .iter()
        .filter(|e| e.folder == "mods" && e.enabled)
        .count() as u32;
    let shader_count = entries
        .iter()
        .filter(|e| e.folder == "shaderpacks" && e.enabled)
        .count() as u32;
    let mods_mb: u64 = entries
        .iter()
        .filter(|e| e.enabled && (e.folder == "mods" || e.folder == "shaderpacks"))
        .map(|e| e.size_bytes)
        .sum::<u64>()
        / (1024 * 1024);

    let loader = loaders::normalize(&meta.loader);
    let mut score: u32 = 0;
    let mut reasons = Vec::new();

    score += match ram {
        n if n >= 8192 => {
            reasons.push(format!("RAM asignada alta ({n} MB)"));
            3
        }
        n if n >= 4096 => {
            reasons.push(format!("RAM asignada media ({n} MB)"));
            2
        }
        n if n >= 3072 => 1,
        _ => 0,
    };

    score += match mod_count {
        n if n >= 40 => {
            reasons.push(format!("{n} mods activos"));
            4
        }
        n if n >= 20 => {
            reasons.push(format!("{n} mods activos"));
            3
        }
        n if n >= 8 => {
            reasons.push(format!("{n} mods activos"));
            2
        }
        n if n >= 1 => 1,
        _ => 0,
    };

    if shader_count > 0 {
        score += 2;
        reasons.push(format!(
            "{shader_count} shaderpack{} activo(s)",
            if shader_count == 1 { "" } else { "s" }
        ));
    }

    score += match loader.as_str() {
        "fabric-iris" => {
            reasons.push("Fabric + Iris".into());
            2
        }
        "paraguacraft-pvp" => 0,
        "forge" | "fabric" | "neoforge" => 1,
        _ => 0,
    };

    if mods_mb >= 512 {
        score += 1;
        reasons.push(format!("{mods_mb} MB en mods/shaders"));
    }

    let (tier, label) = match score {
        n if n >= 7 => ("pesado", "Pesado"),
        n if n >= 4 => ("medio", "Medio"),
        _ => ("liviano", "Liviano"),
    };

    if reasons.is_empty() {
        reasons.push("Poca carga de mods y RAM moderada".into());
    }

    Ok(InstanceWeight {
        tier: tier.into(),
        label: label.into(),
        score,
        mod_count,
        shader_count,
        ram_mb: ram,
        mods_mb,
        reasons,
    })
}
