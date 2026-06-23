//! Clasificación de versión MC → perfil de resource pack pre-generado.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McVersion {
    pub major: u32,
    pub minor: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackProfile {
    /// &lt; 1.6 — `texturepacks/*.zip`
    Classic,
    /// 1.6 – 1.15
    Legacy,
    /// 1.16 – 1.20.1
    Standard,
    /// 1.20.2 – 1.21.3
    StandardRange,
    /// 1.21.4 (textura wide, carpeta)
    Wide,
    /// 1.21.5+ / 26.x — zip en `resourcepacks/`
    Modern,
}

impl PackProfile {
    pub fn uses_zip_file(self) -> bool {
        matches!(self, Self::Classic | Self::Modern)
    }
}

pub fn parse_mc_version(version: &str) -> McVersion {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.first().is_some_and(|p| *p == "26") {
        McVersion {
            major: 26,
            minor: parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
        }
    } else if let Some(m) = parts.get(1).and_then(|s| s.parse().ok()) {
        McVersion {
            major: m,
            minor: parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
        }
    } else {
        McVersion {
            major: 20,
            minor: 0,
        }
    }
}

pub fn pack_profile(ver: McVersion) -> PackProfile {
    if ver.major < 6 {
        PackProfile::Classic
    } else if ver.major < 16 {
        PackProfile::Legacy
    } else if ver.major >= 26 || (ver.major == 21 && ver.minor >= 5) {
        PackProfile::Modern
    } else if ver.major > 21 || (ver.major == 21 && ver.minor >= 4) {
        PackProfile::Wide
    } else if ver.major > 20 || (ver.major == 20 && ver.minor >= 2) {
        PackProfile::StandardRange
    } else {
        PackProfile::Standard
    }
}
