//! IA global (Groq -> OpenAI -> heuristicas de diagnostico).

use serde_json::json;

use crate::config::keys;
use crate::core::diagnostics::CrashDiagnosis;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

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
                "Paraguabot necesita una API key. Agrega GROQ_API_KEY (recomendado) o OPENAI_API_KEY en launcher/.env.".into()
            },
            suggestions: vec![
                "Configura GROQ_API_KEY en launcher/.env (mas rapido y economico).".into(),
                "Usa el panel de diagnostico para crashes automaticos.".into(),
            ],
        })
    }
}

const SYSTEM_PROMPT: &str = "Sos Paraguabot, asistente del launcher Paraguacraft (Minecraft Java). \
Responde en espanol, breve y accionable. Ayuda con versiones, mods, Forge/Fabric, Java, RAM, crashes, servidores locales y Hypixel PvP 1.8.9. \
No inventes rutas de archivos; sugeri Ajustes, Reparar instancia o Versiones del launcher.";

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
        "hypixel_scoreboard" => vec![
            "Actualiza Paraguacraft PvP desde Versiones -> Instalar.".into(),
            "Si el juego funciona, podes ignorar esos mensajes en el log.".into(),
        ],
        "clean_exit" => vec!["No hace falta actualizar drivers si el juego cerro bien.".into()],
        "java_version" => vec!["Instala Temurin 17 o 21 en Ajustes -> Java.".into()],
        "network" => vec!["Revisa firewall o VPN.".into()],
        "install" => vec![
            "Reinstala la version desde la pestana Versiones.".into(),
            "Borra la carpeta de la version en `.minecraft/versions/` si el error persiste.".into(),
        ],
        "permissions" => vec!["Exclui la carpeta `.minecraft` del antivirus.".into()],
        "launch_early" | "launch_abort" => vec![
            "Verifica que el Java correcto este seleccionado en la instancia.".into(),
            "Reinstala la version de Minecraft desde Versiones.".into(),
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

pub async fn analyze_prompt_async(state: &AppState, req: &AiRequest) -> AppResult<AiResponse> {
    if req.diagnosis.is_some() {
        return Ok(analyze_prompt(req));
    }
    let prompt = req.prompt.trim();
    if prompt.is_empty() {
        return Ok(analyze_prompt(req));
    }
    let Some(llm) = keys::resolve_llm_config() else {
        return Ok(analyze_prompt(req));
    };

    let user_content = if let Some(tail) = &req.log_tail {
        format!("{prompt}\n\nLog reciente:\n{tail}")
    } else {
        prompt.to_string()
    };

    let body = json!({
        "model": llm.model,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": user_content }
        ],
        "max_tokens": 512,
        "temperature": 0.4
    });

    let (client, _guard) = state.net_scope();
    let resp = client
        .post(llm.chat_url)
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Paraguabot ({}) red: {e}", llm.provider)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        return Err(AppError::msg(format!(
            "Paraguabot ({}) API {status}: {}",
            llm.provider,
            err_body.chars().take(200).collect::<String>()
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::msg(format!("Paraguabot JSON: {e}")))?;

    let message = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Sin respuesta del modelo.")
        .trim()
        .to_string();

    Ok(AiResponse {
        message,
        suggestions: vec![],
    })
}
