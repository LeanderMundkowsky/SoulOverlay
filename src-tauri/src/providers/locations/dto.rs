use serde::Deserialize;

use crate::uex::types::{deserialize_flexible_id, deserialize_nonempty_string, UexResult};

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct TerminalDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub star_system_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub planet_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub orbit_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub faction_name: Option<String>,
}

impl From<&TerminalDto> for UexResult {
    fn from(dto: &TerminalDto) -> Self {
        let display = if let Some(nick) = &dto.nickname {
            format!("{} ({})", nick, dto.name)
        } else {
            dto.name.clone()
        };
        Self {
            id: dto.id.clone(),
            name: display,
            kind: "location".to_string(),
            slug: String::new(),
            uuid: String::new(),
        }
    }
}
