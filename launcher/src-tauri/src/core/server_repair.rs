//! Diagnóstico y reparación automática de servidores Paper/Spigot locales.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::servers;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerRepairItem {
    /// fixed | warning | error | info
    pub severity: String,
    pub title: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerRepairReport {
    pub items: Vec<ServerRepairItem>,
    pub fixed_count: u32,
    pub warning_count: u32,
}

pub fn repair(server_id: &str) -> AppResult<ServerRepairReport> {
    let st = servers::status(server_id)?;
    if st.running {
        return Err(AppError::msg(
            "Detené el servidor antes de reparar (botón Detener).",
        ));
    }

    let dir = servers::folder_for_id(server_id)?;
    let mut report = ServerRepairReport {
        items: Vec::new(),
        fixed_count: 0,
        warning_count: 0,
    };

    ensure_eula(&dir, &mut report);
    quarantine_broken_jars(&dir.join("plugins"), &mut report)?;
    quarantine_broken_jars(&dir.join("mods"), &mut report)?;
    clean_paper_remapped(&dir, &mut report);
    check_via_pair(&dir.join("plugins"), &mut report);
    analyze_latest_log(&dir.join("logs").join("latest.log"), &mut report);

    if report.fixed_count == 0 && report.warning_count == 0 && report.items.is_empty() {
        report.items.push(item(
            "info",
            "Sin problemas detectados",
            "No se encontraron JARs corruptos ni conflictos habituales. Si sigue fallando, revisá la consola.",
            None,
        ));
    }

    Ok(report)
}

fn ensure_eula(dir: &Path, report: &mut ServerRepairReport) {
    let eula = dir.join("eula.txt");
    let needs_write = if eula.is_file() {
        fs::read_to_string(&eula)
            .map(|c| !c.to_lowercase().contains("eula=true"))
            .unwrap_or(true)
    } else {
        true
    };
    if needs_write {
        if fs::write(&eula, "eula=true\n").is_ok() {
            push_fixed(
                report,
                "EULA aceptada",
                "Se creó/actualizó eula.txt con eula=true.",
                Some(eula),
            );
        }
    }
}

fn quarantine_broken_jars(folder: &Path, report: &mut ServerRepairReport) -> AppResult<()> {
    if !folder.is_dir() {
        return Ok(());
    }
    let quarantine = folder.join(".paraguacraft-broken");
    fs::create_dir_all(&quarantine)?;

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("jar") {
            continue;
        }
        if !is_valid_jar(&path) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "plugin.jar".into());
            let dest = unique_quarantine_path(&quarantine, &name);
            match fs::rename(&path, &dest) {
                Ok(()) => push_fixed(
                    report,
                    format!("JAR corrupto movido: {name}"),
                    "El archivo no es un ZIP/JAR válido (descarga incompleta). Reinstalalo desde Hangar o Modrinth.",
                    Some(dest),
                ),
                Err(e) => push_warning(
                    report,
                    format!("No se pudo mover JAR corrupto: {name}"),
                    format!("{e}. Borralo manualmente de plugins/."),
                    Some(path),
                ),
            }
        }
    }
    Ok(())
}

fn clean_paper_remapped(dir: &Path, report: &mut ServerRepairReport) {
    let remapped = dir.join("plugins").join(".paper-remapped");
    if !remapped.is_dir() {
        return;
    }
    match fs::remove_dir_all(&remapped) {
        Ok(()) => push_fixed(
            report,
            "Caché Paper limpiada",
            "Se eliminó plugins/.paper-remapped (soluciona SkinsRestorer duplicado y remapeos rotos).",
            Some(remapped),
        ),
        Err(e) => push_warning(
            report,
            "No se pudo limpiar .paper-remapped",
            format!("{e}"),
            Some(remapped),
        ),
    }
}

fn check_via_pair(plugins: &Path, report: &mut ServerRepairReport) {
    let Some((via, backwards)) = read_via_versions(plugins) else {
        return;
    };
    let (Some(v), Some(b)) = (via, backwards) else {
        return;
    };
    if v != b {
        push_warning(
            report,
            "ViaVersion y ViaBackwards desincronizados",
            format!(
                "ViaVersion {v} + ViaBackwards {b} suelen fallar juntos. Actualizá ambos a la misma versión (ej. {v})."
            ),
            None,
        );
    }
}

