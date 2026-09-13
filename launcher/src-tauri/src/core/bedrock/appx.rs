//! Extraer AppX y registrar el paquete UWP suelto (requiere Modo desarrollador).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, AppResult};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const PACKAGE_NAME: &str = "Microsoft.MinecraftUWP";
const DEV_MODE_KEY: &str = r"HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock";

pub struct RegisteredPackage {
    pub version: String,
    pub install_location: String,
}

fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub fn run_powershell(script: &str) -> AppResult<String> {
    use std::os::windows::process::CommandExt;

    let out = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| AppError::msg(format!("No se pudo ejecutar PowerShell: {e}")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(AppError::msg(if detail.is_empty() {
            format!("PowerShell salió con código {}", out.status)
        } else {
            detail
        }));
    }
    Ok(stdout)
}

fn extract_zip_file(archive: &Path, dest: &Path) -> AppResult<usize> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let total = zip.len();
    for i in 0..total {
        let mut entry = zip.by_index(i)?;
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let out_path = dest.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = std::fs::File::create(&out_path)?;
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = entry.read(&mut buf)?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])?;
        }
    }
    Ok(total)
}

fn find_nested_packages(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        if name.ends_with(".appx") || name.ends_with(".msix") {
            found.push(path);
        }
    }
    found.sort_by_key(|p| {
        let n = p.file_name().map(|s| s.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        if n.contains("x64") {
            0
        } else if n.contains("neutral") {
            1
        } else {
            2
        }
    });
    found
}

/// Extrae un `.appx` / bundle y quita la firma para registro suelto.
pub fn extract_appx(appx_path: &Path, dest: &Path) -> AppResult<()> {
    if dest.exists() {
        let _ = std::fs::remove_dir_all(dest);
    }
    std::fs::create_dir_all(dest)?;
    extract_zip_file(appx_path, dest)?;

    if !dest.join("AppxManifest.xml").is_file() {
        let nested = find_nested_packages(dest);
        let Some(inner) = nested.into_iter().next() else {
            return Err(AppError::msg(
                "El paquete descargado no trae AppxManifest.xml. No es un AppX de Minecraft usable.",
            ));
        };
        let tmp = dest.join(".pg-nested-appx");
        std::fs::create_dir_all(&tmp)?;
        extract_zip_file(&inner, &tmp)?;
        hoist_dir(&tmp, dest)?;
        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::remove_file(&inner);
        for leftover in find_nested_packages(dest) {
            let _ = std::fs::remove_file(leftover);
        }
    }

    let _ = std::fs::remove_file(dest.join("AppxSignature.p7x"));
    if !dest.join("AppxManifest.xml").is_file() {
        return Err(AppError::msg(
            "Tras extraer el AppX no aparece AppxManifest.xml.",
        ));
    }
    Ok(())
}

fn hoist_dir(src: &Path, dest: &Path) -> AppResult<()> {
    let Ok(entries) = std::fs::read_dir(src) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if to.exists() {
            if to.is_dir() {
                let _ = std::fs::remove_dir_all(&to);
            } else {
                let _ = std::fs::remove_file(&to);
            }
        }
        if std::fs::rename(&from, &to).is_ok() {
            continue;
        }
        if from.is_dir() {
            copy_dir(&from, &to)?;
            let _ = std::fs::remove_dir_all(&from);
        } else {
            std::fs::copy(&from, &to)?;
            let _ = std::fs::remove_file(&from);
        }
    }
    Ok(())
}

pub fn copy_dir(src: &Path, dest: &Path) -> AppResult<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let to = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else {
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

pub fn query_registered() -> Option<RegisteredPackage> {
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue';\
         $p = Get-AppxPackage -Name {PACKAGE_NAME} | Select-Object -First 1;\
         if (-not $p) {{ $p = Get-AppxPackage -Name Microsoft.MinecraftWindowsBeta | Select-Object -First 1 }};\
         if (-not $p) {{ $p = Get-AppxPackage -Name '*Minecraft*' | Where-Object {{ $_.Name -notmatch 'Education|Dungeons|Launcher' }} | Select-Object -First 1 }};\
         if ($p) {{ Write-Output ('{{0}}|{{1}}|{{2}}' -f $p.Version, $p.InstallLocation, $p.PackageFullName) }}"
    );
    let out = run_powershell(&script).ok()?;
    if out.is_empty() {
        return None;
    }
    let mut parts = out.splitn(3, '|');
    let version = parts.next().unwrap_or("").trim().to_string();
    let install_location = parts.next().unwrap_or("").trim().to_string();
    let _package_full_name = parts.next().unwrap_or("").trim().to_string();
    if install_location.is_empty() && version.is_empty() {
        return None;
    }
    Some(RegisteredPackage {
        version,
        install_location,
    })
}

