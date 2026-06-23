//! Gestor de Java: deteccion, runtime Mojang, Temurin y resolucion por version MC.

pub mod adoptium;
pub mod detect;
pub mod mojang;
pub mod resolve;
pub mod verify;

use crate::models::JavaInstallation;

/// Major de Java requerido por la version de Minecraft (espeja `java_requerido_para_mc`).
pub fn required_for_mc(mc_version: &str) -> u32 {
    let v = mc_version.trim();
    if v.is_empty() {
        return 17;
    }
    let parts: Vec<&str> = v.split('.').collect();
    let major: u32 = parts[0].parse().unwrap_or(1);
    if major >= 23 {
        return 21;
    }
    if major != 1 {
        return 17;
    }
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    if minor <= 16 {
        8
    } else if minor == 17 {
        16
    } else if minor < 20 {
        17
    } else if minor == 20 {
        if patch >= 5 { 21 } else { 17 }
    } else {
        21
    }
}

/// ¿Este major de Java puede ejecutar esta version de MC?
pub fn is_compatible(java_major: u32, mc_version: &str) -> bool {
    let required = required_for_mc(mc_version);
    let parts: Vec<&str> = mc_version.trim().split('.').collect();
    let head: u32 = parts.first().and_then(|p| p.parse().ok()).unwrap_or(1);
    if head != 1 {
        return java_major >= required;
    }
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    if minor <= 16 {
        java_major == 8
    } else if minor == 17 {
        java_major >= 16
    } else {
        java_major >= required
    }
}

/// Elige el mejor Java cuyo major este en la lista (coincidencia exacta).
pub fn pick_best(accepted: &[u32], javas: &[JavaInstallation]) -> Option<JavaInstallation> {
    javas
        .iter()
        .find(|j| accepted.contains(&j.version_major))
        .cloned()
}
