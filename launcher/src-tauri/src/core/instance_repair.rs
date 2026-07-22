//! Diagnóstico y reparación de instancias locales (loader, libs, mods corruptos).

use std::fs;
use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::core::instances::{self, profiles};
use crate::core::launch;
use crate::core::loaders;
use crate::core::server_repair::{ServerRepairItem, ServerRepairReport};
use crate::core::versions;
use crate::error::{AppError, AppResult};

pub async fn repair(
    app: &AppHandle,
    client: &reqwest::Client,
    instance_id: &str,
) -> AppResult<ServerRepairReport> {
    let meta = instances::read_meta(instance_id)
        .or_else(|| instances::resolve_meta(instance_id))
        .ok_or_else(|| AppError::msg("Instancia no encontrada"))?;

    let dir = instances::instance_dir(instance_id);
    let mut report = ServerRepairReport {
        items: Vec::new(),
        fixed_count: 0,
        warning_count: 0,
    };

    quarantine_broken_jars(&dir.join("mods"), &mut report)?;
    quarantine_broken_jars(&dir.join("shaderpacks"), &mut report)?;

    push_info(
        &mut report,
        "Reinstalando Minecraft base",
        &format!("Descargando/verificando Minecraft {}…", meta.mc_version),
        None,
    );

    if let Err(e) = versions::install_vanilla(app, client, &meta.mc_version).await {
        push_error(
            &mut report,
            "Fallo al reinstalar Minecraft",
            e.to_string(),
            None,
        );
        return Ok(report);
    }
    push_fixed(
        &mut report,
        "Minecraft base verificado",
        "Librerías y cliente de la versión están presentes.",
        None,
    );

    let loader = loaders::normalize(&meta.loader);
    match loaders::install_loader(
        app,
        client,
        &meta.mc_version,
        &loader,
        &meta.loader_version,
    )
    .await
    {
        Ok(version_id) => {
            let _ = profiles::set_version_id(instance_id, &version_id);
            push_fixed(
                &mut report,
                "Loader reinstalado",
                format!("Perfil activo: {version_id}"),
                None,
            );
            if let Ok(merged) = launch::load_merged(&version_id) {
                if let Err(e) = versions::ensure_merged_libraries(
                    app,
                    client,
                    &merged,
                    &format!("Dependencias {}", meta.mc_version),
                )
                .await
                {
                    push_warning(
                        &mut report,
                        "Algunas librerías no se pudieron verificar",
                        e.to_string(),
                        None,
                    );
                } else {
                    let base = launch::resolve_version_chain_base_id(&version_id);
                    let natives = versions::versions_dir().join(&base).join("natives");
                    let _ = fs::remove_dir_all(&natives);
                    push_fixed(
                        &mut report,
                        "Natives LWJGL",
                        "Se volverán a extraer al iniciar el juego.",
                        Some(natives),
                    );
                }
            }
        }
        Err(e) => {
            push_error(
                &mut report,
                "Fallo al reinstalar loader",
                e.to_string(),
                None,
            );
            return Ok(report);
        }
    }

    if loader == "paraguacraft-pvp" {
        match loaders::pvp::install_bundle(app, client, &dir).await {
            Ok(()) => push_fixed(
                &mut report,
                "Bundle PvP sincronizado",
                "ParaguacraftPvP + OptiFine descargados desde GitHub.",
                Some(dir.join("mods")),
            ),
            Err(e) => push_warning(
                &mut report,
                "Bundle PvP incompleto",
                e.to_string(),
                Some(dir.join("mods")),
            ),
        }
    }

    if loader == "fabric-iris" {
        match loaders::fabric_iris::install_bundle(app, client, &meta.mc_version, &dir).await {
            Ok(()) => push_fixed(
                &mut report,
                "Fabric + Iris verificados",
                "Mods de rendimiento sincronizados.",
                Some(dir.join("mods")),
            ),
            Err(e) => push_warning(
                &mut report,
                "Fabric/Iris incompleto",
                e.to_string(),
                Some(dir.join("mods")),
            ),
        }
    }

    if loader == "paraguacraft-pvp-modern" {
        match crate::core::modern_pvp::sync_instance_bundles(app, client, instance_id).await {
            Ok(()) => push_fixed(
                &mut report,
                "Cliente PvP 1.21.11 sincronizado",
                "Stack pinneado completo (Iris, Sodium, HUD y mod Paraguacraft).",
                Some(dir.join("mods")),
            ),
            Err(e) => push_warning(
                &mut report,
                "Cliente PvP 1.21.11 incompleto",
                e.to_string(),
                Some(dir.join("mods")),
            ),
        }
    }

    let log_path = dir.join("logs").join("latest.log");
    if log_path.is_file() {
        if let Ok(content) = fs::read_to_string(&log_path) {
            if content.contains("zip END header not found") {
                push_warning(
                    &mut report,
                    "Log menciona JARs corruptos",
                    "Si el juego sigue fallando, revisá mods/ y reinstalá los mods afectados.",
                    Some(log_path),
                );
            }
        }
    }

    if report.fixed_count == 0 && report.warning_count == 0 && report.items.len() <= 2 {
        push_info(
            &mut report,
            "Instancia reparada",
            "Loader y archivos base verificados. Probá jugar de nuevo.",
            None,
        );
    }

    Ok(report)
}

pub fn read_log_lines(instance_id: &str, max_lines: usize) -> AppResult<Vec<String>> {
    let dir = instances::instance_dir(instance_id);
    let log_path = dir.join("logs").join("latest.log");
    if !log_path.is_file() {
        return Ok(vec!["(Sin latest.log — jugá al menos una vez para generar logs.)".into()]);
    }
    let content = fs::read_to_string(&log_path)?;
    let lines: Vec<String> = content.lines().map(String::from).collect();
    let start = lines.len().saturating_sub(max_lines);
    Ok(lines[start..].to_vec())
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
                .unwrap_or_else(|| "mod.jar".into());
            let dest = unique_quarantine_path(&quarantine, &name);
            match fs::rename(&path, &dest) {
                Ok(()) => push_fixed(
                    report,
                    format!("JAR corrupto movido: {name}"),
                    "Descarga incompleta o archivo dañado. Reinstalalo desde la tienda.",
                    Some(dest),
                ),
                Err(e) => push_warning(
                    report,
                    format!("No se pudo mover JAR corrupto: {name}"),
                    format!("{e}"),
                    Some(path),
                ),
            }
        }
    }
    Ok(())
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
    archive.len() > 0 && archive.by_index(0).is_ok()
}

fn unique_quarantine_path(dir: &Path, name: &str) -> PathBuf {
    let base = dir.join(name);
    if !base.exists() {
        return base;
    }
    let stem = Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "mod".into());
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    dir.join(format!("{stem}-{ts}.jar"))
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

fn push_error(report: &mut ServerRepairReport, title: impl Into<String>, detail: impl Into<String>, path: Option<PathBuf>) {
    report.warning_count += 1;
    report.items.push(item("error", title, detail, path));
}

fn item(severity: &str, title: impl Into<String>, detail: impl Into<String>, path: Option<PathBuf>) -> ServerRepairItem {
    ServerRepairItem {
        severity: severity.into(),
        title: title.into(),
        detail: detail.into(),
        path: path.map(|p| p.to_string_lossy().to_string()),
    }
}
