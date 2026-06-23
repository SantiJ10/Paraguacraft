//! Comandos de diagnostico de crashes + IA heuristica.

use crate::core::ai::{self, AiRequest};
use crate::core::diagnostics::{self, CrashDiagnosis};
use crate::error::AppResult;

#[tauri::command]
pub fn diagnose_instance(instance_id: String, exit_code: i32) -> AppResult<CrashDiagnosis> {
    diagnostics::analyze_instance(&instance_id, exit_code)
}

#[tauri::command]
pub fn ai_assist(prompt: String, diagnosis: Option<CrashDiagnosis>) -> ai::AiResponse {
    ai::analyze_prompt(&AiRequest {
        prompt,
        log_tail: diagnosis.as_ref().map(|d| d.log_tail.clone()),
        diagnosis,
    })
}
