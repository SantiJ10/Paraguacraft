//! Perfil JVM para **Paraguacraft PvP 1.8.9** (Java 8 + Forge/OptiFine).
//!
//! En 1.8.9 asignar >4 GB suele empeorar pausas del recolector; el tope es 4096M.

/// ¿Aplica el preset PvP competitivo?
pub fn applies(loader: &str, mc_version: &str, java_major: u32) -> bool {
    let loader = loader.trim().to_lowercase();
    (loader == "paraguacraft-pvp" || loader.contains("paraguacraft-pvp"))
        && mc_version.trim() == "1.8.9"
        && java_major == 8
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

/// RAM fija (Xms = Xmx) según RAM total del sistema.
pub fn resolve_ram_mb(ram_gb: f64) -> u32 {
    let mb = if ram_gb <= 6.0 {
        2560
    } else if ram_gb <= 8.0 {
        3584
    } else {
        // >8 GB: tope 4 GB para 1.8.9 (media y alta comparten techo).
        4096
    };
    mb
}

fn g1_flags_baja() -> Vec<&'static str> {
    vec![
        "-XX:+UseG1GC",
        "-XX:MaxGCPauseMillis=20",
        "-XX:G1NewSizePercent=20",
        "-XX:G1ReservePercent=20",
        "-XX:MaxTenuringThreshold=1",
    ]
}

fn g1_flags_media_alta() -> Vec<&'static str> {
    vec![
        "-XX:+UseG1GC",
        "-XX:MaxGCPauseMillis=50",
        "-XX:G1NewSizePercent=30",
        "-XX:G1MaxNewSizePercent=40",
        "-XX:G1HeapRegionSize=8M",
        "-XX:G1ReservePercent=20",
        "-XX:G1HeapWastePercent=5",
        "-XX:G1MixedGCCountTarget=4",
        "-XX:InitiatingHeapOccupancyPercent=15",
        "-XX:MaxTenuringThreshold=1",
        "-XX:SurvivorRatio=32",
        "-XX:+PerfDisableSharedMem",
    ]
}

/// `-Xms`/`-Xmx` + G1 tuneado por gama (solo Java 8).
pub fn build_jvm_args(ram_gb: f64) -> Vec<String> {
    let ram_mb = resolve_ram_mb(ram_gb);
    let tier = tier_from_ram_gb(ram_gb);
    let mut args = vec![
        format!("-Xms{ram_mb}M"),
        format!("-Xmx{ram_mb}M"),
        "-XX:+DisableExplicitGC".into(),
    ];
    let flags = match tier {
        Tier::Baja => g1_flags_baja(),
        Tier::Media | Tier::Alta => g1_flags_media_alta(),
    };
    args.extend(flags.iter().map(|s| (*s).to_string()));
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_ram_for_189() {
        assert_eq!(resolve_ram_mb(4.0), 2560);
        assert_eq!(resolve_ram_mb(8.0), 3584);
        assert_eq!(resolve_ram_mb(32.0), 4096);
    }

    #[test]
    fn applies_only_pvp_java8() {
        assert!(applies("paraguacraft-pvp", "1.8.9", 8));
        assert!(!applies("paraguacraft-pvp", "1.8.9", 21));
        assert!(!applies("forge", "1.8.9", 8));
    }
}
