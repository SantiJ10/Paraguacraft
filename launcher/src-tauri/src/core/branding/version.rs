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
    /// 1.16 – 1.19.x — atlas split 256×256
    Standard,
    /// 1.20.0 – 1.21.3 — minecraft.png wide 1024×256
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

fn parse_leading_uint(s: &str) -> Option<u32> {
    let digits: String = s.trim().chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u32>().ok()
    }
}

pub fn parse_mc_version(version: &str) -> McVersion {
    let raw = version.trim().to_ascii_lowercase();
    if raw.is_empty() {
        return McVersion {
            major: 21,
            minor: 11,
            patch: 0,
        };
    }

    // Betas, Alphas, Classic, Infdev, Indev, Pre-classic (b1.7.3, a1.2.6, c0.30, inf-..., rd-...)
    if raw.starts_with("b1.")
        || raw.starts_with("beta")
        || raw.starts_with("a1.")
        || raw.starts_with("a0.")
        || raw.starts_with("alpha")
        || raw.starts_with("c0.")
        || raw.starts_with("inf-")
        || raw.starts_with("in-")
        || raw.starts_with("rd-")
    {
        return McVersion {
            major: 5,
            minor: 0,
            patch: 0,
        };
    }

    // Snapshots semanales tipo YYwWWa (ej. 24w14a, 25w02a, 26w10a, 20w14infinite)
    let bytes = raw.as_bytes();
    if bytes.len() >= 5
        && bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[2] == b'w'
        && bytes[3].is_ascii_digit()
        && bytes[4].is_ascii_digit()
    {
        let yy = ((bytes[0] - b'0') as u32) * 10 + (bytes[1] - b'0') as u32;
        let ww = ((bytes[3] - b'0') as u32) * 10 + (bytes[4] - b'0') as u32;
        return match yy {
            26.. => McVersion {
                major: 26,
                minor: 3,
                patch: 0,
            },
            25 => {
                if ww >= 40 {
                    McVersion {
                        major: 26,
                        minor: 0,
                        patch: 0,
                    }
                } else {
                    McVersion {
                        major: 21,
                        minor: 11,
                        patch: 0,
                    }
                }
            }
            24 => {
                if ww >= 33 {
                    McVersion {
                        major: 21,
                        minor: 4,
                        patch: 0,
                    }
                } else {
                    McVersion {
                        major: 21,
                        minor: 1,
                        patch: 0,
                    }
                }
            }
            23 => {
                if ww >= 31 {
                    McVersion {
                        major: 20,
                        minor: 4,
                        patch: 0,
                    }
                } else {
                    McVersion {
                        major: 20,
                        minor: 1,
                        patch: 0,
                    }
                }
            }
            22 => McVersion {
                major: 19,
                minor: 3,
                patch: 0,
            },
            21 => McVersion {
                major: 18,
                minor: 0,
                patch: 0,
            },
            20 => McVersion {
                major: 16,
                minor: 4,
                patch: 0,
            },
            19 => McVersion {
                major: 15,
                minor: 0,
                patch: 0,
            },
            18 => McVersion {
                major: 14,
                minor: 0,
                patch: 0,
            },
            17 => McVersion {
                major: 13,
                minor: 0,
                patch: 0,
            },
            16 => McVersion {
                major: 11,
                minor: 0,
                patch: 0,
            },
            15 => McVersion {
                major: 9,
                minor: 0,
                patch: 0,
            },
            14 => McVersion {
                major: 8,
                minor: 0,
                patch: 0,
            },
            13 => {
                if ww >= 16 {
                    McVersion {
                        major: 6,
                        minor: 0,
                        patch: 0,
                    }
                } else {
                    McVersion {
                        major: 5,
                        minor: 0,
                        patch: 0,
                    }
                }
            }
            _ => McVersion {
                major: 5,
                minor: 0,
                patch: 0,
            },
        };
    }

    // Buscar tokens de versión dentro de IDs compuestos (ej. "fabric-loader-0.16.10-1.21.4", "1.20.1-forge-47.4.10", "1.21.4-pre2", "26.3-rc1")
    for token in raw.split(|c: char| c.is_whitespace() || c == '/' || c == '\\') {
        let parts: Vec<&str> = token.split('.').collect();
        if let Some(first) = parts.first().and_then(|s| parse_leading_uint(s)) {
            if first >= 26 {
                return McVersion {
                    major: first,
                    minor: parts.get(1).and_then(|s| parse_leading_uint(s)).unwrap_or(0),
                    patch: parts.get(2).and_then(|s| parse_leading_uint(s)).unwrap_or(0),
                };
            }
            if first == 1 {
                if let Some(m) = parts.get(1).and_then(|s| parse_leading_uint(s)) {
                    return McVersion {
                        major: m,
                        minor: parts.get(2).and_then(|s| parse_leading_uint(s)).unwrap_or(0),
                        patch: parts.get(3).and_then(|s| parse_leading_uint(s)).unwrap_or(0),
                    };
                }
            }
        }
    }

    // Fallback si el ID tiene prefijos con guiones como "fabric-loader-0.16.10-1.20.1"
    for sub in raw.split('-') {
        let parts: Vec<&str> = sub.split('.').collect();
        if let Some(first) = parts.first().and_then(|s| parse_leading_uint(s)) {
            if first >= 26 && parts.len() >= 2 {
                return McVersion {
                    major: first,
                    minor: parts.get(1).and_then(|s| parse_leading_uint(s)).unwrap_or(0),
                    patch: parts.get(2).and_then(|s| parse_leading_uint(s)).unwrap_or(0),
                };
            }
            if first == 1 && parts.len() >= 2 {
                if let Some(m) = parts.get(1).and_then(|s| parse_leading_uint(s)) {
                    if m >= 1 && m <= 25 {
                        return McVersion {
                            major: m,
                            minor: parts.get(2).and_then(|s| parse_leading_uint(s)).unwrap_or(0),
                            patch: parts.get(3).and_then(|s| parse_leading_uint(s)).unwrap_or(0),
                        };
                    }
                }
            }
        }
    }

    McVersion {
        major: 20,
        minor: 0,
        patch: 0,
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
    } else if ver.major >= 20 {
        // Desde 1.20.0 vanilla usa logo 1024×256 (no el atlas split pre-1.20).
        PackProfile::StandardRange
    } else {
        // 1.16 – 1.19.x
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

/// JSON de `pack.mcmeta` compatible universalmente (releases, betas, pre-releases, RCs y snapshots).
pub fn pack_mcmeta_json(mc: &str) -> String {
    let fmt = pack_format_for_mc(mc);
    let v = parse_mc_version(mc);
    // 1.20+ (fmt >= 15): incluimos tanto supported_formats [1,999] (1.20.2–1.21.8)
    // como min_format/max_format (1.21.9+ / 26.x / snapshots) junto al pack_format exacto.
    if fmt >= 15 || v.major >= 20 {
        return format!(
            r#"{{"pack":{{"pack_format":{fmt},"description":"Marca Oficial Paraguacraft","supported_formats":[1,999],"min_format":[1,0],"max_format":[999,0]}}}}"#
        );
    }
    format!(r#"{{"pack":{{"pack_format":{fmt},"description":"Marca Oficial Paraguacraft"}}}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_key_versions() {
        assert_eq!(pack_format_for_mc("b1.7.3"), 1);
        assert_eq!(pack_format_for_mc("1.8.9"), 1);
        assert_eq!(pack_format_for_mc("1.12.2"), 3);
        assert_eq!(pack_format_for_mc("1.18.2"), 8);
        assert_eq!(pack_format_for_mc("1.20.1"), 15);
        assert_eq!(pack_format_for_mc("1.21.4-pre2"), 46);
        assert_eq!(pack_format_for_mc("24w14a"), 34);
        assert_eq!(pack_format_for_mc("26w10a"), 88);
        assert_eq!(pack_format_for_mc("1.21.11"), 75);
        assert_eq!(pack_format_for_mc("26.2"), 88);
        assert_eq!(pack_format_for_mc("26.3"), 88);
    }

    #[test]
    fn logo_profile_uses_wide_from_1_20() {
        assert_eq!(pack_profile(parse_mc_version("b1.7.3")), PackProfile::Classic);
        assert_eq!(pack_profile(parse_mc_version("1.19.4")), PackProfile::Standard);
        assert_eq!(pack_profile(parse_mc_version("1.20")), PackProfile::StandardRange);
        assert_eq!(pack_profile(parse_mc_version("1.20.1")), PackProfile::StandardRange);
        assert_eq!(pack_profile(parse_mc_version("1.20.4")), PackProfile::StandardRange);
        assert_eq!(pack_profile(parse_mc_version("1.21.1")), PackProfile::StandardRange);
        assert_eq!(pack_profile(parse_mc_version("1.21.4")), PackProfile::Wide);
        assert_eq!(pack_profile(parse_mc_version("1.21.4-rc1")), PackProfile::Wide);
        assert_eq!(pack_profile(parse_mc_version("1.21.11")), PackProfile::Modern);
        assert_eq!(pack_profile(parse_mc_version("26w10a")), PackProfile::Modern);
    }
}
