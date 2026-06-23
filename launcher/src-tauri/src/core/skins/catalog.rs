//! Catálogo de skins vía MineSkin (millones de skins) + búsqueda por jugador Mojang.

use reqwest::Client;
use serde::Deserialize;

use crate::core::skins::mojang;
use crate::error::{AppError, AppResult};

pub const PAGE_SIZE: usize = 15;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinCatalogEntry {
    pub id: String,
    pub label: String,
    pub skin_url: String,
    pub preview_url: String,
    /// `player` | `mineskin`
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinCatalogPage {
    pub entries: Vec<SkinCatalogEntry>,
    pub page: u32,
    pub total_pages: u32,
    pub total_skins: u64,
    pub query: String,
}

#[derive(Debug, Deserialize)]
struct MineskinListResponse {
    #[serde(default)]
    skins: Vec<MineskinListItem>,
    page: MineskinPageMeta,
}

#[derive(Debug, Deserialize)]
struct MineskinListItem {
    uuid: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct MineskinPageMeta {
    index: u32,
    amount: u32,
    total: u64,
}

#[derive(Debug, Deserialize)]
struct MineskinDetailResponse {
    #[serde(default)]
    name: String,
    #[serde(default)]
    model: String,
}

/// Jugadores populares para búsqueda parcial por nombre (complemento a Mojang).
pub const POPULAR_PLAYERS: &[&str] = &[
    "Dream", "Technoblade", "Notch", "jeb_", "Dinnerbone", "Grumm", "MrBeast", "xNestorio",
    "Quackity", "TommyInnit", "Tubbo", "Wilbur_Soot", "Philza", "BadBoyHalo", "Skeppy", "Sapnap",
    "GeorgeNotFound", "Punz", "Ranboo", "HBomb94", "Eret", "Fundy", "Nihachu", "Karl_Jacobs",
    "Antfrost", "Purpled", "Slimecicle", "Sneegsnag", "ConnorEatsPants", "Hannahxxrose",
    "xIsuma", "GoodTimesWithScar", "Grian", "Mumbo", "Rendog", "Etho", "BdoubleO100", "Tango",
    "ZombieCleo", "PearlescentMoon", "Iskall85", "Hypnotizd", "Cubfan135", "FalseSymmetry",
    "Impulsesv", "Keralis", "SmallishBeans", "VintageBeef", "xBCrafted", "Jessassin",
    "CaptainSparklez", "Syndicate", "Stampylonghead", "DanTDM", "LDShadowLady",
    "iBallisticSquid", "PopbobMC", "jschlatt", "Foolish_Gamers", "Awesamdude", "Skydoesminecraft",
    "AntVenom", "Paulsoaresjr", "SethBling", "Docm77", "JoeHills", "Biffa2001", "GeminiTay",
    "Stressmonster101", "Welsknight", "TheCampingRusher", "Vikkstar123", "Lachlan", "PrestonPlayz",
    "JeromeASF", "BajanCanadian", "Logdotzip", "UnspeakableGaming", "Guude", "KurtJMac",
    "PewDiePie", "KSI", "Miniminter", "Fruitberries", "Illumina", "PeteZahHutt", "Hypixel",
    "Steve", "Alex", "San_Jlf", "SantiJ10", "Aphmau", "PopularMMOs", "ExplodingTNT",
];

pub async fn search(
    client: &Client,
    query: &str,
    page: u32,
    random: bool,
) -> AppResult<SkinCatalogPage> {
    let q = query.trim();
    if q.is_empty() {
        return fetch_mineskin_page(client, page, random).await;
    }
    search_players(client, q, page).await
}

async fn fetch_mineskin_page(
    client: &Client,
    page: u32,
    random: bool,
) -> AppResult<SkinCatalogPage> {
    let first: MineskinListResponse = client
        .get("https://api.mineskin.org/get/list/1")
        .header("User-Agent", mineskin_user_agent())
        .send()
        .await
        .map_err(|e| AppError::msg(format!("MineSkin no disponible: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::msg(format!("MineSkin respondió error: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::msg(format!("Respuesta MineSkin inválida: {e}")))?;

    let total_pages = first.page.amount.max(1);
    let total_skins = first.page.total;

    let page_num = if random {
        (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as u32
            % total_pages)
            + 1
    } else {
        page.max(1).min(total_pages)
    };

    let data: MineskinListResponse = if page_num == 1 {
        first
    } else {
        client
            .get(format!("https://api.mineskin.org/get/list/{page_num}"))
            .header("User-Agent", mineskin_user_agent())
            .send()
            .await
            .map_err(|e| AppError::msg(format!("MineSkin no disponible: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::msg(format!("MineSkin respondió error: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::msg(format!("Respuesta MineSkin inválida: {e}")))?
    };

    let tasks: Vec<_> = data
        .skins
        .into_iter()
        .map(|item| {
            let client = client.clone();
            async move {
                let label = fetch_mineskin_label(&client, &item.uuid)
                    .await
                    .unwrap_or_else(|| short_label(&item.uuid));
                let skin_url = item.url.replace("http://", "https://");
                let preview_url = catalog_body_preview(&label, &skin_url);
                SkinCatalogEntry {
                    id: item.uuid,
                    label,
                    skin_url,
                    preview_url,
                    kind: "mineskin".into(),
                    model: None,
                }
            }
        })
        .collect();
    let entries: Vec<SkinCatalogEntry> = futures_util::future::join_all(tasks).await;

    Ok(SkinCatalogPage {
        entries,
        page: page_num,
        total_pages,
        total_skins,
        query: String::new(),
    })
}

async fn search_players(client: &Client, query: &str, page: u32) -> AppResult<SkinCatalogPage> {
    let lc = query.to_lowercase();
    let mut names: Vec<String> = Vec::new();

    if page == 1 {
        names.push(query.to_string());
        if let Ok(lookup) = mojang::lookup_player(client, Some(query), None).await {
            if lookup.ok && !names.iter().any(|n| n.eq_ignore_ascii_case(&lookup.username)) {
                names.push(lookup.username);
            }
        }
    }

    for p in POPULAR_PLAYERS {
        if p.to_lowercase().contains(&lc) && !names.iter().any(|n| n.eq_ignore_ascii_case(p)) {
            names.push((*p).to_string());
        }
    }

    let total_pages = ((names.len() + PAGE_SIZE - 1) / PAGE_SIZE).max(1) as u32;
    let page_num = page.max(1).min(total_pages);
    let start = ((page_num - 1) as usize) * PAGE_SIZE;
    let slice = names[start..names.len().min(start + PAGE_SIZE)].to_vec();

    let mut entries = Vec::new();
    for name in slice {
        let lookup = mojang::lookup_player(client, Some(&name), None).await;
        if let Ok(info) = lookup {
            if info.ok {
                let skin_url = info
                    .skin_url
                    .clone()
                    .unwrap_or_else(|| minotar_skin_url(&info.username));
                entries.push(SkinCatalogEntry {
                    id: info.username.clone(),
                    label: info.username.clone(),
                    skin_url: skin_url.clone(),
                    preview_url: body_preview_url(&info.username, 160),
                    kind: "player".into(),
                    model: Some(info.model),
                });
                continue;
            }
        }
        entries.push(SkinCatalogEntry {
            id: name.clone(),
            label: name.clone(),
            skin_url: minotar_skin_url(&name),
            preview_url: body_preview_url(&name, 160),
            kind: "player".into(),
            model: Some("classic".into()),
        });
    }

    Ok(SkinCatalogPage {
        entries,
        page: page_num,
        total_pages,
        total_skins: names.len() as u64,
        query: query.to_string(),
    })
}

async fn fetch_mineskin_label(client: &Client, uuid: &str) -> Option<String> {
    let text = client
        .get(format!("https://api.mineskin.org/get/uuid/{uuid}"))
        .header("User-Agent", mineskin_user_agent())
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?;

    if let Ok(detail) = serde_json::from_str::<MineskinDetailResponse>(&text) {
        if !detail.name.trim().is_empty() {
            return Some(detail.name);
        }
    }

    extract_profile_name(&text)
}

fn extract_profile_name(json: &str) -> Option<String> {
    let needle = "\"profileName\"";
    let start = json.find(needle)? + needle.len();
    let rest = json.get(start..)?.trim_start();
    if !rest.starts_with(':') {
        return None;
    }
    let rest = rest.trim_start_matches(':').trim();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    let name = rest[..end].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn short_label(uuid: &str) -> String {
    if uuid.len() >= 8 {
        format!("Skin {}", &uuid[..8])
    } else {
        uuid.to_string()
    }
}

fn mineskin_user_agent() -> &'static str {
    "ParaguacraftLauncher/1.0 (+https://github.com/paraguacraft)"
}

pub fn body_preview_url(id: &str, size: u32) -> String {
    format!(
        "https://minotar.net/armor/body/{}/{size}.png",
        urlencoding_encode(id),
        size = size.max(32).min(512)
    )
}

/// Vista previa tipo NameMC: cuerpo 3D, no la textura plana 64×64.
pub fn catalog_body_preview(label: &str, texture_url: &str) -> String {
    if is_generic_skin_label(label) {
        if let Some(hash) = texture_hash_from_url(texture_url) {
            return mc_heads_body_url(&hash, 160);
        }
    }
    body_preview_url(label, 160)
}

fn is_generic_skin_label(label: &str) -> bool {
    label.starts_with("Skin ")
}

fn texture_hash_from_url(url: &str) -> Option<String> {
    url.rsplit('/')
        .next()
        .filter(|h| h.len() >= 32 && h.chars().all(|c| c.is_ascii_hexdigit()))
        .map(|s| s.to_string())
}

fn mc_heads_body_url(hash: &str, size: u32) -> String {
    format!(
        "https://mc-heads.net/body/{}/{size}",
        hash,
        size = size.max(32).min(512)
    )
}

pub fn helm_preview_url(id: &str, size: u32) -> String {
    format!("https://minotar.net/helm/{id}/{}.png", size.max(16).min(512))
}

fn minotar_skin_url(username: &str) -> String {
    format!(
        "https://minotar.net/skin/{}",
        urlencoding_encode(username)
    )
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
