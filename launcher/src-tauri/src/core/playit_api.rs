//! Cliente mínimo de la API de playit.gg con autenticación `agent-key`.
//!
//! Replica lo que hace el plugin playit-minecraft (`/v1/tunnels/list|create`):
//! - si no hay túnel Java → lo crea
//! - si el server es Geyser y no hay Bedrock → lo crea (UDP 19132)
//!
//! Nota: el plugin Paper solo enruta TCP. Para que Bedrock funcione de verdad
//! hay que correr el agente desktop (`playit.exe`) con el mismo secret.

use serde_json::{json, Value};

use crate::error::{AppError, AppResult};

const API_URL: &str = "https://api.playit.gg";

#[derive(Debug, Clone, Default)]
pub struct EnsuredTunnels {
    pub java_address: Option<String>,
    pub bedrock_address: Option<String>,
    pub created_java: bool,
    pub created_bedrock: bool,
    pub messages: Vec<String>,
}

fn post_json(secret: &str, path: &str, body: Value) -> AppResult<Value> {
    let secret = secret.trim();
    if secret.len() < 32 {
        return Err(AppError::msg("Secret playit inválido o incompleto"));
    }
    let url = format!("{API_URL}{path}");
    let resp = tauri::async_runtime::block_on(async {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(25))
            .build()
            .map_err(|e| AppError::msg(e.to_string()))?;
        let res = client
            .post(&url)
            .header("Authorization", format!("agent-key {secret}"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::msg(format!("playit API red: {e}")))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::msg(format!("playit API body: {e}")))?;
        if !status.is_success() {
            return Err(AppError::msg(format!(
                "playit API {path} → HTTP {status}: {}",
                text.chars().take(240).collect::<String>()
            )));
        }
        serde_json::from_str::<Value>(&text)
            .map_err(|e| AppError::msg(format!("playit API JSON inválido: {e}")))
    })?;
    Ok(resp)
}

fn api_ok_data(v: &Value) -> Option<&Value> {
    if v.get("status").and_then(|s| s.as_str()) == Some("success") {
        v.get("data")
    } else {
        None
    }
}

fn tunnel_type_str(t: &Value) -> Option<&str> {
    t.get("tunnel_type")
        .and_then(|x| x.as_str())
        .or_else(|| t.get("tunnelType").and_then(|x| x.as_str()))
}

/// Extrae dirección visible de un túnel account (`connect_addresses`) o agent (`display_address`).
fn extract_display(tunnel: &Value) -> Option<String> {
    if let Some(d) = tunnel
        .get("display_address")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
    {
        return Some(d.to_string());
    }
    let addrs = tunnel.get("connect_addresses")?.as_array()?;
    for a in addrs {
        let typ = a.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let val = a.get("value")?;
        match typ {
            "auto" | "domain" | "addr4" | "addr6" => {
                if let Some(addr) = val.get("address").and_then(|x| x.as_str()) {
                    if !addr.is_empty() {
                        return Some(addr.to_string());
                    }
                }
            }
            "ip4" | "ip6" => {
                let address = val.get("address").and_then(|x| x.as_str())?;
                let port = val
                    .get("default_port")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0);
                if port > 0 {
                    return Some(format!("{address}:{port}"));
                }
                return Some(address.to_string());
            }
            _ => {
                if let Some(addr) = val.get("address").and_then(|x| x.as_str()) {
                    if !addr.is_empty() {
                        return Some(addr.to_string());
                    }
                }
            }
        }
    }
    None
}

fn list_account_tunnels(secret: &str) -> AppResult<Vec<Value>> {
    let resp = post_json(secret, "/v1/tunnels/list", json!({}))?;
    let data = api_ok_data(&resp).ok_or_else(|| {
        AppError::msg(format!(
            "playit tunnels/list falló: {}",
            resp.to_string().chars().take(200).collect::<String>()
        ))
    })?;
    Ok(data
        .get("tunnels")
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_default())
}

fn agent_rundata(secret: &str) -> AppResult<Value> {
    let resp = post_json(secret, "/v1/agents/rundata", json!({}))?;
    api_ok_data(&resp)
        .cloned()
        .ok_or_else(|| AppError::msg("playit agents/rundata sin data"))
}

fn find_by_type(tunnels: &[Value], want: &str) -> Option<String> {
    for t in tunnels {
        if tunnel_type_str(t) == Some(want) {
            if let Some(addr) = extract_display(t) {
                return Some(addr);
            }
        }
    }
    None
}

