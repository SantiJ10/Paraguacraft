//! Perfil JVM para **Paraguacraft PvP 1.21.11** (Java 21 + Fabric).
//!
//! A diferencia de 1.8.9 (tope 4 GB en Java 8), 1.21.11 puede usar más heap,
//! ZGC generacional y flags modernos sin penalizar pausas del recolector.

/// ¿Aplica el preset PvP moderno?
pub fn applies(loader: &str, mc_version: &str, java_major: u32) -> bool {
    let loader = loader.trim().to_lowercase();
    (loader == "paraguacraft-pvp-modern" || loader.contains("paraguacraft-pvp-modern"))
        && mc_version.trim() == "1.21.11"
        && java_major >= 21
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Baja,
    Media,
    Alta,
}

pub fn tier_from_ram_gb(ram_gb: f64) -> Tier {
    if ram_gb <= 8.0 {
        Tier::Baja
    } else if ram_gb <= 16.0 {
        Tier::Media
    } else {
        Tier::Alta
    }
}

/// RAM fija (Xms = Xmx). 1.21.11 + Sodium/Iris se benefician de más heap que 1.8.9.
pub fn resolve_ram_mb(ram_gb: f64) -> u32 {
    if ram_gb <= 6.0 {
        3072
    } else if ram_gb <= 8.0 {
        4096
    } else if ram_gb <= 16.0 {
        6144
    } else {
        8192
    }
}

fn g1_flags_baja() -> Vec<&'static str> {
    vec![
        "-XX:+UseG1GC",
        "-XX:MaxGCPauseMillis=30",
        "-XX:G1NewSizePercent=25",
        "-XX:G1ReservePercent=20",
        "-XX:MaxTenuringThreshold=1",
        "-XX:+UseStringDeduplication",
    ]
}

#[allow(dead_code)]
fn g1_flags_media_alta() -> Vec<&'static str> {
    vec![
        "-XX:+UseG1GC",
        "-XX:MaxGCPauseMillis=40",
        "-XX:G1NewSizePercent=30",
        "-XX:G1MaxNewSizePercent=40",
        "-XX:G1HeapRegionSize=8M",
        "-XX:G1ReservePercent=20",
        "-XX:G1MixedGCCountTarget=4",
        "-XX:InitiatingHeapOccupancyPercent=15",
        "-XX:MaxTenuringThreshold=1",
        "-XX:+PerfDisableSharedMem",
        "-XX:+UseStringDeduplication",
    ]
}

fn zgc_flags() -> Vec<&'static str> {
    vec![
        "-XX:+UseZGC",
        "-XX:+ZGenerational",
        "-XX:+AlwaysPreTouch",
        "-XX:+UseStringDeduplication",
    ]
}

/// Igual que [`resolve_ram_mb`] pero sin pasarse del techo seguro del sistema.
///
/// `ceiling_mb` es la RAM ya acotada en `commands::launch` (deja ~1.5 GB para el
/// SO). Sin esto, en una máquina de 4 GB el preset pedía 3 GB de heap más lo que
/// Minecraft reserva fuera del heap, y el equipo terminaba en swap.
pub fn resolve_ram_mb_capped(ram_gb: f64, ceiling_mb: u32) -> u32 {
    resolve_ram_mb(ram_gb).min(ceiling_mb.max(1024))
}

/// `-Xms`/`-Xmx` + GC tuneado por gama (Java 21+).
pub fn build_jvm_args(ram_gb: f64, ceiling_mb: u32) -> Vec<String> {
    let ram_mb = resolve_ram_mb_capped(ram_gb, ceiling_mb);
    let tier = tier_from_ram_gb(ram_gb);
    let mut args = vec![
        format!("-Xms{ram_mb}M"),
        format!("-Xmx{ram_mb}M"),
        "-XX:+UnlockExperimentalVMOptions".into(),
        "-XX:+DisableExplicitGC".into(),
    ];

    // ZGC rinde mal en heaps chicos, así que además de la gama se mira el heap
    // real: si el techo seguro del sistema lo recortó, conviene volver a G1.
    let flags = match tier {
        Tier::Baja => g1_flags_baja(),
        // Java 21+: ZGC generacional en media/alta (pausas sub-ms vs G1).
        Tier::Media | Tier::Alta if ram_mb >= 4096 => zgc_flags(),
        _ => g1_flags_baja(),
    };
    args.extend(flags.iter().map(|s| (*s).to_string()));

    if matches!(tier, Tier::Media) && ram_gb > 8.0 {
        args.push("-XX:+AlwaysPreTouch".into());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_ram_above_189_cap() {
        assert_eq!(resolve_ram_mb(4.0), 3072);
        assert_eq!(resolve_ram_mb(8.0), 4096);
        assert_eq!(resolve_ram_mb(16.0), 6144);
        assert_eq!(resolve_ram_mb(32.0), 8192);
    }

    #[test]
    fn respects_safe_ram_ceiling() {
        // 4 GB de sistema: el techo seguro manda sobre el preset.
        assert_eq!(resolve_ram_mb_capped(4.0, 2048), 2048);
        // Con margen de sobra, el preset queda intacto.
        assert_eq!(resolve_ram_mb_capped(32.0, 12288), 8192);
    }

    #[test]
    fn falls_back_to_g1_when_heap_is_capped_small() {
        // Sistema de 16 GB (gama media) pero heap recortado: ZGC no conviene.
        let args = build_jvm_args(16.0, 3072);
        assert!(args.iter().any(|a| a == "-XX:+UseG1GC"));
        assert!(!args.iter().any(|a| a == "-XX:+UseZGC"));
    }

    #[test]
    fn applies_only_modern_java21() {
        assert!(applies("paraguacraft-pvp-modern", "1.21.11", 21));
        assert!(!applies("paraguacraft-pvp-modern", "1.21.11", 17));
        assert!(!applies("paraguacraft-pvp", "1.8.9", 21));
        assert!(!applies("fabric-iris", "1.21.11", 21));
    }
}