fn read_via_versions(plugins: &Path) -> Option<(Option<String>, Option<String>)> {
    if !plugins.is_dir() {
        return None;
    }
    let mut via = None;
    let mut backwards = None;
    for entry in fs::read_dir(plugins).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(v) = parse_version_prefix(&name, "ViaVersion-") {
            via = Some(v);
        } else if let Some(v) = parse_version_prefix(&name, "ViaBackwards-") {
            backwards = Some(v);
        }
    }
    Some((via, backwards))
}

fn parse_version_prefix(filename: &str, prefix: &str) -> Option<String> {
    if !filename.starts_with(prefix) || !filename.ends_with(".jar") {
        return None;
    }
    let mid = filename.strip_prefix(prefix)?.strip_suffix(".jar")?;
    if mid.chars().all(|c| c.is_ascii_digit() || c == '.') {
        Some(mid.to_string())
    } else {
        None
    }
}

fn analyze_latest_log(path: &Path, report: &mut ServerRepairReport) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };

    if content.contains("UnsupportedClassVersionError") {
        if content.contains("WorldEdit") || content.contains("worldedit") {
            if content.contains("class file version 69") {
                push_warning(
                    report,
                    "WorldEdit requiere Java más nuevo",
                    "WorldEdit 7.4+ necesita Java 25. Instalá Java 25 en Ajustes → Java, o bajá WorldEdit 7.3.x compatible con Java 21.",
                    None,
                );
            } else {
                push_warning(
                    report,
                    "Plugin incompatible con la versión de Java",
                    "Un plugin fue compilado con Java más reciente que el del servidor. Revisá latest.log o actualizá Java.",
                    Some(path.to_path_buf()),
                );
            }
        }
    }

    if content.contains("zip END header not found") && report.fixed_count == 0 {
        push_warning(
            report,
            "Log menciona JARs corruptos",
            "Ejecutá Reparar de nuevo o borrá manualmente los .jar que fallen en plugins/.",
            Some(path.to_path_buf()),
        );
    }

    if content.contains("Ambiguous plugin name") && content.contains("SkinsRestorer") {
        push_info(
            report,
            "SkinsRestorer duplicado (log)",
            "Si persiste tras reparar, dejá un solo SkinsRestorer.jar en plugins/.",
            None,
        );
    }
}

fn is_valid_jar(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if meta.len() < 256 {
        return false;
    }
    let Ok(file) = fs::File::open(path) else {
        return false;
    };
    let Ok(mut archive) = zip::ZipArchive::new(file) else {
        return false;
    };
    if archive.len() == 0 {
        return false;
    }
    {
        let entry = archive.by_index(0);
        entry.is_ok()
    }
}

fn unique_quarantine_path(dir: &Path, name: &str) -> PathBuf {
    let base = dir.join(name);
    if !base.exists() {
        return base;
    }
    let stem = Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "plugin".into());
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    dir.join(format!("{stem}-{ts}.jar"))
}

fn item(severity: &str, title: impl Into<String>, detail: impl Into<String>, path: Option<PathBuf>) -> ServerRepairItem {
    ServerRepairItem {
        severity: severity.into(),
        title: title.into(),
        detail: detail.into(),
        path: path.map(|p| p.to_string_lossy().to_string()),
    }
}

fn push_fixed(report: &mut ServerRepairReport, title: impl Into<String>, detail: impl Into<String>, path: Option<PathBuf>) {
    report.fixed_count += 1;
    report.items.push(item("fixed", title, detail, path));
}

fn push_warning(report: &mut ServerRepairReport, title: impl Into<String>, detail: impl Into<String>, path: Option<PathBuf>) {
    report.warning_count += 1;
    report.items.push(item("warning", title, detail, path));
}

fn push_info(report: &mut ServerRepairReport, title: impl Into<String>, detail: impl Into<String>, path: Option<PathBuf>) {
    report.items.push(item("info", title, detail, path));
}
