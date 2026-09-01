//! Discord Overlay auto-detecta `Minecraft.exe`, no un `javaw.exe` suelto
//! (Temurin/Adoptium). Clonamos el runtime a `Minecraft.exe` por cada Java.

use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

use crate::core::paths;

/// Ejecutable con el que Discord reconoce Minecraft. Si no se puede clonar,
/// se devuelve el `javaw` original.
pub fn overlay_executable(java: &Path) -> PathBuf {
    #[cfg(not(target_os = "windows"))]
    {
        return java.to_path_buf();
    }
    #[cfg(target_os = "windows")]
    {
        alias_minecraft_exe(java).unwrap_or_else(|| java.to_path_buf())
    }
}

#[cfg(target_os = "windows")]
fn alias_minecraft_exe(java: &Path) -> Option<PathBuf> {
    let name = java.file_name()?.to_str()?;
    if name.eq_ignore_ascii_case("Minecraft.exe") {
        return Some(java.to_path_buf());
    }
    if !java.is_file() {
        return None;
    }
    let mut hasher = Sha1::new();
    hasher.update(java.to_string_lossy().as_bytes());
    if let Ok(meta) = java.metadata() {
        hasher.update(meta.len().to_le_bytes());
    }
    let hex = hex::encode(hasher.finalize());
    let dir = paths::data_dir().join("java-overlay").join(&hex[..16]);
    let dest = dir.join("Minecraft.exe");
    if dest.is_file() {
        if let (Ok(a), Ok(b)) = (dest.metadata(), java.metadata()) {
            if a.len() == b.len() {
                return Some(dest);
            }
        }
    }
    std::fs::create_dir_all(&dir).ok()?;
    let _ = std::fs::remove_file(&dest);
    if std::fs::hard_link(java, &dest).is_err() {
        std::fs::copy(java, &dest).ok()?;
    }
    dest.is_file().then_some(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_java_file_returns_original() {
        let p = PathBuf::from("Z:/no-such-java/javaw.exe");
        assert_eq!(overlay_executable(&p), p);
    }
}
