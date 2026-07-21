//! Motor de optimización dinámica: gama de PC (Baja/Media/Alta) + versión/loader
//! → argumentos JVM extra y perfil de `options.txt`, aplicados justo antes de lanzar
//! el proceso Java.
//!
//! Los presets PvP (`pvp_jvm` para 1.8.9 Forge+OptiFine, `modern_pvp_jvm` para
//! 1.21.11 Fabric+Sodium+Iris) ya tienen su propio tuning fino por gama; este módulo
//! los reutiliza tal cual y agrega el mismo tipo de diferenciación para el resto de
//! instancias (vanilla/Forge/Fabric/NeoForge genéricos), que antes recibían siempre
//! los mismos flags de GC sin importar la RAM/CPU detectada.

use std::path::Path;

use crate::core::performance;
use crate::error::AppResult;
use crate::models::AppSettings;

use super::{modern_pvp_jvm, pvp_jvm};

fn loader_is_pvp_189(loader: &str, mc_version: &str, java_major: u32) -> bool {
    pvp_jvm::applies(loader, mc_version, java_major)
}

fn loader_is_pvp_modern(loader: &str, mc_version: &str, java_major: u32) -> bool {
    modern_pvp_jvm::applies(loader, mc_version, java_major)
}

/// JVM args extra por gama para instancias **genéricas** (no PvP): los presets PvP ya
/// tienen su propio tuning fino y no deben duplicarse acá. Java 8 no soporta varias de
/// estas flags (G1HeapRegionSize dinámico, etc.) así que se deja tal cual estaba.
fn generic_tier_jvm_args(tier: &str, java_major: u32) -> Vec<String> {
    if java_major <= 8 {
        return Vec::new();
    }
    match tier {
        "alta" => vec![
            "-XX:G1HeapRegionSize=16M".into(),
            "-XX:MaxGCPauseMillis=30".into(),
        ],
        "media" => vec![
            "-XX:G1HeapRegionSize=8M".into(),
            "-XX:MaxGCPauseMillis=50".into(),
        ],
        _ => vec![
            "-XX:MaxGCPauseMillis=100".into(),
            "-XX:ParallelGCThreads=2".into(),
            "-XX:ConcGCThreads=1".into(),
        ],
    }
}

/// Argumentos JVM adicionales según gama de PC + versión/loader. Para los presets PvP
/// (que ya diferencian 1.8.9 Forge+OptiFine de 1.21.11 Fabric+Sodium+Iris con su propio
/// tuning completo) devuelve vacío para no pisar esos flags.
pub fn extra_jvm_args_for(tier: &str, loader: &str, mc_version: &str, java_major: u32) -> Vec<String> {
    if loader_is_pvp_189(loader, mc_version, java_major) || loader_is_pvp_modern(loader, mc_version, java_major) {
        return Vec::new();
    }
    generic_tier_jvm_args(tier, java_major)
}

/// Aplica el perfil de `options.txt` correspondiente (diferenciado por loader + gama)
/// cuando el usuario tiene activo «Optimizar gráficos». Antes esto era un único preset
/// fijo para todas las instancias; ahora distingue 1.8.9 Forge+OptiFine, 1.21.11
/// Fabric+Sodium+Iris y el resto (vanilla/Forge/Fabric genéricos), cada uno con su
/// tabla de valores ya afinada por gama en `core::performance`.
pub fn apply_graphics_profile(game_dir: &Path, loader: &str, tier: &str) -> AppResult<()> {
    let loader_l = loader.trim().to_lowercase();
    if loader_l.contains("paraguacraft-pvp-modern") {
        performance::optimize_modern_pvp_options(game_dir, tier)?;
        performance::apply_modern_pvp_mod_configs(game_dir, tier)?;
    } else {
        // 1.8.9 Forge+OptiFine y el resto de loaders comparten la tabla generica por
        // gama (`optimize_instance_options`), mas agresiva que el viejo preset fijo.
        performance::optimize_instance_options(game_dir)?;
    }
    Ok(())
}

/// Punto único de entrada, invocado justo antes de lanzar el proceso Java:
/// limpieza de logs/crash-reports (si está activada) + perfil de gráficos por
/// gama/loader (si está activado). Reemplaza la lógica que antes estaba repartida
/// e inconsistente entre distintos comandos.
pub fn apply_pre_launch(game_dir: &Path, loader: &str, tier: &str, settings: &AppSettings) {
    if settings.deep_clean_on_launch {
        let _ = crate::core::extras::maintenance::run("both");
    }
    if settings.optimize_graphics {
        let _ = apply_graphics_profile(game_dir, loader, tier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pvp_presets_are_not_duplicated() {
        assert!(extra_jvm_args_for("alta", "paraguacraft-pvp", "1.8.9", 8).is_empty());
        assert!(extra_jvm_args_for("alta", "paraguacraft-pvp-modern", "1.21.11", 21).is_empty());
    }

    #[test]
    fn generic_instances_get_tier_flags_on_java9plus() {
        assert!(!extra_jvm_args_for("alta", "fabric", "1.21.11", 21).is_empty());
        assert!(!extra_jvm_args_for("baja", "forge", "1.20.1", 17).is_empty());
    }

    #[test]
    fn java8_generic_instances_get_no_extra_flags() {
        assert!(extra_jvm_args_for("alta", "forge", "1.8.9", 8).is_empty());
    }
}
