//! Deteccion de instalaciones de Java (espeja `detectar_javas` del Python).
//!
//! Recoge rutas candidatas del sistema, las verifica y deduplica por JDK
//! (no por binario: evita duplicar java.exe + javaw.exe del mismo JRE).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::paths;
use crate::models::JavaInstallation;

use super::verify::verify;

const JAVA_BINS_WIN: [&str; 2] = ["javaw.exe", "java.exe"];
const JAVA_BIN_UNIX: &str = "java";

/// Lista todos los Javas detectables y verificados, ordenados por preferencia.
pub fn detect_all() -> Vec<JavaInstallation> {
    let candidates = if cfg!(target_os = "windows") {
        candidates_windows()
    } else {
        candidates_unix()
    };

    let mut by_jdk: HashMap<String, JavaInstallation> = HashMap::new();

    for (path, source) in candidates {
        let Some(info) = verify(&path, &source) else {
            continue;
        };
        let root = jdk_root(&path);
        by_jdk
            .entry(root)
            .and_modify(|existing| {
                if prefer_installation(&info, existing) {
                    *existing = info.clone();
                }
            })
            .or_insert(info);
    }

    let mut out: Vec<JavaInstallation> = by_jdk.into_values().collect();
    out.sort_by(|a, b| {
        rank(&a.source)
            .cmp(&rank(&b.source))
            .then(b.version_major.cmp(&a.version_major))
    });
    out
}

fn jdk_root(path: &Path) -> String {
    path.parent()
        .and_then(|bin| bin.parent())
        .map(|home| home.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| path.to_string_lossy().to_lowercase())
}

fn prefer_installation(candidate: &JavaInstallation, current: &JavaInstallation) -> bool {
    if cfg!(target_os = "windows") {
        let c_javaw = candidate.path.to_ascii_lowercase().contains("javaw.exe");
        let cur_javaw = current.path.to_ascii_lowercase().contains("javaw.exe");
        if c_javaw && !cur_javaw {
            return true;
        }
        if cur_javaw && !c_javaw {
            return false;
        }
    }
    rank(&candidate.source) < rank(&current.source)
        || (rank(&candidate.source) == rank(&current.source)
            && candidate.version_major > current.version_major)
}

fn rank(source: &str) -> u8 {
    match source.split(':').next().unwrap_or(source) {
        "paraguacraft" => 0,
        "mojang" => 1,
        "java_home" => 2,
        "path" => 3,
        _ => 4,
    }
}

fn push_bins(dir: &Path, source: &str, out: &mut Vec<(PathBuf, String)>) {
    if cfg!(target_os = "windows") {
        for b in JAVA_BINS_WIN {
            let p = dir.join(b);
            if p.is_file() {
                out.push((p, source.to_string()));
            }
        }
    } else {
        let p = dir.join(JAVA_BIN_UNIX);
        if p.is_file() {
            out.push((p, source.to_string()));
        }
    }
}

fn candidates_windows() -> Vec<(PathBuf, String)> {
    let mut out: Vec<(PathBuf, String)> = Vec::new();

    let brands = [
        "Java",
        "Eclipse Adoptium",
        "Eclipse Foundation",
        "AdoptOpenJDK",
        "Microsoft",
        "Zulu",
        "Amazon Corretto",
        "BellSoft",
        "GraalVM",
        "Semeru",
    ];
    for var in ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"] {
        let Ok(pf) = std::env::var(var) else { continue };
        for brand in brands {
            let root = Path::new(&pf).join(brand);
            if let Ok(entries) = std::fs::read_dir(&root) {
                for e in entries.flatten() {
                    push_bins(&e.path().join("bin"), &format!("system:{brand}"), &mut out);
                }
            }
        }
    }

    let mc_runtime = paths::default_minecraft_dir().join("runtime");
    collect_recursive(&mc_runtime, "mojang", 6, &mut out);

    if let Ok(jh) = std::env::var("JAVA_HOME") {
        push_bins(&Path::new(&jh).join("bin"), "java_home", &mut out);
    }

    collect_recursive(&paths::java_dir(), "paraguacraft", 5, &mut out);

    if let Some(p) = which_java() {
        out.push((p, "path".into()));
    }
    out
}

fn candidates_unix() -> Vec<(PathBuf, String)> {
    let mut out: Vec<(PathBuf, String)> = Vec::new();
    for base in [
        "/usr/lib/jvm",
        "/usr/java",
        "/opt",
        "/Library/Java/JavaVirtualMachines",
    ] {
        let base = Path::new(base);
        if let Ok(entries) = std::fs::read_dir(base) {
            for e in entries.flatten() {
                let p = e.path();
                push_bins(&p.join("bin"), "system", &mut out);
                push_bins(&p.join("Contents/Home/bin"), "system", &mut out);
                push_bins(&p.join("jre/bin"), "system", &mut out);
            }
        }
    }
    if let Ok(jh) = std::env::var("JAVA_HOME") {
        push_bins(&Path::new(&jh).join("bin"), "java_home", &mut out);
    }
    collect_recursive(&paths::java_dir(), "paraguacraft", 5, &mut out);
    if let Some(p) = which_java() {
        out.push((p, "path".into()));
    }
    out
}

fn collect_recursive(root: &Path, source: &str, max_depth: usize, out: &mut Vec<(PathBuf, String)>) {
    fn walk(dir: &Path, source: &str, depth: usize, max: usize, out: &mut Vec<(PathBuf, String)>) {
        if depth > max {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().and_then(|n| n.to_str()) == Some("bin") {
                    push_bins(&p, source, out);
                }
                walk(&p, source, depth + 1, max, out);
            }
        }
    }
    if root.is_dir() {
        walk(root, source, 0, max_depth, out);
    }
}

fn which_java() -> Option<PathBuf> {
    let bin = if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    };
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let p = dir.join(bin);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
