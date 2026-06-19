//! FASE 4 - IA global (provider-agnostico).
//!
//! Define el trait `AiProvider` para que la asistencia por IA del launcher sea
//! enchufable: hoy una implementacion heuristica/local, manana OpenAI/Anthropic
//! u un backend propio, sin cambiar la UI ni el resto del backend.

/// Contexto que la UI/diagnostico pasa al proveedor de IA.
pub struct AiRequest {
    pub prompt: String,
    pub log_tail: Option<String>,
}

/// Respuesta uniforme del proveedor.
pub struct AiResponse {
    pub message: String,
    pub suggestions: Vec<String>,
}

/// Proveedor de IA enchufable. Implementaciones futuras: `LocalHeuristic`,
/// `OpenAiProvider`, `OwnBackendProvider`.
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, req: &AiRequest) -> Result<AiResponse, String>;
}

/// Stub heuristico para Fase 1: responde sin red. Se reemplaza/extiende en Fase 4.
pub struct LocalHeuristic;

impl AiProvider for LocalHeuristic {
    fn name(&self) -> &str {
        "local-heuristic"
    }

    fn complete(&self, _req: &AiRequest) -> Result<AiResponse, String> {
        Ok(AiResponse {
            message: "Asistente IA en construccion (Fase 4).".to_string(),
            suggestions: vec![],
        })
    }
}
