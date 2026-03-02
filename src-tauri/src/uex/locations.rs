use serde::Deserialize;

use super::client::UexClient;
use super::types::{deserialize_flexible_id, deserialize_nonempty_string, UexResult};

// ── API DTOs ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct TerminalDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub displayname: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub fullname: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub code: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
}

impl From<&TerminalDto> for UexResult {
    fn from(dto: &TerminalDto) -> Self {
        let name = dto
            .displayname
            .as_deref()
            .or(dto.fullname.as_deref())
            .filter(|s| !s.is_empty())
            .unwrap_or(&dto.name)
            .to_string();

        let slug = dto
            .code
            .as_deref()
            .or(dto.slug.as_deref())
            .unwrap_or("")
            .to_string();

        Self {
            id: dto.id.clone(),
            name,
            kind: "location".to_string(),
            slug,
            uuid: String::new(),
        }
    }
}

// ── Public functions ───────────────────────────────────────────────────────

/// Fetch ALL locations (terminals) from UEX.
pub async fn fetch_all_locations(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let dtos: Vec<TerminalDto> = client.get("/terminals", &[], api_key).await?;
    Ok(dtos.iter().map(UexResult::from).collect())
}

/// Search UEX for locations (terminals) by query string (direct API call, no cache).
pub async fn search_locations(
    client: &UexClient,
    query: &str,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let dtos: Vec<TerminalDto> = client
        .get("/terminals", &[("name_filter", query)], api_key)
        .await?;
    let query_lower = query.to_lowercase();
    Ok(dtos
        .iter()
        .map(UexResult::from)
        .filter(|r| r.name.to_lowercase().contains(&query_lower))
        .collect())
}
