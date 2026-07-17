//! Detección on-demand de conflictos conocidos entre mods.

use std::collections::HashMap;

use crate::core::instances::content;
use crate::error::AppResult;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModConflict {
    /// "warn" | "error"
    pub severity: String,
    pub title: String,
    pub detail: String,
    pub mods: Vec<String>,
}

fn norm(name: &str) -> String {
    name.to_lowercase()
        .replace(['-', '_', ' '], "")
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn family(name: &str) -> String {
    let n = name.to_lowercase();
    if n.contains("optifine") {
        return "optifine".into();
    }
    if n.contains("paraguacraftpvp") {
        return "paraguacraftpvp".into();
    }
    if n.contains("essential") {
        return "essential".into();
    }
    if n.contains("patcher") {
        return "patcher".into();
    }
    if n.contains("iris") {
        return "iris".into();
    }
    if n.contains("sodium") {
        return "sodium".into();
    }
    if n.contains("rubidium") {
        return "rubidium".into();
    }
    norm(name).chars().take(12).collect()
}

pub fn scan(instance_id: &str) -> AppResult<Vec<ModConflict>> {
    let entries = content::list(instance_id)?;
    let mods: Vec<_> = entries
        .iter()
        .filter(|e| e.folder == "mods" && e.enabled)
        .collect();

    let mut out = Vec::new();
    let mut by_family: HashMap<String, Vec<String>> = HashMap::new();

    for m in &mods {
        by_family
            .entry(family(&m.name))
            .or_default()
            .push(m.name.clone());
    }

    for (fam, names) in &by_family {
        if names.len() > 1 && matches!(fam.as_str(), "optifine" | "paraguacraftpvp" | "iris") {
            out.push(ModConflict {
                severity: "error".into(),
                title: format!("Varias copias de {fam}"),
                detail: "Dejá solo un JAR de este mod; los duplicados suelen crashear al iniciar.".into(),
                mods: names.clone(),
            });
        }
    }

    let has_optifine = by_family.contains_key("optifine");
    let has_iris = by_family.contains_key("iris");
    if has_optifine && has_iris {
        out.push(ModConflict {
            severity: "error".into(),
            title: "OptiFine + Iris".into(),
            detail: "No conviven en la misma instancia. Usá Fabric+Iris o Forge+OptiFine, no ambos.".into(),
            mods: by_family
                .get("optifine")
                .into_iter()
                .chain(by_family.get("iris"))
                .flatten()
                .cloned()
                .collect(),
        });
    }

    if by_family.contains_key("essential") || by_family.contains_key("patcher") {
        out.push(ModConflict {
            severity: "warn".into(),
            title: "Essential / Patcher detectado".into(),
            detail: "Paraguacraft PvP ya no los incluye: agregan procesos extra y reinicios. Quitálos si no los necesitás.".into(),
            mods: by_family
                .get("essential")
                .into_iter()
                .chain(by_family.get("patcher"))
                .flatten()
                .cloned()
                .collect(),
        });
    }

    if mods.len() > 35 {
        out.push(ModConflict {
            severity: "warn".into(),
            title: "Muchos mods activos".into(),
            detail: format!(
                "{} mods en mods/ — en PCs de 4–8 GB puede bajar FPS o alargar el arranque.",
                mods.len()
            ),
            mods: Vec::new(),
        });
    }

    Ok(out)
}
