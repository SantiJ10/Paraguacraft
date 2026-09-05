//! Deshabilita mods de ventana (borderless/fullscreen) para que el hook HWND
//! del launcher tenga el control exclusivo. Conservador: no toca Sodium/Iris.

use std::path::Path;

/// Stems conocidos de mods que pelean el modo de ventana con el borderless OS.
const WINDOW_MOD_NEEDLES: &[&str] = &[
    "borderless-mining",
    "borderlessmining",
    "borderless-window",
    "borderlesswindow",
    "borderlessfullscreen",
    "borderless-fullscreen",
    "fullscreen-windowed",
    "fullscreenwindowed",
    "windowed-fullscreen",
    "windowedfullscreen",
    "desktop-fullscreen",
    "desktopfullscreen",
    "better-full-screen",
    "betterfullscreen",
    "window-tweaks",
    "windowtweaks",
];

/// True si el nombre de archivo (minúsculas) es un mod de ventana conflictivo.
pub fn is_conflicting_window_mod(filename: &str) -> bool {
    let n = filename.to_ascii_lowercase();
    if !n.ends_with(".jar") {
        return false;
    }
    if n.ends_with(".jar.disabled") {
        return false;
    }
    // Evitar falsos positivos (fullbright, sodium, iris, …).
    if n.contains("fullbright") || n.contains("sodium") || n.contains("iris") {
        return false;
    }
    WINDOW_MOD_NEEDLES.iter().any(|needle| n.contains(needle))
}

/// Renombra `*.jar` conflictivos a `*.jar.disabled` en `game_dir/mods`.
pub fn disable_conflicting(game_dir: &Path) -> std::io::Result<u32> {
    let mods = game_dir.join("mods");
    if !mods.is_dir() {
        return Ok(0);
    }
    let mut n = 0u32;
    for entry in std::fs::read_dir(&mods)? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_conflicting_window_mod(&name) {
            continue;
        }
        let dest = path.with_file_name(format!("{name}.disabled"));
        if dest.exists() {
            let _ = std::fs::remove_file(&dest);
        }
        match std::fs::rename(&path, &dest) {
            Ok(()) => {
                n += 1;
                eprintln!(
                    "[paraguacraft] window mod disabled: {} -> {}.disabled",
                    name, name
                );
            }
            Err(e) => {
                eprintln!(
                    "[paraguacraft] no se pudo deshabilitar {name}: {e}"
                );
            }
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_window_mods() {
        assert!(is_conflicting_window_mod("borderless-mining-1.1.5.jar"));
        assert!(is_conflicting_window_mod("BorderlessMining-1.1.1.jar"));
        assert!(is_conflicting_window_mod("fullscreen-windowed-1.0.jar"));
        assert!(is_conflicting_window_mod("WindowedFullscreen-forge.jar"));
        assert!(is_conflicting_window_mod("better-full-screen-1.2.jar"));
    }

    #[test]
    fn ignores_unrelated_and_already_disabled() {
        assert!(!is_conflicting_window_mod("sodium-fabric-0.6.jar"));
        assert!(!is_conflicting_window_mod("iris-1.8.jar"));
        assert!(!is_conflicting_window_mod("fullbright-1.0.jar"));
        assert!(!is_conflicting_window_mod("borderless-mining-1.1.5.jar.disabled"));
        assert!(!is_conflicting_window_mod("optifine_1.8.9.jar"));
        assert!(!is_conflicting_window_mod("readme.txt"));
    }

    #[test]
    fn disable_renames_jars() {
        let dir = std::env::temp_dir().join(format!(
            "pc_wmods_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let mods = dir.join("mods");
        std::fs::create_dir_all(&mods).unwrap();
        std::fs::write(mods.join("borderless-mining-1.1.jar"), b"x").unwrap();
        std::fs::write(mods.join("sodium-0.6.jar"), b"x").unwrap();
        let n = disable_conflicting(&dir).unwrap();
        assert_eq!(n, 1);
        assert!(mods.join("borderless-mining-1.1.jar.disabled").is_file());
        assert!(mods.join("sodium-0.6.jar").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
