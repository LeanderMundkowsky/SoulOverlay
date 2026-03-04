use serde::Deserialize;

use super::client::UexClient;
use super::types::{
    deserialize_bool_flag, deserialize_flexible_id, deserialize_nonempty_string, UexUserProfile,
};

// ── API DTO ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct UserDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    username: String,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    email: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    avatar: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    bio: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    website_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    timezone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    language: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    discord_username: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    twitch_username: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    day_availability: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    time_availability: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    specializations: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    languages: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    archetypes: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    is_datarunner: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    is_datarunner_banned: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    is_staff: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    is_away_game: Option<bool>,
    #[serde(default)]
    date_added: Option<serde_json::Value>,
    #[serde(default)]
    date_modified: Option<serde_json::Value>,
    #[serde(default)]
    date_rsi_verified: Option<serde_json::Value>,
    #[serde(default)]
    date_twitch_verified: Option<serde_json::Value>,
}

/// Split a CSV string into a Vec of trimmed, non-empty strings.
fn split_csv(csv: &Option<String>) -> Vec<String> {
    csv.as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Convert an optional JSON value (number or string) to an optional String.
fn timestamp_opt(val: &Option<serde_json::Value>) -> Option<String> {
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

// ── Public function ────────────────────────────────────────────────────────

/// Fetch the authenticated user's profile from UEX.
pub async fn fetch_user_profile(
    client: &UexClient,
    api_key: &str,
    secret_key: &str,
) -> Result<UexUserProfile, String> {
    let body = client
        .get_raw_with_secret("/user", &[], api_key, secret_key)
        .await?;

    // /user returns data as a single object, not an array
    let data = body.get("data")
        .ok_or_else(|| "No data field in UEX user response".to_string())?;

    let dto: UserDto = serde_json::from_value(data.clone())
        .map_err(|e| format!("Failed to parse user profile: {}", e))?;

    Ok(UexUserProfile::from(&dto))
}
