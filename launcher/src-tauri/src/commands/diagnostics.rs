//! Comandos de diagnostico de crashes + Paraguabot.

use tauri::State;

use crate::core::ai::{self, AiRequest};
use crate::core::diagnostics::{self, CrashDiagnosis};
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub fn diagnose_instance(instance_id: String, exit_code: i32) -> AppResult<CrashDiagnosis> {
    diagnostics::analyze_instance(&instance_id, exit_code)
}

#[tauri::command]
pub async fn ai_assist(
    state: State<'_, AppState>,
    prompt: String,
    diagnosis: Option<CrashDiagnosis>,
    active_instance_id: Option<String>,
) -> AppResult<ai::AiResponse> {
    ai::analyze_prompt_async(
        &state,
        &AiRequest {
            prompt,
            log_tail: diagnosis.as_ref().map(|d| d.log_tail.clone()),
            diagnosis,
            active_instance_id,
        },
    )
    .await
}

#[tauri::command]
pub fn ai_status() -> ai::AiStatus {
    ai::ai_status()
}

#[tauri::command]
pub fn save_groq_api_key(key: String) -> AppResult<()> {
    crate::config::keys::save_groq_api_key(&key)
}
