//! Clasificación de versión MC → perfil de resource pack + pack_format exacto.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
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
            patch: parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
        }
    } else if let Some(m) = parts.get(1).and_then(|s| s.parse().ok()) {
        McVersion {
            major: m,
            minor: parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            patch: parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
        }
    } else {
        McVersion {
            major: 20,
            minor: 0,
            patch: 0,
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

/// `pack_format` exacto para que el Brand Pack no figure como incompatible.
/// Tabla alineada con minecraft.wiki / MineVinyl.
pub fn pack_format_for_mc(mc: &str) -> u32 {
    let v = parse_mc_version(mc);
    if v.major >= 26 {
        return match v.minor {
            0 => 84,
            1 => 84,
            _ => 88,
        };
    }
    match (v.major, v.minor) {
        (0..=5, _) => 1, // classic texture packs ignore this
        (6..=8, _) => 1,
        (9..=10, _) => 2,
        (11..=12, _) => 3,
        (13..=14, _) => 4,
        (15, _) => 5,
        (16, 0..=1) => 5,
        (16, _) => 6,
        (17, _) => 7,
        (18, _) => 8,
        (19, 0..=2) => 9,
        (19, 3) => 12,
        (19, _) => 13,
        (20, 0..=1) => 15,
        (20, 2) => 18,
        (20, 3..=4) => 22,
        (20, _) => 32,
        (21, 0..=1) => 34,
        (21, 2..=3) => 42,
        (21, 4) => 46,
        (21, 5) => 55,
        (21, 6) => 63,
        (21, 7..=8) => 64,
        (21, 9..=10) => 69,
        (21, 11..) => 75,
        _ => 15,
    }
}

/// JSON de `pack.mcmeta` compatible con la versión concreta de MC.
pub fn pack_mcmeta_json(mc: &str) -> String {
    let fmt = pack_format_for_mc(mc);
    let v = parse_mc_version(mc);
    // 1.21.9+ / formatos altos: schema min_format / max_format.
    if fmt >= 65 || v.major >= 26 || (v.major == 21 && v.minor >= 9) {
        return format!(
            r#"{{"pack":{{"pack_format":{fmt},"description":"Marca Oficial Paraguacraft","min_format":[{fmt},0],"max_format":[{fmt},0]}}}}"#
        );
    }
    // 1.20.2+ entiende supported_formats (evita warnings en rangos cercanos).
    if fmt >= 18 {
        return format!(
            r#"{{"pack":{{"pack_format":{fmt},"description":"Marca Oficial Paraguacraft","supported_formats":[{fmt},{fmt}]}}}}"#
        );
    }
    format!(r#"{{"pack":{{"pack_format":{fmt},"description":"Marca Oficial Paraguacraft"}}}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_key_versions() {
        assert_eq!(pack_format_for_mc("1.8.9"), 1);
        assert_eq!(pack_format_for_mc("1.12.2"), 3);
        assert_eq!(pack_format_for_mc("1.18.2"), 8);
        assert_eq!(pack_format_for_mc("1.20.1"), 15);
        assert_eq!(pack_format_for_mc("1.21.11"), 75);
        assert_eq!(pack_format_for_mc("26.2"), 88);
    }
}
