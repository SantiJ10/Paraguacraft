//! Preset **Fabric + Iris** (loader separado de Fabric).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::Value;
use tauri::AppHandle;

use crate::core::net::{self, DownloadItem};
use crate::core::paths;
use crate::error::{AppError, AppResult};

use super::fabric;
use crate::core::pvp_mod_stack;

const BUNDLE_SLUGS: &[&str] = &[
    "fabric-api",
    "sodium",
    "iris",
    "lithium",
    "ferrite-core",
    "entityculling",
    "immediatelyfast",
    "modmenu",
];

const CACHE_DAYS: u64 = 30;
const CACHE_MARKER_NAME: &str = ".cache_ok_v6";
const BUNDLE_MANIFEST: &str = "bundle_manifest.json";
const MODRINTH: &str = "https://api.modrinth.com/v2";
/// Mods que todo bundle Fabric+Iris debe tener; si falta alguno, se invalida el cache.
const REQUIRED_BUNDLE: &[&str] = &["fabric-api", "sodium", "iris", "lithium"];

fn cache_root(mc: &str) -> PathBuf {
    paths::default_minecraft_dir()
        .join("Paraguacraft_cache")
        .join("fabric_iris")
        .join(mc.replace('/', "_"))
}

fn cache_valid(marker: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(marker) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return false;
    };
    age.as_secs() / 86400 < CACHE_DAYS
}

pub async fn versions(client: &reqwest::Client, mc: &str) -> AppResult<Vec<String>> {
    fabric::versions(client, mc).await
}

pub async fn install_fabric_profile(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    loader_version: &str,
) -> AppResult<String> {
    fabric::install(app, client, mc, loader_version).await
}

