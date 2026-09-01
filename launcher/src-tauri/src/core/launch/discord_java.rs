//! Discord Overlay auto-detecta Minecraft así:
//! 1. Proceso `javaw.exe` (no un Java renombrado: eso no está en su lista).
//! 2. La línea de comandos **visible** contiene `net.minecraft.client.main.Main`.
//!
//! Fabric/Forge/Quilt arrancan con otra main class (`KnotClient`, `Launch`, …).
//! En Java 9+ además metemos los args en `@args.txt`, y Discord no abre ese
//! archivo: hay que pasar el marcador **fuera** del argfile.

/// Flag JVM inofensivo. Discord solo busca el substring; Minecraft lo ignora.
pub const DETECT_FLAG: &str = "-Ddiscordfix=net.minecraft.client.main.Main";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_is_the_string_discord_scans_for() {
        assert!(DETECT_FLAG.contains("net.minecraft.client.main.Main"));
        assert!(DETECT_FLAG.starts_with("-D"));
        assert!(!DETECT_FLAG.contains(' '));
    }
}
