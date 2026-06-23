//! Verificacion de una ruta de Java (espeja `verificar_java` del Python).

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::models::JavaInstallation;

/// Ejecuta `java -version` (escribe en stderr) y parsea el major.
pub fn verify(path: &Path, source: &str) -> Option<JavaInstallation> {
    if !path.is_file() {
        return None;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let p = path.to_path_buf();
    let src = source.to_string();
    std::thread::spawn(move || {
        let _ = tx.send(verify_inner(&p, &src));
    });
    rx.recv_timeout(Duration::from_secs(8)).ok().flatten()
}

fn verify_inner(path: &Path, source: &str) -> Option<JavaInstallation> {
    let mut cmd = Command::new(path);
    cmd.arg("-version");
    hide_console(&mut cmd);

    let output = cmd.output().ok()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );

    let ver_str = parse_version_string(&combined)?;
    let major = parse_major(&ver_str)?;
    let vendor = parse_vendor(&combined);

    Some(JavaInstallation {
        path: path.to_string_lossy().to_string(),
        version_major: major,
        version_full: ver_str,
        vendor,
        source: source.to_string(),
    })
}

/// Extrae la cadena de version: `version "1.8.0_281"` -> `1.8.0_281`.
fn parse_version_string(out: &str) -> Option<String> {
    let idx = out.find("version")?;
    let rest = &out[idx + 7..];
    let start = rest.find('"')? + 1;
    let end = rest[start..].find('"')? + start;
    Some(rest[start..end].to_string())
}

/// `1.8.0_281` -> 8 ; `17.0.5` -> 17 ; `21` -> 21.
fn parse_major(ver: &str) -> Option<u32> {
    if let Some(stripped) = ver.strip_prefix("1.") {
        stripped.split('.').next()?.parse().ok()
    } else {
        ver.split(['.', '-', '+']).next()?.parse().ok()
    }
}

fn parse_vendor(out: &str) -> String {
    const KEYS: [&str; 8] = [
        "openjdk", "temurin", "adoptium", "graalvm", "zulu", "corretto", "liberica", "hotspot",
    ];
    for line in out.lines() {
        let low = line.to_lowercase();
        if KEYS.iter().any(|k| low.contains(k)) {
            return line.trim().to_string();
        }
    }
    String::new()
}

#[cfg(target_os = "windows")]
fn hide_console(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_console(_cmd: &mut Command) {}