pub async fn install_bundle(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    instance_dir: &Path,
) -> AppResult<()> {
    let cache = cache_root(mc);
    std::fs::create_dir_all(&cache)?;
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    let marker = cache.join(CACHE_MARKER_NAME);
    let manifest_path = cache.join(BUNDLE_MANIFEST);

    if !cache_valid(&marker) || !manifest_path.is_file() || !manifest_complete(&manifest_path)? {
        rebuild_cache(app, client, mc, &cache, &marker, &manifest_path).await?;
    }

    let manifest = read_manifest(&manifest_path)?;
    sync_bundle_to_instance(&cache, &mods_dir, &manifest)?;
    enforce_render_stack(&mods_dir, mc, 2);
    enforce_bundle_manifest(&mods_dir, &manifest);

    let missing: Vec<_> = REQUIRED_BUNDLE
        .iter()
        .filter_map(|slug| manifest.get(*slug).map(|fname| (*slug, fname.as_str())))
        .filter(|(_, fname)| !mods_dir.join(*fname).is_file())
        .map(|(slug, fname)| format!("{slug} ({fname})"))
        .collect();
    if !missing.is_empty() {
        return Err(AppError::msg(format!(
            "Bundle Fabric+Iris incompleto: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

fn enforce_render_stack(mods_dir: &Path, mc: &str, tier: u8) {
    pvp_mod_stack::enforce_pinned_stack(mods_dir, mc, tier);
}

pub fn enforce_render_stack_for_instance(mods_dir: &Path, mc: &str, tier: u8) {
    enforce_render_stack(mods_dir, mc, tier);
}

async fn rebuild_cache(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    cache: &Path,
    marker: &Path,
    manifest_path: &Path,
) -> AppResult<()> {
    for e in std::fs::read_dir(cache).into_iter().flatten().flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) == Some("jar") {
            let _ = std::fs::remove_file(p);
        }
    }
    if manifest_path.is_file() {
        let _ = std::fs::remove_file(manifest_path);
    }
    if marker.is_file() {
        let _ = std::fs::remove_file(marker);
    }

    if mc == "1.21.11" {
        return rebuild_cache_pinned(app, client, mc, cache, marker, manifest_path).await;
    }

    let mut items = Vec::new();
    let mut picked: HashMap<String, String> = HashMap::new();
    let mut manifest: HashMap<String, String> = HashMap::new();
    let mut forced: HashMap<String, String> = HashMap::new();

    let Some((iris_item, sodium_forced)) = resolve_compatible_iris(client, mc).await? else {
        return Err(AppError::msg(format!(
            "No se encontró par Iris+Sodium compatible para Minecraft {mc}"
        )));
    };
    if let Some(sodium_vid) = sodium_forced {
        forced.insert("sodium".into(), sodium_vid);
    }
    picked.insert("iris".into(), iris_item.version_id.clone());
    manifest.insert("iris".into(), iris_item.filename.clone());
    items.push(
        DownloadItem::new(iris_item.url, cache.join(&iris_item.filename)).with_sha1(iris_item.sha1),
    );

    for slug in BUNDLE_SLUGS.iter().filter(|s| **s != "iris") {
        let forced_vid = forced.get(*slug).map(String::as_str);
        let Some(item) = resolve_modrinth_jar(client, slug, mc, &picked, forced_vid).await? else {
            if REQUIRED_BUNDLE.contains(slug) {
                return Err(AppError::msg(format!(
                    "No se pudo resolver {slug} compatible para Fabric+Iris ({mc})"
                )));
            }
            continue;
        };
        picked.insert(slug.to_string(), item.version_id.clone());
        manifest.insert(slug.to_string(), item.filename.clone());
        items.push(
            DownloadItem::new(item.url, cache.join(&item.filename)).with_sha1(item.sha1),
        );
    }
    if !items.is_empty() {
        net::download_all(
            client,
            items,
            6,
            app,
            "fabric-iris-bundle",
            &format!("Mods Fabric + Iris ({mc})"),
        )
        .await?;
    }
    validate_manifest(&manifest, mc)?;
    let body = serde_json::to_string_pretty(&manifest)
        .map_err(|e| AppError::msg(format!("Manifest bundle: {e}")))?;
    std::fs::write(manifest_path, body)?;
    let _ = std::fs::write(marker, b"ok");
    Ok(())
}

async fn rebuild_cache_pinned(
    app: &AppHandle,
    client: &reqwest::Client,
    mc: &str,
    cache: &Path,
    marker: &Path,
    manifest_path: &Path,
) -> AppResult<()> {
    let mut items = Vec::new();
    let mut manifest: HashMap<String, String> = HashMap::new();

    for pin in pvp_mod_stack::bundle_pins(mc) {
        let version: Value =
            net::fetch_json(client, &format!("{MODRINTH}/version/{}", pin.version_id)).await?;
        let item = jar_item_from_version(pin.slug, &version)?;
        if item.filename != pin.filename {
            return Err(AppError::msg(format!(
                "Pin desactualizado: {} esperaba {}, Modrinth devolvió {}",
                pin.slug, pin.filename, item.filename
            )));
        }
        manifest.insert(pin.slug.to_string(), pin.filename.to_string());
        items.push(
            DownloadItem::new(item.url, cache.join(&item.filename)).with_sha1(item.sha1),
        );
    }

    if !items.is_empty() {
        net::download_all(
            client,
            items,
            6,
            app,
            "fabric-iris-bundle",
            &format!("Mods Fabric + Iris ({mc})"),
        )
        .await?;
    }
    validate_manifest(&manifest, mc)?;
    let body = serde_json::to_string_pretty(&manifest)
        .map_err(|e| AppError::msg(format!("Manifest bundle: {e}")))?;
    std::fs::write(manifest_path, body)?;
    let _ = std::fs::write(marker, b"ok");
    Ok(())
}

fn manifest_complete(path: &Path) -> AppResult<bool> {
    if !path.is_file() {
        return Ok(false);
    }
    let manifest = read_manifest(path)?;
    Ok(REQUIRED_BUNDLE.iter().all(|slug| manifest.contains_key(*slug)))
}

fn validate_manifest(manifest: &HashMap<String, String>, mc: &str) -> AppResult<()> {
    let missing: Vec<_> = REQUIRED_BUNDLE
        .iter()
        .filter(|slug| !manifest.contains_key(**slug))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(AppError::msg(format!(
            "Bundle Fabric+Iris incompleto para {mc}: faltan {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

/// Quita JARs del bundle cuyo nombre no coincide con el manifest (p.ej. Sodium 0.8.13).
fn enforce_bundle_manifest(mods_dir: &Path, manifest: &HashMap<String, String>) {
    for slug in BUNDLE_SLUGS {
        let Some(keep) = manifest.get(*slug) else {
            continue;
        };
        let keep_lower = keep.to_lowercase();
        let exclude: &[&str] = match *slug {
            "sodium" => &["sodium-extra", "reeses-sodium"],
            _ => &[],
        };
        for entry in std::fs::read_dir(mods_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !name.ends_with(".jar") && !name.ends_with(".jar.disabled") {
                continue;
            }
            if exclude.iter().any(|x| name.contains(x)) {
                continue;
            }
            if !name.contains(&slug.to_lowercase()) {
                continue;
            }
            if name.trim_end_matches(".disabled") != keep_lower {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

/// Busca Iris con pin explícito de Sodium resoluble (evita Iris reciente + Sodium incompatible).
async fn resolve_compatible_iris(
    client: &reqwest::Client,
    mc: &str,
) -> AppResult<Option<(JarItem, Option<String>)>> {
    let url = format!(
        "{MODRINTH}/project/iris/version?loaders={}&game_versions={}",
        net::url_encode(r#"["fabric"]"#),
        net::url_encode(&format!(r#"["{mc}"]"#))
    );
    let versions: Value = net::fetch_json(client, &url).await?;
    let arr = versions.as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        return Ok(None);
    }

    let sodium_pid = modrinth_project_id(client, "sodium").await.ok();

    for version in &arr {
        let mut item = jar_item_from_version("iris", version)?;
        let Some(ref pid) = sodium_pid else {
            return Ok(Some((item, None)));
        };
        let Some(forced_sodium) = pinned_dependency_version(version, pid) else {
            continue;
        };
        if resolve_modrinth_jar(
            client,
            "sodium",
            mc,
            &HashMap::new(),
            Some(&forced_sodium),
        )
        .await?
        .is_some()
        {
            item.required_dep_version = Some(forced_sodium.clone());
            return Ok(Some((item, Some(forced_sodium))));
        }
    }

    // Fallback: primera Iris publicada (versiones viejas sin pin exacto de Sodium).
    let item = jar_item_from_version("iris", &arr[0])?;
    let forced = sodium_pid
        .as_ref()
        .and_then(|pid| pinned_dependency_version(&arr[0], pid));
    Ok(Some((item, forced)))
}

fn read_manifest(path: &Path) -> AppResult<HashMap<String, String>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| AppError::msg(format!("No se pudo leer {BUNDLE_MANIFEST}: {e}")))?;
    serde_json::from_str(&text).map_err(|e| AppError::msg(format!("Manifest invalido: {e}")))
}

fn sync_bundle_to_instance(
    cache: &Path,
    mods_dir: &Path,
    manifest: &HashMap<String, String>,
) -> AppResult<()> {
    for slug in BUNDLE_SLUGS {
        remove_bundle_slug_jars(mods_dir, slug);
    }
    for slug in BUNDLE_SLUGS {
        let Some(fname) = manifest.get(*slug) else {
            continue;
        };
        let src = cache.join(fname);
        if !src.is_file() {
            continue;
        }
        let dest = mods_dir.join(fname);
        if dest.is_file() {
            let _ = std::fs::remove_file(&dest);
        }
        let disabled = dest.with_extension("jar.disabled");
        if disabled.is_file() {
            let _ = std::fs::remove_file(disabled);
        }
        std::fs::copy(&src, &dest)
            .map_err(|e| AppError::msg(format!("No se pudo copiar {fname}: {e}")))?;
    }
    Ok(())
}

/// Quita JARs del bundle en `mods/` (p.ej. sodium) sin tocar sub-mods (`sodium-extra`).
fn remove_bundle_slug_jars(mods_dir: &Path, slug: &str) {
    let slug = slug.to_lowercase();
    let exclude: &[&str] = match slug.as_str() {
        "sodium" => &["sodium-extra", "reeses-sodium"],
        "fabric-api" => &[],
        _ => &[],
    };
    for entry in std::fs::read_dir(mods_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        if !name.ends_with(".jar") && !name.ends_with(".jar.disabled") {
            continue;
        }
        if exclude.iter().any(|x| name.contains(x)) {
            continue;
        }
        if name.contains(&slug) {
            let _ = std::fs::remove_file(path);
        }
    }
}

struct JarItem {
    url: String,
    filename: String,
    sha1: Option<String>,
    version_id: String,
    /// Si esta versión fija una dependencia requerida con `version_id` exacto (p.ej. Iris
    /// pineando una versión concreta de Sodium), queda acá para forzar esa misma versión
    /// al resolver esa dependencia más abajo en el bundle.
    required_dep_version: Option<String>,
}

async fn modrinth_project_id(client: &reqwest::Client, slug: &str) -> AppResult<String> {
    let p: Value = net::fetch_json(client, &format!("{MODRINTH}/project/{slug}")).await?;
    p["id"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AppError::msg(format!("Modrinth: sin id para {slug}")))
}

fn deps_ok(version: &Value, picked: &HashMap<String, String>, ids: &HashMap<String, String>) -> bool {
    let Some(deps) = version["dependencies"].as_array() else {
        return true;
    };
    for dep in deps {
        if dep["dependency_type"].as_str() == Some("incompatible") {
            continue;
        }
        let Some(pid) = dep["project_id"].as_str() else { continue };
        let Some(required_vid) = dep["version_id"].as_str() else { continue };
        for (slug, id) in ids {
            if id == pid {
                if let Some(have) = picked.get(slug) {
                    if have != required_vid {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Extrae, si existe, la dependencia requerida de `version` hacia `dep_project_id` con
/// `version_id` exacto pineado (p.ej. la versión de Sodium que una versión de Iris exige).
fn pinned_dependency_version(version: &Value, dep_project_id: &str) -> Option<String> {
    version["dependencies"].as_array()?.iter().find_map(|d| {
        if d["dependency_type"].as_str() == Some("incompatible") {
            return None;
        }
        if d["project_id"].as_str() != Some(dep_project_id) {
            return None;
        }
        d["version_id"].as_str().map(String::from)
    })
}

fn jar_item_from_version(slug: &str, version: &Value) -> AppResult<JarItem> {
    let vid = version["id"].as_str().unwrap_or_default().to_string();
    let files = version["files"].as_array().cloned().unwrap_or_default();
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool().unwrap_or(false))
        .or_else(|| files.first())
        .ok_or_else(|| AppError::msg(format!("Modrinth: {slug} sin archivos")))?;
    Ok(JarItem {
        url: file["url"].as_str().unwrap_or_default().to_string(),
        filename: file["filename"].as_str().unwrap_or("mod.jar").to_string(),
        sha1: file["hashes"]["sha1"].as_str().map(String::from),
        version_id: vid,
        required_dep_version: None,
    })
}

async fn resolve_modrinth_jar(
    client: &reqwest::Client,
    slug: &str,
    mc: &str,
    picked: &HashMap<String, String>,
    // Si viene seteado (p.ej. Sodium forzado por el pin de Iris), se busca esa versión
    // exacta en vez de aplicar la heurística "más nueva compatible". Si no aparece en el
    // listado filtrado por mc/loader, se la busca directo por id (Modrinth la sigue
    // devolviendo aunque no sea la última).
    forced_version_id: Option<&str>,
) -> AppResult<Option<JarItem>> {
    let url = format!(
        "{MODRINTH}/project/{slug}/version?loaders={}&game_versions={}",
        net::url_encode(r#"["fabric"]"#),
        net::url_encode(&format!(r#"["{mc}"]"#))
    );
    let versions: Value = net::fetch_json(client, &url).await?;
    let arr = versions.as_array().cloned().unwrap_or_default();

    if let Some(forced) = forced_version_id {
        if let Some(version) = arr.iter().find(|v| v["id"].as_str() == Some(forced)) {
            return Ok(Some(jar_item_from_version(slug, version)?));
        }
        // No vino en el listado filtrado (puede pasar si Modrinth no le marcó ese
        // game_version explícitamente) — la pedimos directo por id, es la que Iris exige.
        let version: Value = net::fetch_json(client, &format!("{MODRINTH}/version/{forced}")).await?;
        if version["id"].as_str() == Some(forced) {
            return Ok(Some(jar_item_from_version(slug, &version)?));
        }
        return Ok(None);
    }

    if arr.is_empty() {
        return Ok(None);
    }

    let mut ids: HashMap<String, String> = HashMap::new();
    for s in ["sodium", "iris", "fabric-api"] {
        if let Ok(id) = modrinth_project_id(client, s).await {
            ids.insert(s.to_string(), id);
        }
    }

    for version in &arr {
        if !deps_ok(version, picked, &ids) {
            continue;
        }
        let mut item = jar_item_from_version(slug, version)?;
        if slug == "iris" {
            item.required_dep_version = ids
                .get("sodium")
                .and_then(|pid| pinned_dependency_version(version, pid));
        }
        return Ok(Some(item));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// JSON real de Modrinth: Iris 1.10.7 para Fabric 1.21.11 exige exactamente
    /// Sodium `UddlN6L4` (mc1.21.11-0.8.7-fabric), que ya no es la última versión de
    /// Sodium publicada (siempre hay una más nueva). Sin este pin, el bundle se queda
    /// intentando emparejar el Sodium más nuevo contra el pin de Iris y nunca coincide,
    /// por lo que Iris termina sin descargarse.
    fn iris_1_21_11_version() -> Value {
        json!({
            "id": "fDpuVzVr",
            "version_number": "1.10.7+1.21.11-fabric",
            "dependencies": [
                { "version_id": "UddlN6L4", "project_id": "AANobbMI", "dependency_type": "required" }
            ],
            "files": [{
                "primary": true,
                "url": "https://cdn.modrinth.com/data/YL57xq9U/versions/fDpuVzVr/iris-fabric-1.10.7%2Bmc1.21.11.jar",
                "filename": "iris-fabric-1.10.7+mc1.21.11.jar",
                "hashes": { "sha1": "aae8567bd9ea397d50aff1d0b680a82ffe67040c" }
            }]
        })
    }

    #[test]
    fn extracts_pinned_sodium_version_from_iris() {
        let v = iris_1_21_11_version();
        assert_eq!(
            pinned_dependency_version(&v, "AANobbMI"),
            Some("UddlN6L4".to_string())
        );
        // Un project_id que no aparece en las dependencias no debe devolver nada.
        assert_eq!(pinned_dependency_version(&v, "otro-proyecto"), None);
    }

    #[test]
    fn jar_item_carries_filename_and_sha1() {
        let v = iris_1_21_11_version();
        let item = jar_item_from_version("iris", &v).unwrap();
        assert_eq!(item.version_id, "fDpuVzVr");
        assert_eq!(item.filename, "iris-fabric-1.10.7+mc1.21.11.jar");
        assert_eq!(item.sha1, Some("aae8567bd9ea397d50aff1d0b680a82ffe67040c".to_string()));
    }

    #[test]
    fn incompatible_dependency_is_ignored_when_pinning() {
        let v = json!({
            "id": "x",
            "dependencies": [
                { "version_id": "should-not-count", "project_id": "AANobbMI", "dependency_type": "incompatible" }
            ],
            "files": [{ "primary": true, "url": "u", "filename": "f.jar", "hashes": {} }]
        });
        assert_eq!(pinned_dependency_version(&v, "AANobbMI"), None);
    }

    #[test]
    fn enforce_render_stack_removes_sodium_813_with_iris_1107() {
        let tmp = std::env::temp_dir().join(format!(
            "paraguacraft-enforce-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("iris-fabric-1.10.7+mc1.21.11.jar"),
            b"iris",
        )
        .unwrap();
        std::fs::write(
            tmp.join("sodium-fabric-0.8.13+mc1.21.11.jar"),
            b"bad",
        )
        .unwrap();
        std::fs::write(
            tmp.join("sodium-fabric-0.8.12+mc1.21.11.jar"),
            b"ok",
        )
        .unwrap();
        enforce_render_stack(&tmp, "1.21.11", 2);
        assert!(!tmp.join("sodium-fabric-0.8.13+mc1.21.11.jar").exists());
        assert!(tmp.join("sodium-fabric-0.8.12+mc1.21.11.jar").exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
