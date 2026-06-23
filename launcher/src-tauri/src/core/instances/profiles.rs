//! Perfiles / CRUD de instancias del ecosistema Paraguacraft.
//!
//! Espeja `crear_instancia_personalizada`, `actualizar_instancia_nombre`,
//! `duplicar_instancia`, `eliminar_instancia` del Python. Solo opera sobre
//! instancias locales (las externas son de solo lectura hasta importarse).

use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::models::Instance;

use super::{folder_for, instance_dir, read_meta, write_meta, InstanceMeta};

fn ensure_local(id: &str) -> AppResult<()> {
    if id.starts_with("ext::") {
        return Err(AppError::msg(
            "Esta instancia es de otro launcher (solo lectura). Importala primero.",
        ));
    }
    if !instance_dir(id).is_dir() {
        return Err(AppError::msg("Instancia no encontrada"));
    }
    Ok(())
}

pub fn create(
    name: &str,
    mc_version: &str,
    loader: &str,
    loader_version: &str,
    icon: &str,
    ram_mb: u32,
) -> AppResult<Instance> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::msg("El nombre no puede estar vacio"));
    }
    // Carpeta unica.
    let mut folder = folder_for(mc_version, loader);
    let mut n = 2;
    while instance_dir(&folder).exists() {
        folder = format!("{}_{n}", folder_for(mc_version, loader));
        n += 1;
    }
    let dir = instance_dir(&folder);
    std::fs::create_dir_all(dir.join("mods"))?;

    let meta = InstanceMeta {
        name: name.to_string(),
        icon: if icon.trim().starts_with("custom:") {
            icon.to_string()
        } else {
            super::icon_for_loader(loader)
        },
        mc_version: mc_version.to_string(),
        loader: loader.to_string(),
        loader_version: loader_version.to_string(),
        source: "paraguacraft".into(),
        ram_mb: if ram_mb == 0 { 4096 } else { ram_mb },
        total_play_minutes: 0,
        last_played: None,
        version_id: None,
        auto_managed: true,
        jvm_args: None,
        gc: None,
        java_path: None,
    };
    write_meta(&folder, &meta)?;
    Ok(meta.into_instance(&folder, &dir))
}

pub fn rename(id: &str, name: &str, icon: &str) -> AppResult<Instance> {
    ensure_local(id)?;
    let mut meta = read_meta(id)
        .or_else(|| super::resolve_meta(id))
        .ok_or_else(|| AppError::msg("Sin metadata"))?;
    if !name.trim().is_empty() {
        meta.name = name.trim().to_string();
    }
    if !icon.is_empty() {
        meta.icon = icon.to_string();
    }
    write_meta(id, &meta)?;
    Ok(meta.into_instance(id, &instance_dir(id)))
}

pub fn set_ram(id: &str, ram_mb: u32) -> AppResult<Instance> {
    ensure_local(id)?;
    let mut meta = read_meta(id)
        .or_else(|| super::resolve_meta(id))
        .ok_or_else(|| AppError::msg("Sin metadata"))?;
    meta.ram_mb = ram_mb;
    // Edicion manual => el launcher ya no autogestiona (Regla 2).
    meta.auto_managed = false;
    write_meta(id, &meta)?;
    Ok(meta.into_instance(id, &instance_dir(id)))
}

/// Override completo de la config JVM por instancia. Cualquier edicion manual
/// fija `auto_managed=false` para que el launcher NO la sobrescriba (Regla 2).
pub fn set_config(
    id: &str,
    ram_mb: Option<u32>,
    jvm_args: Option<String>,
    gc: Option<String>,
    java_path: Option<String>,
) -> AppResult<InstanceMeta> {
    ensure_local(id)?;
    let mut meta = read_meta(id)
        .or_else(|| super::resolve_meta(id))
        .ok_or_else(|| AppError::msg("Sin metadata"))?;
    if let Some(r) = ram_mb {
        meta.ram_mb = r;
    }
    meta.jvm_args = jvm_args.filter(|s| !s.trim().is_empty());
    meta.gc = gc.filter(|s| !s.trim().is_empty());
    meta.java_path = java_path.filter(|s| !s.trim().is_empty());
    meta.auto_managed = false;
    write_meta(id, &meta)?;
    Ok(meta)
}

/// Reactiva la autogestion por hardware (vuelve a valores por defecto).
pub fn set_auto_managed(id: &str, auto: bool) -> AppResult<InstanceMeta> {
    ensure_local(id)?;
    let mut meta = read_meta(id)
        .or_else(|| super::resolve_meta(id))
        .ok_or_else(|| AppError::msg("Sin metadata"))?;
    meta.auto_managed = auto;
    if auto {
        meta.jvm_args = None;
        meta.gc = None;
    }
    write_meta(id, &meta)?;
    Ok(meta)
}

/// Fija el `version_id` del perfil instalado (tras instalar loader).
pub fn set_version_id(id: &str, version_id: &str) -> AppResult<()> {
    let mut meta = read_meta(id)
        .or_else(|| super::resolve_meta(id))
        .ok_or_else(|| AppError::msg("Sin metadata"))?;
    meta.version_id = Some(version_id.to_string());
    write_meta(id, &meta)
}

/// Actualiza loader y version; limpia version_id para forzar reinstalacion.
pub fn set_loader(id: &str, loader: &str, loader_version: &str) -> AppResult<InstanceMeta> {
    ensure_local(id)?;
    let mut meta = read_meta(id)
        .or_else(|| super::resolve_meta(id))
        .ok_or_else(|| AppError::msg("Sin metadata"))?;
    meta.loader = loader.trim().to_string();
    meta.loader_version = loader_version.trim().to_string();
    meta.version_id = None;
    super::sync_loader_icon(&mut meta);
    write_meta(id, &meta)?;
    Ok(meta)
}

pub fn duplicate(id: &str, new_name: &str) -> AppResult<Instance> {
    ensure_local(id)?;
    let src_meta = read_meta(id).unwrap_or_else(|| super::meta_from_folder_name(id));
    let mut folder = format!("{id}_copy");
    let mut n = 2;
    while instance_dir(&folder).exists() {
        folder = format!("{id}_copy{n}");
        n += 1;
    }
    copy_dir_all(&instance_dir(id), &instance_dir(&folder))?;
    let mut meta = src_meta;
    meta.name = if new_name.trim().is_empty() {
        format!("{} (copia)", meta.name)
    } else {
        new_name.trim().to_string()
    };
    meta.total_play_minutes = 0;
    meta.last_played = None;
    write_meta(&folder, &meta)?;
    Ok(meta.into_instance(&folder, &instance_dir(&folder)))
}

pub fn delete(id: &str) -> AppResult<()> {
    ensure_local(id)?;
    std::fs::remove_dir_all(instance_dir(id))?;
    Ok(())
}

/// Copia recursiva (para duplicar / importar).
pub fn copy_dir_all(src: &Path, dst: &Path) -> AppResult<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
