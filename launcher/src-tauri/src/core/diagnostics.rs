//! Diagnostico de crashes: lee `latest.log` + crash-report y explica la causa.

use std::path::Path;

use crate::core::instances;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashDiagnosis {
    pub category: String,
    pub message: String,
    pub hint: String,
    /// Linea concreta del error (Exception, ERROR, etc.).
    pub error_line: Option<String>,
    pub exit_code: i32,
    pub crash_file: Option<String>,
    pub log_tail: String,
    pub suggestions: Vec<String>,
}

fn read_tail(path: &Path, max: usize) -> String {
    let Ok(meta) = std::fs::metadata(path) else {
        return String::new();
    };
    let Ok(mut f) = std::fs::File::open(path) else {
        return String::new();
    };
    use std::io::{Read, Seek, SeekFrom};
    let size = meta.len() as i64;
    if size > max as i64 {
        let _ = f.seek(SeekFrom::End(-(max as i64)));
    }
    let mut buf = Vec::new();
    if f.read_to_end(&mut buf).is_ok() {
        return String::from_utf8_lossy(&buf).to_string();
    }
    String::new()
}

fn latest_crash_report(dir: &Path) -> Option<(String, String)> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut files: Vec<_> = entries
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("txt"))
        .collect();
    files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    let last = files.last()?;
    let name = last.file_name().to_string_lossy().to_string();
    let content = std::fs::read_to_string(last.path()).ok()?;
    Some((name, content))
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

fn extract_error_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let low = line.to_lowercase();
        if line.contains("Exception")
            || line.contains("Error:")
            || line.starts_with("Caused by:")
            || low.contains("[error]")
            || low.contains(" fatal ")
            || low.contains("modloadingexception")
            || low.contains("crash report")
            || low.contains("the game crashed")
            || low.contains("process exited with exit code")
        {
            lines.push(line.to_string());
        }
    }
    lines
}

fn crash_description(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("// Description:") {
            let d = rest.trim();
            if !d.is_empty() {
                return Some(d.to_string());
            }
        }
        if let Some(rest) = t.strip_prefix("Description:") {
            let d = rest.trim();
            if !d.is_empty() {
                return Some(d.to_string());
            }
        }
    }
    None
}

fn pick_best_error_line(log: &str, crash: Option<&str>) -> Option<String> {
    if let Some(cr) = crash {
        if let Some(desc) = crash_description(cr) {
            return Some(desc);
        }
        if let Some(line) = extract_error_lines(cr).into_iter().rev().find(|l| {
            l.contains("Exception") || l.starts_with("Caused by:") || l.contains("Error")
        }) {
            return Some(line);
        }
    }
    extract_error_lines(log)
        .into_iter()
        .rev()
        .find(|l| {
            !l.contains("Process exited") && !l.to_lowercase().contains("trace/warn")
        })
}

struct DiagnosisCore {
    category: &'static str,
    message: String,
    hint: String,
}

