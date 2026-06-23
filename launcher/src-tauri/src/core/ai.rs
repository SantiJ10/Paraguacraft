//! IA global (provider-agnostico) + heuristicas de diagnostico.

use crate::core::diagnostics::CrashDiagnosis;

/// Contexto que la UI/diagnostico pasa al proveedor de IA.
pub struct AiRequest {
    pub prompt: String,
    pub log_tail: Option<String>,
    pub diagnosis: Option<CrashDiagnosis>,
}

/// Respuesta uniforme del proveedor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiResponse {
    pub message: String,
    pub suggestions: Vec<String>,
}

pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, req: &AiRequest) -> Result<AiResponse, String>;
}

pub struct LocalHeuristic;

impl AiProvider for LocalHeuristic {
    fn name(&self) -> &str {
        "local-heuristic"
    }

    fn complete(&self, req: &AiRequest) -> Result<AiResponse, String> {
        if let Some(d) = &req.diagnosis {
            let mut message = format!("{}\n\n{}", d.message, d.hint);
            if let Some(line) = &d.error_line {
                message.push_str("\n\nDetalle:\n");
                message.push_str(line);
            }
            return Ok(AiResponse {
                message,
                suggestions: d.suggestions.clone(),
            });
        }
        Ok(AiResponse {
            message: if req.prompt.trim().is_empty() {
                "Describe el problema o abri el diagnostico de crash.".into()
            } else {
                format!("Recibido: {}. Usa el panel de diagnostico para crashes automaticos.", req.prompt)
            },
            suggestions: vec![],
        })
    }
}

pub fn suggestions_for_category(category: &str) -> Vec<String> {
    match category {
        "oom_java" => vec![
            "Reduce mods pesados o shaders.".into(),
            "Cierra otras aplicaciones antes de jugar.".into(),
        ],
        "oom_reserve" => vec!["Prueba con 4096 MB en lugar de 8 GB.".into()],
        "auth" => vec!["Reconecta tu cuenta Microsoft.".into()],
        "mods" => vec![
            "Quita mods recientes uno por uno.".into(),
            "Verifica compatibilidad MC + loader.".into(),
        ],
        "gpu" => vec!["Actualiza drivers NVIDIA/AMD/Intel.".into()],
        "java_version" => vec!["Instala Temurin 17 o 21 en Ajustes → Java.".into()],
        "network" => vec!["Revisa firewall o VPN.".into()],
        "install" => vec![
            "Reinstala la versión desde la pestaña Versiones.".into(),
            "Borrá la carpeta de la versión en `.minecraft/versions/` si el error persiste.".into(),
        ],
        "permissions" => vec!["Excluí la carpeta `.minecraft` del antivirus.".into()],
        "launch_early" | "launch_abort" => vec![
            "Verificá que el Java correcto esté seleccionado en la instancia.".into(),
            "Reinstalá la versión de Minecraft desde Versiones.".into(),
        ],
        _ => vec!["Comparte el crash-report en Discord si persiste.".into()],
    }
}

pub fn analyze_prompt(req: &AiRequest) -> AiResponse {
    LocalHeuristic.complete(req).unwrap_or(AiResponse {
        message: "Error interno de Paraguabot.".into(),
        suggestions: vec![],
    })
}