fn create_tunnel(
    secret: &str,
    name: &str,
    tunnel_type: &str,
    agent_id: Option<&str>,
    local_port: u16,
) -> AppResult<()> {
    let mut fields = vec![json!({"name": "local_ip", "value": "127.0.0.1"})];
    if local_port > 0 {
        fields.push(json!({"name": "local_port", "value": local_port.to_string()}));
    }
    let body = json!({
        "name": name,
        "protocol": { "type": "tunnel-type", "details": tunnel_type },
        "origin": {
            "type": "agent",
            "data": {
                "agent_id": agent_id,
                "config": { "fields": fields }
            }
        },
        "endpoint": {
            "type": "region",
            "details": { "region": "global", "port": null }
        },
        "enabled": true
    });
    let resp = post_json(secret, "/v1/tunnels/create", body)?;
    match resp.get("status").and_then(|s| s.as_str()) {
        Some("success") => Ok(()),
        Some("fail") => {
            let fail = resp
                .get("data")
                .map(|d| d.to_string())
                .unwrap_or_else(|| resp.to_string());
            // Idempotencia laxa: ya existe / límites se interpretan arriba.
            Err(AppError::msg(format!("No se pudo crear túnel {tunnel_type}: {fail}")))
        }
        _ => Err(AppError::msg(format!(
            "Respuesta create {tunnel_type}: {}",
            resp.to_string().chars().take(200).collect::<String>()
        ))),
    }
}

fn wait_for_type(
    secret: &str,
    tunnel_type: &str,
    attempts: u32,
) -> Option<String> {
    for _ in 0..attempts {
        if let Ok(list) = list_account_tunnels(secret) {
            if let Some(addr) = find_by_type(&list, tunnel_type) {
                return Some(addr);
            }
        }
        // rundata a veces muestra display antes que account list
        if let Ok(rd) = agent_rundata(secret) {
            if let Some(arr) = rd.get("tunnels").and_then(|t| t.as_array()) {
                if let Some(addr) = find_by_type(arr, tunnel_type) {
                    return Some(addr);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(900));
    }
    None
}

/// Asegura túneles Java (y opcionalmente Bedrock) en la cuenta del agent-secret.
pub fn ensure_tunnels(
    secret: &str,
    want_bedrock: bool,
    java_local_port: u16,
    bedrock_local_port: u16,
) -> AppResult<EnsuredTunnels> {
    let mut out = EnsuredTunnels::default();

    let mut tunnels = list_account_tunnels(secret)?;
    out.java_address = find_by_type(&tunnels, "minecraft-java");
    out.bedrock_address = find_by_type(&tunnels, "minecraft-bedrock");

    let agent_id = agent_rundata(secret)
        .ok()
        .and_then(|rd| {
            rd.get("agent_id")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
        });

    if out.java_address.is_none() {
        out.messages
            .push("Creando túnel Minecraft Java en playit.gg…".into());
        match create_tunnel(
            secret,
            "Paraguacraft Java",
            "minecraft-java",
            agent_id.as_deref(),
            java_local_port,
        ) {
            Ok(()) => {
                out.created_java = true;
                out.java_address = wait_for_type(secret, "minecraft-java", 12);
                if out.java_address.is_some() {
                    out.messages
                        .push(format!("✅ Túnel Java: {}", out.java_address.as_deref().unwrap_or("?")));
                } else {
                    out.messages
                        .push("Túnel Java creado; la dirección puede tardar unos segundos.".into());
                }
            }
            Err(e) => out.messages.push(format!("⚠ Java: {e}")),
        }
    }

    if want_bedrock && out.bedrock_address.is_none() {
        out.messages
            .push("Creando túnel Minecraft Bedrock (UDP) en playit.gg…".into());
        // refrescar agent_id / tunnels
        let agent_id = agent_rundata(secret)
            .ok()
            .and_then(|rd| {
                rd.get("agent_id")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string())
            })
            .or(agent_id);
        match create_tunnel(
            secret,
            "Paraguacraft Bedrock",
            "minecraft-bedrock",
            agent_id.as_deref(),
            bedrock_local_port.max(1),
        ) {
            Ok(()) => {
                out.created_bedrock = true;
                out.bedrock_address = wait_for_type(secret, "minecraft-bedrock", 12);
                if let Some(ref a) = out.bedrock_address {
                    out.messages
                        .push(format!("✅ Túnel Bedrock: {a}"));
                } else {
                    out.messages.push(
                        "Túnel Bedrock creado; la dirección puede tardar unos segundos.".into(),
                    );
                }
            }
            Err(e) => out.messages.push(format!("⚠ Bedrock: {e}")),
        }
    } else if want_bedrock {
        if let Some(ref a) = out.bedrock_address {
            out.messages
                .push(format!("Túnel Bedrock ya existía: {a}"));
        }
    }

    // Último refresh
    if let Ok(list) = list_account_tunnels(secret) {
        tunnels = list;
        if out.java_address.is_none() {
            out.java_address = find_by_type(&tunnels, "minecraft-java");
        }
        if out.bedrock_address.is_none() {
            out.bedrock_address = find_by_type(&tunnels, "minecraft-bedrock");
        }
    }

    Ok(out)
}

/// Solo lista lo que ya hay (sin crear).
pub fn list_tunnel_addresses(secret: &str) -> AppResult<(Option<String>, Option<String>)> {
    let tunnels = list_account_tunnels(secret)?;
    Ok((
        find_by_type(&tunnels, "minecraft-java"),
        find_by_type(&tunnels, "minecraft-bedrock"),
    ))
}
