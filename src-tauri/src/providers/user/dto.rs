use serde::Deserialize;

use crate::uex::types::{
    deserialize_bool_flag, deserialize_flexible_id, deserialize_nonempty_string, UexUserProfile,
};

#[derive(Deserialize)]
pub struct UserDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub username: String,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub email: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub avatar: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub bio: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub website_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub timezone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub language: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub discord_username: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub twitch_username: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub day_availability: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub time_availability: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub specializations: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub languages: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub archetypes: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_datarunner: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_datarunner_banned: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_staff: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_away_game: Option<bool>,
    #[serde(default)]
    pub date_added: Option<serde_json::Value>,
    #[serde(default)]
    pub date_modified: Option<serde_json::Value>,
    #[serde(default)]
    pub date_rsi_verified: Option<serde_json::Value>,
    #[serde(default)]
    pub date_twitch_verified: Option<serde_json::Value>,
}

/// Split a CSV string into a Vec of trimmed, non-empty strings.
pub fn split_csv(csv: &Option<String>) -> Vec<String> {
    csv.as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Convert an optional JSON value (number or string) to an optional String.
pub fn timestamp_opt(val: &Option<serde_json::Value>) -> Option<String> {
    val.as_ref().and_then(|v| match v {
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    })
}

impl From<&UserDto> for UexUserProfile {
    fn from(dto: &UserDto) -> Self {
        Self {
            id: dto.id.parse::<u32>().unwrap_or(0),
            name: dto.name.clone(),
            username: dto.username.clone(),
            email: dto.email.clone(),
            avatar: dto.avatar.clone(),
            bio: dto.bio.clone(),
            website_url: dto.website_url.clone(),
            timezone: dto.timezone.clone(),
            language: dto.language.clone(),
            discord_username: dto.discord_username.clone(),
            twitch_username: dto.twitch_username.clone(),
            day_availability: split_csv(&dto.day_availability),
            time_availability: split_csv(&dto.time_availability),
            specializations: split_csv(&dto.specializations),
            languages: split_csv(&dto.languages),
            archetypes: split_csv(&dto.archetypes),
            is_datarunner: dto.is_datarunner.unwrap_or(false),
            is_datarunner_banned: dto.is_datarunner_banned.unwrap_or(false),
            is_staff: dto.is_staff.unwrap_or(false),
            is_away_game: dto.is_away_game.unwrap_or(false),
            date_added: timestamp_opt(&dto.date_added),
            date_modified: timestamp_opt(&dto.date_modified),
            date_rsi_verified: timestamp_opt(&dto.date_rsi_verified),
            date_twitch_verified: timestamp_opt(&dto.date_twitch_verified),
        }
    }
}
