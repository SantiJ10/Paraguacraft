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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub configured: bool,
    pub provider: Option<String>,
}

pub fn ai_status() -> AiStatus {
    keys::resolve_llm_config().map_or(
        AiStatus {
            configured: false,
            provider: None,
        },
        |llm| AiStatus {
            configured: true,
            provider: Some(llm.provider.to_string()),
        },
    )
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
                "Paraguabot necesita una API key. Agrega GROQ_API_KEY en launcher/.env, en \
                 %APPDATA%/ParaguacraftLauncher/.env, o en Ajustes > Paraguabot."
                    .into()
            },
            suggestions: vec![
                "Ajustes > Paraguabot: pega tu Groq API key (gratis en console.groq.com).".into(),
                "Reinicia el launcher despues de editar .env.".into(),
                "Usa el panel de diagnostico para crashes automaticos.".into(),
            ],
        })
    }
}

const KNOWLEDGE: &str = r#"
LAUNCHER Paraguacraft (Tauri v2 + Rust + Vue 3):
- 0% CPU en segundo plano al jugar. Instancias vanilla/Fabric/Forge/NeoForge/Quilt + preset PvP 1.8.9.
- Tienda Modrinth + CurseForge (key CF en Ajustes). Modpacks .mrpack/.zip. Servidores Paper/Fabric/Forge + Playit.gg.
- Ajustes: RAM, GC, options.txt, cerrar al jugar, Discord RPC, Java Temurin 17/21, reparar instancia, auto-update SHA-256.

CLIENTE Paraguacraft PvP Modern (Fabric 1.21.11 + Iris/Sodium, v0.6.4):
- Loader paraguacraft-pvp-modern. Mod Menu (Right Shift), menu custom, Hypixel Quick Play, borderless LWJGL3.
- HUD: FPS, ping, CPS, keystrokes, armadura vertical, bloques, musica Spotify/YT, pociones estilo 1.8.9.
- Toggle sprint legacy (W). Config persiste en options.txt (launcher ya no la resetea al actualizar).
- Discord RPC in-game: usuario - version - loader + servidor/mundo/menu.

CLIENTE Paraguacraft PvP (Forge 1.8.9 + OptiFine + mod propio, v2.1.28):
- Hypixel-safe: solo HUD/visual/rendimiento. Mod Menu (Right Shift). Hytils camas, OptiFine, mod ParaguacraftPvP.
- Toggle Sprint: (M) teclas virtuales Lunar, (N) legacy. Toggle Sneak en Mod Menu (Shift alterna).
- HUD: FPS, ping, CPS, keystrokes, reach, combo, armadura, freelook, Quick Play Hypixel, alertas chat, musica overlay.
- Boost FPS + culling en Mod Menu > Rendimiento. Config: .minecraft/paraguacraft_v2.properties.
- Actualiza solo al Iniciar desde manifest GitHub (Ajustes > Cliente PvP muestra version).

SETTINGS PvP recomendados: Boost FPS ON, entity/nametag cull ON, Toggle Sprint (M) ON, 4-8GB RAM, perfil hardware en Ajustes.
"#;

const SYSTEM_PROMPT: &str = "Sos Paraguabot, experto en el launcher Paraguacraft, el cliente PvP 1.8.9 y PvP Modern 1.21.11. \
Responde en espanol rioplatense, claro y accionable. Usa el conocimiento del producto abajo. \
Para crashes inclui pasos concretos (Reparar instancia, Ajustes > Java, actualizar cliente PvP). \
No inventes rutas; referi Ajustes, Versiones, Mod Menu (Right Shift) y teclas M/N para sprint. \
Si no sabes algo especifico del PC del usuario, pedi el log o crash-report.";

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
            "Actualiza Paraguacraft PvP desde Ajustes > Cliente PvP.".into(),
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

fn build_user_content(req: &AiRequest) -> String {
    let mut parts = vec![req.prompt.trim().to_string()];

    if let Some(d) = &req.diagnosis {
        parts.push(format!(
            "\n\n[Diagnostico reciente de crash]\nCategoria: {}\n{}\n{}\nSugerencias: {}",
            d.category,
            d.message,
            d.hint,
            d.suggestions.join("; ")
        ));
        if let Some(line) = &d.error_line {
            parts.push(format!("\nLinea de error: {line}"));
        }
    }

    if let Some(tail) = &req.log_tail {
        if !tail.trim().is_empty() {
            let truncated: String = tail.chars().rev().take(4000).collect::<String>().chars().rev().collect();
            parts.push(format!("\n\nLog reciente:\n{truncated}"));
        }
    }

    parts.join("")
}

pub async fn analyze_prompt_async(state: &AppState, req: &AiRequest) -> AppResult<AiResponse> {
    let prompt = req.prompt.trim();
    if prompt.is_empty() {
        return Ok(analyze_prompt(req));
    }
    let Some(llm) = keys::resolve_llm_config() else {
        return Ok(analyze_prompt(req));
    };

    let system = format!("{SYSTEM_PROMPT}\n\n--- CONOCIMIENTO PARAGUACRAFT ---\n{KNOWLEDGE}");
    let user_content = build_user_content(req);

    let models: Vec<&str> = if llm.provider == "groq" {
        vec![
            llm.model,
            "openai/gpt-oss-120b",
            "openai/gpt-oss-20b",
            "llama-3.3-70b-versatile",
            "llama-3.1-8b-instant",
        ]
    } else {
        vec![llm.model]
    };

    let (client, _guard) = state.net_scope();
    let mut last_err = String::new();

    for model in models {
        let body = json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user_content }
            ],
            "max_tokens": 1024,
            "temperature": 0.35
        });

        let resp = match client
            .post(llm.chat_url)
            .header("Authorization", format!("Bearer {}", llm.api_key))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                last_err = format!("Paraguabot ({}) red: {e}", llm.provider);
                continue;
            }
        };

        if !resp.status().is_success() {
            last_err = format!(
                "Paraguabot ({}) API {}: {}",
                llm.provider,
                resp.status(),
                resp.text().await.unwrap_or_default().chars().take(300).collect::<String>()
            );
            continue;
        }

        let json: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                last_err = format!("Paraguabot JSON: {e}");
                continue;
            }
        };

        let message = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Sin respuesta del modelo.")
            .trim()
            .to_string();

        return Ok(AiResponse {
            message,
            suggestions: vec![],
        });
    }

    Err(AppError::msg(if last_err.is_empty() {
        "Paraguabot no pudo contactar al proveedor.".into()
    } else {
        last_err
    }))
}