fn norm_path(p: &str) -> String {
    p.trim_end_matches(['\\', '/'])
        .replace('/', "\\")
        .to_ascii_lowercase()
}

pub fn is_store_location(install_location: &str) -> bool {
    let n = norm_path(install_location);
    n.contains("\\windowsapps\\") || n.contains("\\xboxgames\\")
}

pub fn register_package(game_dir: &Path) -> AppResult<()> {
    let manifest = game_dir.join("AppxManifest.xml");
    if !manifest.is_file() {
        return Err(AppError::msg("Falta AppxManifest.xml en la versión extraída."));
    }
    let dir = game_dir.to_string_lossy().to_string();
    let manifest_s = manifest.to_string_lossy().to_string();
    let script = format!(
        "$ErrorActionPreference='Stop';\
         try {{\
           $pkgs = Get-AppxPackage -Name '*Minecraft*' | Where-Object {{ $_.Name -notmatch 'Education|Dungeons|Launcher' }};\
           foreach ($p in $pkgs) {{\
             if ($p.InstallLocation -ne {loc}) {{\
               try {{ Remove-AppxPackage -Package $p.PackageFullName -PreserveApplicationData }}\
               catch {{ Remove-AppxPackage -Package $p.PackageFullName }}\
             }}\
           }}\
           Add-AppxPackage -Register {man} -ForceApplicationShutdown;\
           Write-Output 'PG_REGISTER_OK';\
         }} catch {{\
           Write-Output ('PG_REGISTER_ERR:' + $_.Exception.Message);\
         }}",
        loc = ps_quote(&dir),
        man = ps_quote(&manifest_s),
    );
    let out = run_powershell(&script)?;
    if out.contains("PG_REGISTER_OK") {
        return Ok(());
    }
    let detail = out.replace("PG_REGISTER_ERR:", "");
    let lower = detail.to_ascii_lowercase();
    if lower.contains("0x80073cff") || lower.contains("developer") {
        return Err(AppError::msg(
            "Hace falta el Modo desarrollador de Windows para registrar esta versión de Bedrock.",
        ));
    }
    Err(AppError::msg(if detail.is_empty() {
        "No se pudo registrar el paquete Bedrock.".into()
    } else {
        detail
    }))
}

pub fn unregister_package(game_dir: &Path) -> AppResult<()> {
    let dir = game_dir.to_string_lossy().to_string();
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue';\
         $pkgs = Get-AppxPackage -Name {PACKAGE_NAME};\
         foreach ($p in $pkgs) {{\
           if ($p.InstallLocation -eq {loc}) {{\
             Remove-AppxPackage -Package $p.PackageFullName;\
           }}\
         }}",
        loc = ps_quote(&dir),
    );
    let _ = run_powershell(&script);
    Ok(())
}

pub fn is_developer_mode_enabled() -> bool {
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue';\
         $v = (Get-ItemProperty -Path {DEV_MODE_KEY} -Name AllowDevelopmentWithoutDevLicense).AllowDevelopmentWithoutDevLicense;\
         Write-Output $v"
    );
    run_powershell(&script)
        .ok()
        .map(|s| s.trim() == "1")
        .unwrap_or(false)
}

pub fn enable_developer_mode() -> AppResult<()> {
    use base64::Engine;
    use std::os::windows::process::CommandExt;

    let inner = format!(
        "New-Item -Path '{DEV_MODE_KEY}' -Force | Out-Null; \
         Set-ItemProperty -Path '{DEV_MODE_KEY}' -Name AllowDevelopmentWithoutDevLicense -Value 1 -Type DWord; \
         Set-ItemProperty -Path '{DEV_MODE_KEY}' -Name AllowAllTrustedApps -Value 1 -Type DWord"
    );
    let utf16: Vec<u8> = inner.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    let encoded = base64::engine::general_purpose::STANDARD.encode(utf16);
    let outer = format!(
        "Start-Process powershell.exe -Verb RunAs -Wait -WindowStyle Hidden -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-EncodedCommand','{encoded}'"
    );
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &outer,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| AppError::msg(format!("No se pudo pedir elevación UAC: {e}")))?;
    if !status.success() {
        return Err(AppError::msg(
            "No se pudo activar el Modo desarrollador. Activalo a mano en Ajustes → Para desarrolladores.",
        ));
    }
    if !is_developer_mode_enabled() {
        return Err(AppError::msg(
            "El Modo desarrollador sigue desactivado. Activalo en Ajustes → Para desarrolladores.",
        ));
    }
    Ok(())
}

pub fn open_store_product() -> AppResult<()> {
    use std::os::windows::process::CommandExt;
    Command::new("explorer.exe")
        .arg("ms-windows-store://pdp/?productid=9NBLGGH2JHXJ")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| AppError::msg(format!("No se pudo abrir Microsoft Store: {e}")))?;
    Ok(())
}
