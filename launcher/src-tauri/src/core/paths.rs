//! Resolucion de rutas multiplataforma.
//!
//! - Datos del launcher: `%APPDATA%/ParaguacraftLauncher` (Win),
//!   `~/Library/Application Support/ParaguacraftLauncher` (mac),
//!   `~/.config/ParaguacraftLauncher` (Linux).  Espeja `Api._DATA_DIR`.
//! - `.minecraft` por defecto y carpeta de instancias compartida
//!   (`<mc>/instancias`) para mantener compatibilidad con el launcher Python.
//! - Carpetas candidatas de otros launchers para el escaneo/import.

use std::path::PathBuf;

const APP_DIR_NAME: &str = "ParaguacraftLauncher";

/// Carpeta de datos del launcher (se crea si no existe).
pub fn data_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default());
    let dir = base.join(APP_DIR_NAME);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn config_file() -> PathBuf {
    data_dir().join("launcher_config.json")
}

pub fn accounts_file() -> PathBuf {
    data_dir().join("accounts.json")
}

/// Tokens sensibles (refresh tokens). Archivo separado y con permisos
/// restringidos en Unix (ver `config::secrets`).
pub fn secrets_file() -> PathBuf {
    data_dir().join("secrets.json")
}

pub fn java_dir() -> PathBuf {
    let dir = data_dir().join("java");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// `.minecraft` por defecto segun el SO.
pub fn default_minecraft_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::config_dir().unwrap_or_default());
        appdata.join(".minecraft")
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap_or_default()
            .join("Library/Application Support/minecraft")
    } else {
        dirs::home_dir().unwrap_or_default().join(".minecraft")
    }
}

/// Carpeta de instancias del ecosistema Paraguacraft (compartida con el Python).
pub fn instances_dir() -> PathBuf {
    default_minecraft_dir().join("instancias")
}

/// Biblioteca de skins descargadas (`~/Paraguacraft_Skins`, igual que el launcher Python).
pub fn skins_library_dir() -> PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| data_dir())
        .join("Paraguacraft_Skins");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Carpetas candidatas de otros launchers, para escaneo (solo lectura).
/// Cada entrada es (origen, ruta_raiz_de_instancias). No falla si no existen.
pub fn external_launcher_roots() -> Vec<(&'static str, PathBuf)> {
    let home = dirs::home_dir().unwrap_or_default();
    let config = dirs::config_dir().unwrap_or_else(|| home.clone());
    let data = dirs::data_dir().unwrap_or_else(|| config.clone());
    let mut roots: Vec<(&'static str, PathBuf)> = Vec::new();

    // Vanilla / launcher oficial.
    roots.push(("vanilla", default_minecraft_dir().join("versions")));

    // Lunar Client.
    roots.push(("lunar", home.join(".lunarclient").join("offline").join("multiver")));

    // Prism Launcher.
    roots.push(("prism", data.join("PrismLauncher").join("instances")));
    if cfg!(target_os = "windows") {
        if let Ok(appdata) = std::env::var("APPDATA") {
            roots.push(("prism", PathBuf::from(appdata).join("PrismLauncher").join("instances")));
        }
    }

    // PolyMC / MultiMC (compatibles con Prism).
    roots.push(("prism", data.join("PolyMC").join("instances")));

    // TLauncher (usa .minecraft con versiones propias).
    roots.push(("tlauncher", default_minecraft_dir().join("versions")));

    // SKLauncher (tambien sobre .minecraft).
    roots.push(("sklauncher", default_minecraft_dir().join("versions")));

    roots.into_iter().filter(|(_, p)| p.exists()).collect()
}