fn classify(combined: &str, exit_code: i32, log_empty: bool) -> DiagnosisCore {
    let low = combined.to_lowercase();

    if log_empty && (exit_code == -1 || exit_code == -3 || exit_code == 1) {
        return DiagnosisCore {
            category: "launch_early",
            message: "Minecraft no llegó a iniciar (cierre inmediato).".into(),
            hint: "Verificá Java en Ajustes → Java, reinstalá la versión desde Versiones, y desactivá antivirus que bloquee javaw.".into(),
        };
    }

    macro_rules! hit {
        ($cat:expr, $msg:expr, $hint:expr) => {
            return DiagnosisCore {
                category: $cat,
                message: $msg.into(),
                hint: $hint.into(),
            };
        };
    }

    if low.contains("outofmemoryerror") || low.contains("java heap space") {
        hit!(
            "oom_java",
            "Java se quedó sin memoria (heap).",
            "Subí la RAM en la instancia o en Ajustes → Rendimiento."
        );
    }
    if low.contains("could not reserve enough space") || low.contains("insufficient memory") {
        hit!(
            "oom_reserve",
            "Java no pudo reservar la RAM solicitada.",
            "Bajá la RAM asignada (probá 4096 MB) o cerrá otras apps."
        );
    }
    if ["invalid session", "bad login", "authserver", "authentication"]
        .iter()
        .any(|k| low.contains(k))
        || low.contains("401")
        || low.contains("403")
    {
        hit!(
            "auth",
            "Sesión Microsoft inválida o expirada.",
            "Cerrá sesión y volvé a iniciarla en Ajustes → Cuentas."
        );
    }
    if (low.contains("mixin") && (low.contains("apply") || low.contains("injection")))
        || low.contains("modloadingexception")
        || low.contains("failed to load mod")
        || low.contains("missing or unsupported mandatory mods")
    {
        hit!(
            "mods",
            "Un mod o Mixin falló al cargar.",
            "Revisá mods incompatibles con la versión/loader. Probá quitar el último mod agregado."
        );
    }
    if low.contains("glfw")
        || low.contains("pixel format")
        || (low.contains("opengl") && low.contains("version"))
        || low.contains("lwjgl")
    {
        hit!(
            "gpu",
            "Falló la inicialización gráfica (OpenGL/GLFW/LWJGL).",
            "Actualizá drivers GPU o probá modo compatibilidad en Ajustes."
        );
    }
    if low.contains("unsupportedclassversionerror")
        || low.contains("has been compiled by a more recent")
        || low.contains("class file version")
    {
        hit!(
            "java_version",
            "La versión de Java no es compatible con este Minecraft.",
            "Instalá el Java correcto (8 para 1.8, 17 para 1.18–1.20, 21 para 1.20.5+) en Ajustes → Java."
        );
    }
    if low.contains("could not find or load main class")
        || low.contains("noclassdeffounderror")
        || low.contains("classnotfoundexception")
    {
        hit!(
            "install",
            "Faltan librerías o la instalación está incompleta/corrupta.",
            "En Versiones, reinstalá la versión. Si persiste, borrá `.minecraft/versions/<versión>` y volvé a instalar."
        );
    }
    if low.contains("unknownhostexception")
        || low.contains("connection refused")
        || low.contains("no route to host")
        || low.contains("failed to download")
        || low.contains("sha-1")
    {
        hit!(
            "network",
            "Problema de red o descarga de assets/librerías.",
            "Revisá tu conexión, VPN o firewall. Reinstalá la versión para re-descargar archivos corruptos."
        );
    }
    if low.contains("access is denied") || low.contains("permission denied") {
        hit!(
            "permissions",
            "Windows bloqueó un archivo (permisos o antivirus).",
            "Ejecutá el launcher como administrador una vez, o excluí `.minecraft` del antivirus."
        );
    }
    if exit_code == -3 || exit_code == -1 {
        return DiagnosisCore {
            category: "launch_abort",
            message: format!("El proceso de Java terminó abruptamente (código {exit_code})."),
            hint: "Abrí logs/latest.log en la carpeta de la instancia. Suele ser Java incorrecto, RAM muy alta, o un mod que crashea al arrancar.".into(),
        };
    }

    DiagnosisCore {
        category: "generic",
        message: format!("Minecraft terminó con código {exit_code}."),
        hint: "Revisá el detalle del error abajo.".into(),
    }
}

fn enrich_generic(core: &mut DiagnosisCore, error_line: &Option<String>) {
    if core.category != "generic" {
        return;
    }
    if let Some(line) = error_line {
        core.message = format!("Error detectado: {}", truncate(line, 180));
        if line.to_lowercase().contains("exception") {
            core.hint =
                "Buscá esa excepción en el log completo. Si es un mod, quitá mods recientes.".into();
        }
    }
}

/// Analiza el ultimo log/crash de una instancia.
pub fn analyze_instance(instance_id: &str, exit_code: i32) -> AppResult<CrashDiagnosis> {
    let dir = instances::game_dir_for(instance_id)
        .ok_or_else(|| AppError::msg("Instancia no encontrada"))?;
    Ok(analyze_paths(
        &dir.join("logs").join("latest.log"),
        &dir.join("crash-reports"),
        exit_code,
    ))
}

pub fn analyze_paths(log_path: &Path, crash_dir: &Path, exit_code: i32) -> CrashDiagnosis {
    let log_tail = read_tail(log_path, 48_000);
    let log_empty = log_tail.trim().len() < 40;

    let crash = if crash_dir.is_dir() {
        latest_crash_report(crash_dir)
    } else {
        None
    };

    let crash_content = crash.as_ref().map(|(_, c)| c.as_str());
    let combined = format!(
        "{}\n{}",
        log_tail,
        crash_content.unwrap_or_default()
    );

    let error_line = pick_best_error_line(&log_tail, crash_content);
    let mut core = classify(&combined, exit_code, log_empty);
    enrich_generic(&mut core, &error_line);

    let mut suggestions = crate::core::ai::suggestions_for_category(core.category);
    if core.category == "generic" {
        if let Some(line) = &error_line {
            suggestions.insert(0, format!("Error clave: {}", truncate(line, 120)));
        }
        suggestions.push("Abrí la carpeta de la instancia → logs/latest.log.".into());
    }
    if crash.is_some() {
        suggestions.insert(
            0,
            "Hay un crash-report en crash-reports/ con el stack trace completo.".into(),
        );
    }

    let tail_display: String = log_tail
        .chars()
        .rev()
        .take(2500)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    CrashDiagnosis {
        category: core.category.into(),
        message: core.message,
        hint: core.hint,
        error_line,
        exit_code,
        crash_file: crash.map(|(n, _)| n),
        log_tail: tail_display,
        suggestions,
    }
}
