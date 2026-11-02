//! Loader module for Horizon (Rust + Vulkan native Minecraft 26.3).
//!
//! Horizon es un motor nativo experimental que no usa Java. Se integra como un
//! "loader" especial que crea una instancia en la carpeta `instancias/` y
//! lanza el binario nativo directamente.

use reqwest::Client;
use tauri::AppHandle;

use crate::core::horizon::{self as horizon_core, HORIZON_MC_VERSION};
use crate::core::instances;
use crate::error::{AppError, AppResult};

/// Returns available Horizon versions (always just "latest" for now).
pub async fn versions(_client: &Client, mc: &str) -> AppResult<Vec<String>> {
    if mc != HORIZON_MC_VERSION {
        return Ok(vec![]);
    }
    // Horizon doesn't have multiple versions in the traditional sense.
    // We return "latest" to indicate it's available.
    Ok(vec!["latest".to_string()])
}

/// Installs Horizon and creates an instance in the instancias folder.
/// Returns the "version_id" which for Horizon is the instance folder name.
pub async fn install(
    app: &AppHandle,
    client: &Client,
    mc: &str,
    _loader_version: &str,
) -> AppResult<String> {
    if mc != HORIZON_MC_VERSION {
        return Err(AppError::msg(format!(
            "Horizon solo esta disponible para Minecraft {HORIZON_MC_VERSION}"
        )));
    }

    // Ensure vanilla 26.3 is installed (for assets)
    if !crate::core::versions::vanilla_locally_complete(mc) {
        crate::core::versions::install_vanilla(app, client, mc).await?;
    }

    // Install Horizon engine (downloads and extracts the native binary)
    let status = horizon_core::install_engine(app, client).await?;

    // Create the instance in the instancias folder
    let instance_folder = instances::folder_for(mc, "horizon");
    let instance_dir = instances::instance_dir(&instance_folder);
    std::fs::create_dir_all(&instance_dir)?;

    // Create instance metadata
    let meta = instances::InstanceMeta {
        name: format!("{} Horizon", mc),
        icon: instances::icon_for_loader("horizon"),
        mc_version: mc.to_string(),
        loader: "horizon".to_string(),
        loader_version: status.installed_tag.unwrap_or_else(|| "latest".to_string()),
        source: "paraguacraft".to_string(),
        ram_mb: 2048, // Horizon uses less RAM
        ram_min_mb: 1024,
        total_play_minutes: 0,
        last_played: None,
        version_id: Some(format!("horizon-{}", mc)),
        auto_managed: true,
        jvm_args: None,
        gc: None,
        java_path: None,
        performance_tier: None,
        show_game_console: None,
    };
    instances::write_meta(&instance_folder, &meta)?;

    // Return the "version_id" for Horizon - this is the instance folder name
    Ok(format!("horizon-{}", mc))
}